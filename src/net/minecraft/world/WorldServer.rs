use std::collections::{BTreeSet, HashSet};
use std::io;

use crate::compat::Java::JavaRandom;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::Block::Block;
use crate::net::minecraft::item::ItemBlock::isReplaceableState;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::world::chunk::storage::AnvilChunkLoader::{
    AnvilChunkLoader, LoadedChunk,
};
use crate::net::minecraft::world::chunk::storage::AnvilSaveHandler::AnvilSaveHandler;
use crate::net::minecraft::world::chunk::Chunk::Chunk;
use crate::net::minecraft::world::gen::ChunkProviderServer::ChunkProviderServer;
use crate::net::minecraft::world::storage::WorldInfo::WorldInfo;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;
use crate::net::minecraft::world::NextTickListEntry::NextTickListEntry;
use crate::net::minecraft::world::WorldProvider::WorldProvider;
use crate::net::minecraft::world::WorldSettings::WorldSettings;
use crate::net::minecraft::world::WorldType::WorldType;

/// Source-backed persistence/scheduled-update tranche of MCP 1.12.2
/// `WorldServer`.
///
/// This type now owns the first real server ChunkProvider/generator and
/// PlayerChunkMap-adjacent access required by the integrated flat-world path.
/// Entity ticking, EntityTracker, block-event execution and the complete common
/// `World` runtime remain explicit later tranches rather than no-op stand-ins.
#[derive(Debug)]
pub struct WorldServer {
    saveHandler: AnvilSaveHandler,
    pub worldInfo: WorldInfo,
    pub provider: WorldProvider,
    pendingTickListEntriesHashSet: HashSet<NextTickListEntry>,
    pendingTickListEntriesTreeSet: BTreeSet<NextTickListEntry>,
    pendingTickListEntriesThisTick: Vec<NextTickListEntry>,
    chunkProvider: Option<ChunkProviderServer>,
    seaLevel: i32,
    findingSpawnPoint: bool,
    pub disableLevelSaving: bool,
}

impl WorldServer {
    pub fn new(saveHandlerIn: AnvilSaveHandler, info: WorldInfo, dimensionId: i32) -> Self {
        let mut provider = WorldProvider::new(dimensionId);
        provider.configureFromWorldInfo(&info);
        Self {
            saveHandler: saveHandlerIn,
            worldInfo: info,
            provider,
            pendingTickListEntriesHashSet: HashSet::new(),
            pendingTickListEntriesTreeSet: BTreeSet::new(),
            pendingTickListEntriesThisTick: Vec::new(),
            chunkProvider: None,
            seaLevel: 63,
            findingSpawnPoint: false,
            disableLevelSaving: false,
        }
    }

    /// MCP `WorldServer#init` / `createChunkProvider` boundary. The concrete
    /// provider owns the save loader and the dimension's real IChunkGenerator.
    pub fn init(mut self) -> Result<Self, String> {
        let loader = self
            .saveHandler
            .getChunkLoader(&self.provider)
            .map_err(|error| error.to_string())?;
        let generator = self.provider.createChunkGenerator(
            self.worldInfo.getSeed(),
            self.worldInfo.isMapFeaturesEnabled(),
        )?;
        let seaLevel = generator.seaLevelOverride();
        self.chunkProvider = Some(ChunkProviderServer::withGenerator(loader, generator));
        if let Some(seaLevel) = seaLevel {
            self.seaLevel = seaLevel;
        }
        Ok(self)
    }

    pub const fn getSeed(&self) -> i64 {
        self.worldInfo.getSeed()
    }
    pub const fn getSeaLevel(&self) -> i32 {
        self.seaLevel
    }
    pub fn setSeaLevel(&mut self, level: i32) {
        self.seaLevel = level;
    }
    pub fn getSpawnPoint(&self) -> BlockPos {
        BlockPos::new(
            self.worldInfo.getSpawnX(),
            self.worldInfo.getSpawnY(),
            self.worldInfo.getSpawnZ(),
        )
    }
    pub fn getBiomeProvider(
        &self,
    ) -> Option<&crate::net::minecraft::world::biome::BiomeProviderKind::BiomeProviderKind> {
        self.provider.getBiomeProvider()
    }

