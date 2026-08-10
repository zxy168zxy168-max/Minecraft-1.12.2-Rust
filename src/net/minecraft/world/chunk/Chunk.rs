use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

// Renderer-only content stamp. Minecraft does not expose a Chunk revision;
// Rust uses this process-wide monotonic stamp so replacing a Chunk object on a
// full SPacketChunkData load can never alias the section revision of the old
// object. This preserves ChunkProviderClient#loadChunk replacement semantics
// while keeping asynchronous RenderChunk invalidation deterministic.
static NEXT_RENDER_REVISION: AtomicU64 = AtomicU64::new(1);

#[inline]
fn next_render_revision() -> u64 {
    NEXT_RENDER_REVISION.fetch_add(1, Ordering::Relaxed)
}

use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::nbt::NBTBase::TAG_DOUBLE;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::world::chunk::storage::ExtendedBlockStorage::ExtendedBlockStorage;
use crate::net::minecraft::world::chunk::ChunkPrimer::ChunkPrimer;

/// Rust equivalent of MCP `Chunk`'s section ownership.
///
/// RenderChunk jobs need an immutable snapshot while the network thread may
/// replace or mutate a section. `Arc::make_mut` provides copy-on-write section
/// snapshots: cloning a Chunk for background tessellation is cheap, while a
/// later server block update cannot alter the worker's captured data.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub xPosition: i32,
    pub zPosition: i32,
    storageArrays: Vec<Option<Arc<ExtendedBlockStorage>>>,
    blockBiomeArray: [u8; 256],

    // MCP `Chunk` object ownership. The concrete server-side Entity and
    // TileEntity class hierarchy is still migrating, so these collections hold
    // authoritative NBT snapshots rather than client render surrogates. Their
    // shape mirrors `entityLists[16]` and `tileEntities` and is lossless across
    // Anvil async saves; later concrete objects can replace each snapshot in
    // place without changing Chunk ownership or disk layout.
    entityLists: Vec<Vec<NBTTagCompound>>,
    tileEntities: HashMap<BlockPos, NBTTagCompound>,

    // MCP `Chunk` persistent/server state.
    precipitationHeightMap: [i32; 256],
    updateSkylightColumns: [bool; 256],
    isChunkLoaded: bool,
    heightMap: [i32; 256],
    isGapLightingUpdated: bool,
    isTerrainPopulated: bool,
    isLightPopulated: bool,
    chunkTicked: bool,
    isModified: bool,
    hasEntities: bool,
    lastSaveTime: i64,
    heightMapMinimum: i32,
    inhabitedTime: i64,
    queuedLightChecks: i32,
    pub unloaded: bool,

    // Rust renderer-only revisions. These are not serialized to Anvil NBT.
    revision: u64,
    sectionRevisions: [u64; 16],
}

impl Chunk {
    pub fn new(x: i32, z: i32) -> Self {
        let initialRevision = next_render_revision();
        Self {
            xPosition: x,
            zPosition: z,
            storageArrays: vec![None; 16],
            // Java fills the biome array with -1 (0xFF) until populated.
            blockBiomeArray: [0xFF; 256],
            entityLists: (0..16).map(|_| Vec::new()).collect(),
            tileEntities: HashMap::new(),
            precipitationHeightMap: [-999; 256],
            updateSkylightColumns: [false; 256],
            isChunkLoaded: false,
            heightMap: [0; 256],
            isGapLightingUpdated: false,
            isTerrainPopulated: false,
            isLightPopulated: false,
            chunkTicked: false,
            isModified: false,
            hasEntities: false,
            lastSaveTime: 0,
            heightMapMinimum: 0,
            inhabitedTime: 0,
            queuedLightChecks: 4096,
            unloaded: false,
            revision: initialRevision,
            sectionRevisions: [initialRevision; 16],
        }
    }