    pub fn provideChunkSnapshot(&mut self, x: i32, z: i32) -> Result<Chunk, String> {
        let mut provider = self
            .chunkProvider
            .take()
            .ok_or_else(|| "WorldServer has no ChunkProviderServer".to_owned())?;
        let result = provider.provideChunk(self, x, z).map(|chunk| chunk.clone());
        self.chunkProvider = Some(provider);
        result
    }

    pub fn getBlockStateAt(&mut self, pos: BlockPos) -> Result<IBlockState, String> {
        if !(0..256).contains(&pos.y) {
            return Ok(IBlockState::fromGlobalStateId(0));
        }
        let mut provider = self
            .chunkProvider
            .take()
            .ok_or_else(|| "WorldServer has no ChunkProviderServer".to_owned())?;
        let result = provider
            .provideChunk(self, pos.x >> 4, pos.z >> 4)
            .map(|chunk| {
                chunk.getBlockState((pos.x & 15) as usize, pos.y as usize, (pos.z & 15) as usize)
            });
        self.chunkProvider = Some(provider);
        result
    }

    pub fn getGroundAboveSeaLevel(&mut self, pos: BlockPos) -> Result<IBlockState, String> {
        let mut y = self.seaLevel;
        while y < 255
            && !self
                .getBlockStateAt(BlockPos::new(pos.x, y + 1, pos.z))?
                .isAir()
        {
            y += 1;
        }
        self.getBlockStateAt(BlockPos::new(pos.x, y, pos.z))
    }

    /// Server-authoritative `World#setBlockState` substrate. The chunk is
    /// always resident/loaded through ChunkProviderServer, and the mutation
    /// marks it dirty for Anvil autosave/forced shutdown save. Rebuilding the
    /// height/skylight map is behavior-correct at this port boundary; the exact
    /// incremental relight path remains with the common World/lighting tranche.
    pub fn setBlockStateAt(
        &mut self,
        pos: BlockPos,
        state: IBlockState,
    ) -> Result<IBlockState, String> {
        if !(0..256).contains(&pos.y) {
            return Err("block y outside world build height".to_owned());
        }
        let mut provider = self
            .chunkProvider
            .take()
            .ok_or_else(|| "WorldServer has no ChunkProviderServer".to_owned())?;
        let result = (|| {
            let chunk = provider.provideChunk(self, pos.x >> 4, pos.z >> 4)?;
            let old = chunk.setBlockState(
                (pos.x & 15) as usize,
                pos.y as usize,
                (pos.z & 15) as usize,
                state,
                self.provider.hasSkyLight(),
            )?;
            if old != state {
                // MCP Chunk#setBlockState invalidates a previous TileEntity when
                // the block type changes. Concrete TileEntity creation for newly
                // placed TE blocks remains in the TileEntity/onBlockPlacedBy tranche.
                if old.getBlockId() != state.getBlockId() {
                    chunk.removeTileEntityData(&pos);
                }
                chunk.generateSkylightMap(self.provider.hasSkyLight());
            }
            Ok(old)
        })();
        self.chunkProvider = Some(provider);
        result
    }

    pub fn isBlockReplaceableAt(&self, pos: BlockPos) -> bool {
        isReplaceableState(self.getBlockState(pos))
    }

    /// Current server-side subset of `World#func_190527_a`: build-height,
    /// replaceability and the placed block's collision against the acting
    /// player. WorldBorder/entity-list collision expansion remains with the
    /// full common World runtime rather than being guessed.
    pub fn mayPlaceStateForPlayer(
        &self,
        state: IBlockState,
        pos: BlockPos,
        playerBox: crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB,
    ) -> bool {
        if !(0..256).contains(&pos.y) || !self.isBlockReplaceableAt(pos) {
            return false;
        }
        let block = state.getBlock();
        !block
            .getCollisionBoxes(state)
            .into_iter()
            .map(|bb| bb.offset(pos.x as f64, pos.y as f64, pos.z as f64))
            .any(|bb| bb.intersects(playerBox))
    }