    pub fn setStorage(&mut self, index: usize, storage: Option<ExtendedBlockStorage>) {
        if index < 16 {
            self.storageArrays[index] = storage.map(Arc::new);
            let revision = next_render_revision();
            self.revision = revision;
            self.sectionRevisions[index] = revision;
        }
    }

    pub fn getBlockStorageArray(&self) -> &[Option<Arc<ExtendedBlockStorage>>] {
        &self.storageArrays
    }

    pub fn setBiomeArray(&mut self, data: &[u8]) {
        if data.len() != self.blockBiomeArray.len() {
            log::warn!("Could not set level chunk biomes, array length is {} instead of {}", data.len(), self.blockBiomeArray.len());
            return;
        }
        self.blockBiomeArray.copy_from_slice(data);
        let revision = next_render_revision();
        self.revision = revision;
        for sectionRevision in &mut self.sectionRevisions {
            *sectionRevision = revision;
        }
    }

    pub fn getBiomeArray(&self) -> &[u8; 256] {
        &self.blockBiomeArray
    }

    /// MCP `Chunk#isAtLocation`.
    pub const fn isAtLocation(&self, x: i32, z: i32) -> bool {
        self.xPosition == x && self.zPosition == z
    }

    pub fn getHeightMap(&self) -> &[i32; 256] { &self.heightMap }
    pub fn setHeightMap(&mut self, values: &[i32]) {
        if values.len() != self.heightMap.len() {
            log::warn!("Could not set level chunk heightmap, array length is {} instead of {}", values.len(), self.heightMap.len());
            return;
        }
        self.heightMap.copy_from_slice(values);
    }
    pub const fn getHeightValue(&self, x: usize, z: usize) -> i32 {
        if x < 16 && z < 16 { self.heightMap[(z << 4) | x] } else { 0 }
    }
    pub fn getPrecipitationHeightMap(&self) -> &[i32; 256] { &self.precipitationHeightMap }
    pub fn getUpdateSkylightColumns(&self) -> &[bool; 256] { &self.updateSkylightColumns }
    pub const fn isLoaded(&self) -> bool { self.isChunkLoaded }
    pub fn setLoaded(&mut self, value: bool) { self.isChunkLoaded = value; }
    pub const fn isGapLightingUpdated(&self) -> bool { self.isGapLightingUpdated }
    pub fn setGapLightingUpdated(&mut self, value: bool) { self.isGapLightingUpdated = value; }
    pub const fn isTerrainPopulated(&self) -> bool { self.isTerrainPopulated }
    pub fn setTerrainPopulated(&mut self, value: bool) { self.isTerrainPopulated = value; }
    pub const fn isLightPopulated(&self) -> bool { self.isLightPopulated }
    pub fn setLightPopulated(&mut self, value: bool) { self.isLightPopulated = value; }
    pub const fn isChunkTicked(&self) -> bool { self.chunkTicked }
    pub fn setChunkTicked(&mut self, value: bool) { self.chunkTicked = value; }
    pub const fn isModified(&self) -> bool { self.isModified }
    pub fn setModified(&mut self, value: bool) { self.isModified = value; }
    pub const fn hasEntities(&self) -> bool { self.hasEntities }
    pub fn setHasEntities(&mut self, value: bool) { self.hasEntities = value; }
    pub const fn getLastSaveTime(&self) -> i64 { self.lastSaveTime }
    pub fn setLastSaveTime(&mut self, value: i64) { self.lastSaveTime = value; }
    pub const fn getHeightMapMinimum(&self) -> i32 { self.heightMapMinimum }
    pub const fn getInhabitedTime(&self) -> i64 { self.inhabitedTime }
    pub fn setInhabitedTime(&mut self, value: i64) { self.inhabitedTime = value; }
    pub const fn getQueuedLightChecks(&self) -> i32 { self.queuedLightChecks }
    pub fn setQueuedLightChecks(&mut self, value: i32) { self.queuedLightChecks = value; }
    pub fn resetRelightChecks(&mut self) { self.queuedLightChecks = 0; }
    pub const fn getLowestHeight(&self) -> i32 { self.heightMapMinimum }