    /// MCP `World#getTopSolidOrLiquidBlock`. The returned position is the air
    /// block immediately above the highest movement-blocking non-leaf block.
    pub fn getTopSolidOrLiquidBlock(&mut self, pos: BlockPos) -> Result<BlockPos, String> {
        let chunk = self.provideChunkSnapshot(pos.x >> 4, pos.z >> 4)?;
        let mut y = chunk.getTopFilledSegment().wrapping_add(16);
        while y >= 0 {
            let below = y - 1;
            if below < 0 {
                y = below;
                break;
            }
            let state =
                chunk.getBlockState((pos.x & 15) as usize, below as usize, (pos.z & 15) as usize);
            let block = state.getBlock();
            if block.materialBlocksMovement() && !matches!(Block::getIdFromBlock(block), 18 | 161) {
                break;
            }
            y = below;
        }
        Ok(BlockPos::new(pos.x, y, pos.z))
    }

    fn canCoordinateBeSpawn(&mut self, x: i32, z: i32) -> Result<bool, String> {
        let biome = self
            .provider
            .getBiomeProvider()
            .map(|provider| provider.getBiome(BlockPos::new(x, 0, z)));
        if biome.is_some_and(|biome| biome.ignorePlayerSpawnSuitability()) {
            return Ok(true);
        }
        Ok(self
            .getGroundAboveSeaLevel(BlockPos::new(x, 0, z))?
            .getBlockId()
            == 2)
    }

    /// MCP `WorldServer#initialize` / `createSpawnPosition` at the currently
    /// available biome/generator boundary. Bonus-chest generation remains with
    /// its unported WorldGeneratorBonusChest rather than being faked.
    pub fn initialize(&mut self, settings: &WorldSettings) -> Result<(), String> {
        if self.worldInfo.isInitialized() {
            return Ok(());
        }
        if !self.provider.canRespawnHere() {
            let y = self.provider.getAverageGroundLevel(self.seaLevel);
            self.worldInfo.setSpawnX(0);
            self.worldInfo.setSpawnY(y);
            self.worldInfo.setSpawnZ(0);
        } else if self.worldInfo.getTerrainType() == WorldType::DebugWorld {
            self.worldInfo.setSpawnX(0);
            self.worldInfo.setSpawnY(1);
            self.worldInfo.setSpawnZ(0);
        } else {
            self.findingSpawnPoint = true;
            let biomeProvider = self.provider.getBiomeProvider().cloned()
                .ok_or_else(|| "BiomeProvider/GenLayer chain is required before this world type can initialize spawn".to_owned())?;
            let allowed = biomeProvider.getBiomesToSpawnIn();
            let mut random = JavaRandom::new(self.getSeed());
            let found = biomeProvider.findBiomePosition(0, 0, 256, &allowed, &mut random);
            let mut x = found.map(|p| p.x).unwrap_or(8);
            let mut z = found.map(|p| p.z).unwrap_or(8);
            if found.is_none() {
                log::warn!("Unable to find spawn biome");
            }
            let mut attempts = 0;
            while !self.canCoordinateBeSpawn(x, z)? {
                x = x
                    .wrapping_add(random.next_i32_bound(64))
                    .wrapping_sub(random.next_i32_bound(64));
                z = z
                    .wrapping_add(random.next_i32_bound(64))
                    .wrapping_sub(random.next_i32_bound(64));
                attempts += 1;
                if attempts == 1000 {
                    break;
                }
            }
            self.worldInfo.setSpawnX(x);
            self.worldInfo
                .setSpawnY(self.provider.getAverageGroundLevel(self.seaLevel));
            self.worldInfo.setSpawnZ(z);
            self.findingSpawnPoint = false;
            if settings.isBonusChestEnabled() {
                log::warn!("WorldGeneratorBonusChest is not yet ported; bonus chest remains pending rather than fabricated");
            }
        }
        self.worldInfo.setServerInitialized(true);
        self.saveLevelData().map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn saveAllChunks(&mut self, force: bool) -> Result<bool, String> {
        let mut provider = self
            .chunkProvider
            .take()
            .ok_or_else(|| "WorldServer has no ChunkProviderServer".to_owned())?;
        let result = provider.saveChunks(self, force);
        self.chunkProvider = Some(provider);
        self.saveLevelData().map_err(|error| error.to_string())?;
        Ok(result)
    }

    pub fn chunkProvider(&self) -> Option<&ChunkProviderServer> {
        self.chunkProvider.as_ref()
    }
    pub fn isChunkLoaded(&self, x: i32, z: i32) -> bool {
        self.chunkProvider
            .as_ref()
            .is_some_and(|provider| provider.isChunkLoaded(x, z))
    }

    pub const fn getTotalWorldTime(&self) -> i64 {
        self.worldInfo.getWorldTotalTime()
    }
    pub const fn getWorldTime(&self) -> i64 {
        self.worldInfo.getWorldTime()
    }
    pub fn setTotalWorldTime(&mut self, time: i64) {
        self.worldInfo.setWorldTotalTime(time);
    }
    pub fn setWorldTime(&mut self, time: i64) {
        self.worldInfo.setWorldTime(time);
    }
    pub fn saveHandler(&self) -> &AnvilSaveHandler {
        &self.saveHandler
    }

    /// MCP `World#isSpawnChunk`, used by `WorldProviderSurface#canDropChunk`.
    pub fn isSpawnChunk(&self, x: i32, z: i32) -> bool {
        // Java int arithmetic wraps before the range comparison.
        let dx = x
            .wrapping_mul(16)
            .wrapping_add(8)
            .wrapping_sub(self.worldInfo.getSpawnX());
        let dz = z
            .wrapping_mul(16)
            .wrapping_add(8)
            .wrapping_sub(self.worldInfo.getSpawnZ());
        (-128..=128).contains(&dx) && (-128..=128).contains(&dz)
    }

    /// Vanilla provider dispatch for the only behaviour needed by
    /// `ChunkProviderServer#unload`: surface keeps spawn chunks, Nether/End use
    /// base `WorldProvider#canDropChunk` and allow every chunk to drop.
    pub fn canDropChunk(&self, x: i32, z: i32) -> bool {
        self.provider.canDropChunk(self.isSpawnChunk(x, z))
    }

    /// MCP `World#checkSessionLock` delegation.
    pub fn checkSessionLock(&self) -> io::Result<()> {
        self.saveHandler.base().checkSessionLock()
    }

    /// Persistence-visible part of MCP `WorldServer#saveLevel`.
    /// WorldBorder/MapStorage/scoreboard data are not silently invented here;
    /// the already-ported WorldInfo payload is rotated through AnvilSaveHandler.
    pub fn saveLevelData(&mut self) -> io::Result<()> {
        self.checkSessionLock()?;
        self.saveHandler.saveWorldInfo(&mut self.worldInfo)
    }

    /// MCP `WorldServer#isBlockTickPending`.
    pub fn isBlockTickPending(&self, pos: BlockPos, blockType: Block) -> bool {
        self.pendingTickListEntriesThisTick
            .contains(&NextTickListEntry::new(pos, blockType))
    }

    /// MCP `WorldServer#isUpdateScheduled`.
    pub fn isUpdateScheduled(&self, pos: BlockPos, block: Block) -> bool {
        self.pendingTickListEntriesHashSet
            .contains(&NextTickListEntry::new(pos, block))
    }

    /// MCP `WorldServer#scheduleBlockUpdate`.
    pub fn scheduleBlockUpdate(
        &mut self,
        pos: BlockPos,
        blockIn: Block,
        delay: i32,
        priority: i32,
    ) {
        let mut entry = NextTickListEntry::new(pos, blockIn);
        entry.setPriority(priority);
        if !blockIn.isAir() {
            entry.scheduledTime = self
                .worldInfo
                .getWorldTotalTime()
                .wrapping_add(delay as i64);
        }
        self.insertPendingTick(entry);
    }

    /// Restore an Anvil `TileTicks` entry whose scheduledTime has already been
    /// converted from the source relative `t` value to absolute world time.
    pub fn restoreScheduledTick(&mut self, entry: NextTickListEntry) {
        self.insertPendingTick(entry);
    }

    fn insertPendingTick(&mut self, entry: NextTickListEntry) {
        if self.pendingTickListEntriesHashSet.insert(entry.clone()) {
            self.pendingTickListEntriesTreeSet.insert(entry);
        }
    }

    /// World-owned half of `AnvilChunkLoader#readChunkFromNBT`: after the
    /// loader constructs a Chunk, source `TileTicks` are scheduled into
    /// WorldServer rather than being owned by the Chunk.
    pub fn acceptLoadedChunk(&mut self, loaded: LoadedChunk) -> Chunk {
        for entry in loaded.scheduledTicks {
            self.restoreScheduledTick(entry);
        }
        loaded.chunk
    }

    pub fn loadChunkFromLoader(
        &mut self,
        loader: &AnvilChunkLoader,
        x: i32,
        z: i32,
    ) -> io::Result<Option<Chunk>> {
        let currentWorldTime = self.worldInfo.getWorldTotalTime();
        let hasSkyLight = self.provider.hasSkyLight();
        Ok(loader
            .loadChunk(x, z, hasSkyLight, currentWorldTime)?
            .map(|loaded| self.acceptLoadedChunk(loaded)))
    }

    /// MCP `WorldServer#getPendingBlockUpdates(Chunk, boolean)`. The source
    /// expands the chunk X/Z bounds by two blocks so scheduled neighbour work
    /// follows a chunk through unload/save exactly as vanilla does.
    pub fn getPendingBlockUpdates(
        &mut self,
        chunkIn: &Chunk,
        remove: bool,
    ) -> Vec<NextTickListEntry> {
        let minX = (chunkIn.xPosition << 4) - 2;
        // MCP: `j = i + 16 + 2` where `i = chunkStart - 2`, and
        // StructureBoundingBox comparisons use `< maxX` / `< maxZ`. The
        // resulting persisted-tick window is therefore [start-2, start+15],
        // not a symmetric two-block expansion on the positive side.
        let maxX = minX + 18;
        let minZ = (chunkIn.zPosition << 4) - 2;
        let maxZ = minZ + 18;
        self.getPendingBlockUpdatesInBounds(minX, maxX, minZ, maxZ, remove)
    }

    fn getPendingBlockUpdatesInBounds(
        &mut self,
        minX: i32,
        maxX: i32,
        minZ: i32,
        maxZ: i32,
        remove: bool,
    ) -> Vec<NextTickListEntry> {
        let inBounds = |entry: &NextTickListEntry| {
            let pos = entry.position;
            pos.x >= minX && pos.x < maxX && pos.z >= minZ && pos.z < maxZ
        };

        let fromTree: Vec<_> = self
            .pendingTickListEntriesTreeSet
            .iter()
            .filter(|entry| inBounds(entry))
            .cloned()
            .collect();
        let fromThisTick: Vec<_> = self
            .pendingTickListEntriesThisTick
            .iter()
            .filter(|entry| inBounds(entry))
            .cloned()
            .collect();

        if remove {
            for entry in &fromTree {
                self.pendingTickListEntriesTreeSet.remove(entry);
                self.pendingTickListEntriesHashSet.remove(entry);
            }
            self.pendingTickListEntriesThisTick
                .retain(|entry| !inBounds(entry));
        }

        let mut out = Vec::with_capacity(fromTree.len() + fromThisTick.len());
        out.extend(fromTree);
        out.extend(fromThisTick);
        out
    }

    /// Exact cleaning phase of MCP `WorldServer#tickUpdates`: verify the dual
    /// collection invariant, cap at 65,536 entries and move due work into
    /// `pendingTickListEntriesThisTick`. The actual `Block#updateTick` phase is
    /// intentionally not faked until the server World block-access/runtime is
    /// present.
    pub fn beginPendingTickExecution(&mut self, force: bool) -> io::Result<&[NextTickListEntry]> {
        if self.worldInfo.getTerrainType() == WorldType::DebugWorld {
            return Ok(&self.pendingTickListEntriesThisTick);
        }
        if self.pendingTickListEntriesTreeSet.len() != self.pendingTickListEntriesHashSet.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "TickNextTick list out of synch",
            ));
        }
        let count = self.pendingTickListEntriesTreeSet.len().min(65_536);
        let now = self.worldInfo.getWorldTotalTime();
        let mut due = Vec::new();
        for entry in self.pendingTickListEntriesTreeSet.iter().take(count) {
            if !force && entry.scheduledTime > now {
                break;
            }
            due.push(entry.clone());
        }
        for entry in due {
            self.pendingTickListEntriesTreeSet.remove(&entry);
            self.pendingTickListEntriesHashSet.remove(&entry);
            self.pendingTickListEntriesThisTick.push(entry);
        }
        Ok(&self.pendingTickListEntriesThisTick)
    }

    /// Called only after the future block execution phase has consumed the
    /// `pendingTickListEntriesThisTick` list, matching the source clear at the
    /// end of `tickUpdates`.
    pub fn finishPendingTickExecution(&mut self) {
        self.pendingTickListEntriesThisTick.clear();
    }

    pub fn pendingTickCount(&self) -> usize {
        self.pendingTickListEntriesTreeSet.len()
    }
    pub fn pendingThisTickCount(&self) -> usize {
        self.pendingTickListEntriesThisTick.len()
    }
}