    /// MCP `Chunk#setStorageArrays`, retaining the renderer's immutable Arc
    /// snapshot ownership.  Every replacement receives a fresh global render
    /// revision so an old world's asynchronous mesh cannot alias the loaded
    /// server chunk.
    pub fn setStorageArrays(&mut self, values: Vec<Option<ExtendedBlockStorage>>) {
        if values.len() != self.storageArrays.len() {
            log::warn!("Could not set level chunk sections, array length is {} instead of {}", values.len(), self.storageArrays.len());
            return;
        }
        self.storageArrays = values.into_iter().map(|entry| entry.map(Arc::new)).collect();
        let revision = next_render_revision();
        self.revision = revision;
        self.sectionRevisions = [revision; 16];
    }

    /// Top-most present storage section, corresponding to
    /// `Chunk#getTopFilledSegment` (the y-base, not the top block coordinate).
    pub fn getTopFilledSegment(&self) -> i32 {
        self.storageArrays.iter().rev().flatten().next().map(|storage| storage.getYLocation()).unwrap_or(0)
    }

    /// MCP `Chunk#setChunkModified`.
    pub fn setChunkModified(&mut self) { self.isModified = true; }

    /// MCP `Chunk#needsSaving`, with `World#getTotalWorldTime` supplied by the
    /// caller until the complete server `World` ownership is present.
    pub const fn needsSaving(&self, force: bool, totalWorldTime: i64) -> bool {
        if force {
            if (self.hasEntities && totalWorldTime != self.lastSaveTime) || self.isModified {
                return true;
            }
        } else if self.hasEntities && totalWorldTime >= self.lastSaveTime + 600 {
            return true;
        }
        self.isModified
    }

    /// MCP `Chunk#getRandomWithSeed`. Java performs several multiplications as
    /// 32-bit int arithmetic before widening to long; the explicit wrapping
    /// operations preserve that overflow behaviour.
    pub fn getRandomWithSeed(&self, worldSeed: i64, seed: i64) -> crate::compat::Java::JavaRandom {
        let xx = self.xPosition.wrapping_mul(self.xPosition).wrapping_mul(4_987_142) as i64;
        let x = self.xPosition.wrapping_mul(5_947_611) as i64;
        let zz = self.zPosition.wrapping_mul(self.zPosition) as i64;
        let z = self.zPosition.wrapping_mul(389_711) as i64;
        let mixed = worldSeed
            .wrapping_add(xx)
            .wrapping_add(x)
            .wrapping_add(zz.wrapping_mul(4_392_871))
            .wrapping_add(z)
            ^ seed;
        crate::compat::Java::JavaRandom::new(mixed)
    }

    /// Persistence-stage equivalent of MCP `Chunk#getEntityLists`. Each entry
    /// is the exact root entity NBT produced by the source save format. Root
    /// passengers remain nested under `Passengers`, matching `writeToNBTOptional`.
    pub fn getEntityListsData(&self) -> &[Vec<NBTTagCompound>] { &self.entityLists }

    pub fn clearEntityData(&mut self) {
        for list in &mut self.entityLists { list.clear(); }
        self.hasEntities = false;
    }

    /// Source `Chunk#addEntity` chooses the vertical list by floor(posY/16),
    /// clamped to [0,15]. Missing/malformed Pos behaves like NBTTagList numeric
    /// access and therefore resolves to y=0.
    pub fn addEntityData(&mut self, compound: NBTTagCompound) {
        let pos = compound.getTagList("Pos", TAG_DOUBLE);
        let slice = ((pos.getDoubleAt(1).floor() as i32) >> 4).clamp(0, 15) as usize;
        self.entityLists[slice].push(compound);
        self.hasEntities = true;
    }

    /// MCP `Chunk#getTileEntityMap` persistence boundary.
    pub fn getTileEntityMapData(&self) -> &HashMap<BlockPos, NBTTagCompound> { &self.tileEntities }

    pub fn clearTileEntityData(&mut self) { self.tileEntities.clear(); }

    /// TileEntity base NBT always owns x/y/z. Unknown ids are retained here
    /// rather than silently discarded; `TileEntity::create` can take over once
    /// the concrete server registry is complete.
    pub fn addTileEntityData(&mut self, compound: NBTTagCompound) {
        let pos = BlockPos::new(compound.getInteger("x"), compound.getInteger("y"), compound.getInteger("z"));
        self.tileEntities.insert(pos, compound);
    }

    pub fn removeTileEntityData(&mut self, pos: &BlockPos) -> Option<NBTTagCompound> {
        self.tileEntities.remove(pos)
    }

    /// MCP `Chunk(World, ChunkPrimer, int, int)` with the provider skylight
    /// capability supplied explicitly until the common server `World` object
    /// owns the provider reference in the same way as Java.
    pub fn fromPrimer(primer: &ChunkPrimer, x: i32, z: i32, hasSkyLight: bool) -> Result<Self, String> {
        let mut chunk = Self::new(x, z);
        for cx in 0..16 {
            for cz in 0..16 {
                for y in 0..256 {
                    let state = primer.getBlockState(cx, y, cz);
                    if state.isAir() { continue; }
                    let section = y >> 4;
                    if chunk.storageArrays[section].is_none() {
                        chunk.storageArrays[section] = Some(Arc::new(ExtendedBlockStorage::new((section << 4) as i32, hasSkyLight)));
                    }
                    Arc::make_mut(chunk.storageArrays[section].as_mut().expect("primer section"))
                        .set(cx, y & 15, cz, state)?;
                }
            }
        }
        let revision = next_render_revision();
        chunk.revision = revision;
        chunk.sectionRevisions = [revision; 16];
        Ok(chunk)
    }

    fn getBlockLightOpacity(&self, x: usize, y: usize, z: usize) -> i32 {
        self.getBlockState(x, y, z).getLightOpacity()
    }

    /// MCP `Chunk#generateSkylightMap`. `hasSkyLight` is the provider's
    /// `func_191066_m()` value; world light notifications are renderer/runtime
    /// side effects and are not required to build the authoritative arrays.
    pub fn generateSkylightMap(&mut self, hasSkyLight: bool) {
        let top = self.getTopFilledSegment();
        self.heightMapMinimum = i32::MAX;
        for x in 0..16 {
            for z in 0..16 {
                self.precipitationHeightMap[x + (z << 4)] = -999;
                for y in (1..=top + 16).rev() {
                    if self.getBlockLightOpacity(x, (y - 1) as usize, z) != 0 {
                        self.heightMap[(z << 4) | x] = y;
                        if y < self.heightMapMinimum { self.heightMapMinimum = y; }
                        break;
                    }
                }
                if hasSkyLight {
                    let mut light = 15_i32;
                    let mut y = top + 15;
                    loop {
                        let mut opacity = self.getBlockLightOpacity(x, y as usize, z);
                        if opacity == 0 && light != 15 { opacity = 1; }
                        light -= opacity;
                        if light > 0 {
                            if let Some(storage) = self.storageArrays[(y >> 4) as usize].as_mut() {
                                Arc::make_mut(storage).setExtSkylightValue(x, (y & 15) as usize, z, light as u8);
                            }
                        }
                        y -= 1;
                        if y <= 0 || light <= 0 { break; }
                    }
                }
            }
        }
        // MCP intentionally leaves heightMapMinimum at Integer.MAX_VALUE for
        // an entirely empty column/chunk; do not normalize the sentinel.
        self.isModified = true;
    }

    pub fn getGlobalStateId(&self, x: usize, y: usize, z: usize) -> i32 {
        if x >= 16 || z >= 16 || y >= 256 {
            return 0;
        }
        self.storageArrays[y >> 4]
            .as_ref()
            .map(|storage| storage.getGlobalStateId(x, y & 15, z))
            .unwrap_or(0)
    }