impl IBlockAccess for WorldServer {
    fn getBlockState(&self, pos: BlockPos) -> IBlockState {
        if !(0..256).contains(&pos.y) {
            return IBlockState::default();
        }
        self.chunkProvider
            .as_ref()
            .and_then(|provider| provider.getLoadedChunkRef(pos.x >> 4, pos.z >> 4))
            .map(|chunk| {
                chunk.getBlockState((pos.x & 15) as usize, pos.y as usize, (pos.z & 15) as usize)
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::world::GameType::GameType;
    use crate::net::minecraft::world::WorldSettings::WorldSettings;

    fn test_world(name: &str) -> (std::path::PathBuf, WorldServer) {
        let root =
            std::env::temp_dir().join(format!("mc1122-world-server-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let settings = WorldSettings::new(7, GameType::Survival, true, false, WorldType::Default);
        let info = WorldInfo::new(&settings, "World");
        let handler = AnvilSaveHandler::new(&root, "World", false).unwrap();
        (root, WorldServer::new(handler, info, 0))
    }

    #[test]
    fn schedule_block_update_deduplicates_by_position_and_block() {
        let (root, mut world) = test_world("ticks");
        let pos = BlockPos::new(1, 64, 2);
        let stone = Block::getBlockById(1);
        world.setTotalWorldTime(100);
        world.scheduleBlockUpdate(pos, stone, 20, 2);
        world.scheduleBlockUpdate(pos, stone, 40, 1);
        assert_eq!(world.pendingTickCount(), 1);
        assert!(world.isUpdateScheduled(pos, stone));
        let due = world.beginPendingTickExecution(false).unwrap();
        assert!(due.is_empty());
        world.setTotalWorldTime(120);
        let due = world.beginPendingTickExecution(false).unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].scheduledTime, 120);
        assert_eq!(due[0].priority, 2);
        world.finishPendingTickExecution();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn pending_chunk_ticks_use_vanilla_two_block_margin_and_can_remove() {
        let (root, mut world) = test_world("chunk-margin");
        let stone = Block::getBlockById(1);
        world.scheduleBlockUpdate(BlockPos::new(-2, 70, 15), stone, 1, 0);
        world.scheduleBlockUpdate(BlockPos::new(15, 70, 0), stone, 1, 0);
        world.scheduleBlockUpdate(BlockPos::new(16, 70, 0), stone, 1, 0);
        let chunk = Chunk::new(0, 0);
        let found = world.getPendingBlockUpdates(&chunk, true);
        assert_eq!(found.len(), 2);
        assert_eq!(world.pendingTickCount(), 1);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn authoritative_block_mutation_survives_forced_anvil_flush_and_reload() {
        let root = std::env::temp_dir().join(format!(
            "mc1122-world-server-save-reload-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let settings = WorldSettings::new(12345, GameType::Creative, true, false, WorldType::Flat);
        let handler = AnvilSaveHandler::new(&root, "World", false).unwrap();
        let info = WorldInfo::new(&settings, "World");
        let mut world = WorldServer::new(handler, info, 0).init().unwrap();
        world.initialize(&settings).unwrap();
        let pos = BlockPos::new(3, 4, 3);
        let original = world.getBlockStateAt(pos).unwrap();
        assert!(original.isAir());
        let diamond = IBlockState::fromGlobalStateId(57 << 4);
        world.setBlockStateAt(pos, diamond).unwrap();
        assert_eq!(world.getBlockStateAt(pos).unwrap(), diamond);
        assert!(world.saveAllChunks(true).unwrap());
        world.saveHandler().flush();
        drop(world);

        let handler = AnvilSaveHandler::new(&root, "World", false).unwrap();
        let info = handler.loadWorldInfo().unwrap().unwrap();
        let mut reloaded = WorldServer::new(handler, info, 0).init().unwrap();
        assert_eq!(reloaded.getBlockStateAt(pos).unwrap(), diamond);
        let _ = std::fs::remove_dir_all(root);
    }
}