    pub fn getBlockState(&self, x: usize, y: usize, z: usize) -> IBlockState {
        IBlockState::fromGlobalStateId(self.getGlobalStateId(x, y, z))
    }

    pub fn setBlockState(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        state: IBlockState,
        hasSkyLight: bool,
    ) -> Result<IBlockState, String> {
        if x >= 16 || z >= 16 || y >= 256 {
            return Err("block coordinate outside chunk".to_owned());
        }
        let sectionIndex = y >> 4;
        if self.storageArrays[sectionIndex].is_none() {
            self.storageArrays[sectionIndex] = Some(Arc::new(ExtendedBlockStorage::new(
                (sectionIndex * 16) as i32,
                hasSkyLight,
            )));
        }
        let storage = Arc::make_mut(
            self.storageArrays[sectionIndex]
                .as_mut()
                .expect("created section"),
        );
        let old = storage.set(x, y & 15, z, state)?;
        self.isModified = true;
        let revision = next_render_revision();
        self.revision = revision;
        self.sectionRevisions[sectionIndex] = revision;
        Ok(old)
    }

    /// Render-only invalidation for tile entities whose NBT changes the
    /// baked block model without changing the compact block state (notably
    /// `BlockFlowerPot.CONTENTS`).
    pub fn markSectionDirty(&mut self, sectionIndex: usize) {
        if sectionIndex < 16 {
            let revision = next_render_revision();
            self.revision = revision;
            self.sectionRevisions[sectionIndex] = revision;
        }
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Revision of one 16-block-high storage section. RenderChunk invalidation
    /// uses this instead of rebuilding the complete 16 x 256 x 16 column.
    pub const fn sectionRevision(&self, sectionIndex: usize) -> u64 {
        if sectionIndex < 16 {
            self.sectionRevisions[sectionIndex]
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_snapshot_is_copy_on_write_after_block_update() {
        let mut chunk = Chunk::new(0, 0);
        chunk
            .setBlockState(1, 2, 3, IBlockState::fromGlobalStateId(16), true)
            .unwrap();
        let snapshot = chunk.clone();
        chunk
            .setBlockState(1, 2, 3, IBlockState::fromGlobalStateId(32), true)
            .unwrap();
        assert_eq!(snapshot.getGlobalStateId(1, 2, 3), 16);
        assert_eq!(chunk.getGlobalStateId(1, 2, 3), 32);
    }
    #[test]
    fn block_update_only_advances_its_render_section_revision() {
        let mut chunk = Chunk::new(0, 0);
        let before_low = chunk.sectionRevision(1);
        let before_high = chunk.sectionRevision(9);
        chunk
            .setBlockState(1, 20, 3, IBlockState::fromGlobalStateId(16), true)
            .unwrap();
        assert_ne!(chunk.sectionRevision(1), before_low);
        assert_eq!(chunk.sectionRevision(9), before_high);
    }

    #[test]
    fn fresh_chunk_objects_never_alias_render_revisions() {
        let first = Chunk::new(4, -2);
        let second = Chunk::new(4, -2);
        for section in 0..16 {
            assert_ne!(first.sectionRevision(section), second.sectionRevision(section));
        }
    }

    #[test]
    fn needs_saving_matches_entity_and_modified_cadence() {
        let mut chunk = Chunk::new(0, 0);
        chunk.setLastSaveTime(100);
        assert!(!chunk.needsSaving(false, 699));
        chunk.setHasEntities(true);
        assert!(!chunk.needsSaving(false, 699));
        assert!(chunk.needsSaving(false, 700));
        assert!(chunk.needsSaving(true, 101));
        chunk.setHasEntities(false);
        chunk.setModified(true);
        assert!(chunk.needsSaving(false, 0));
    }

}
