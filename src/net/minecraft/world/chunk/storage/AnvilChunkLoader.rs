use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::net::minecraft::block::Block::Block;
use crate::net::minecraft::nbt::CompressedStreamTools;
use crate::net::minecraft::nbt::NBTBase::{NBTBase, TAG_BYTE_ARRAY, TAG_COMPOUND, TAG_LIST, TAG_STRING};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::nbt::NBTTagList::NBTTagList;
use crate::net::minecraft::util::datafix::DataFixer::DataFixer;
use crate::net::minecraft::util::datafix::DataFixesManager::DataFixesManager;
use crate::net::minecraft::util::datafix::FixTypes::FixTypes;
use crate::net::minecraft::util::datafix::IDataFixer::IDataFixer;
use crate::net::minecraft::util::datafix::IDataWalker::IDataWalker;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::math::ChunkPos::ChunkPos;
use crate::net::minecraft::world::NextTickListEntry::NextTickListEntry;
use crate::net::minecraft::world::chunk::Chunk::Chunk;
use crate::net::minecraft::world::chunk::NibbleArray::NibbleArray;
use crate::net::minecraft::world::chunk::storage::ExtendedBlockStorage::ExtendedBlockStorage;
use crate::net::minecraft::world::chunk::storage::IChunkLoader::IChunkLoader;
use crate::net::minecraft::world::chunk::storage::RegionFileCache;
use crate::net::minecraft::world::storage::IThreadedFileIO::IThreadedFileIO;
use crate::net::minecraft::world::storage::SaveHandler::SaveHandler;
use crate::net::minecraft::world::storage::ThreadedFileIOBase::ThreadedFileIOBase;

#[derive(Debug, Default)]
struct PendingChunkIO {
    /// MCP `chunksToRemove`: latest queued root NBT per chunk.
    chunksToRemove: HashMap<ChunkPos, NBTTagCompound>,
    /// MCP `field_193415_c`: chunk positions currently being written.
    chunksBeingWritten: HashSet<ChunkPos>,
}

#[derive(Debug)]
struct AnvilChunkLoaderInner {
    chunkSaveLocation: PathBuf,
    dataFixer: Arc<DataFixer>,
    pending: Mutex<PendingChunkIO>,
    savingExtraData: AtomicBool,
}

/// Result of the source `readChunkFromNBT` boundary before a complete Rust
/// `WorldServer` exists. `Chunk` owns block/entity/tile data; scheduled ticks
/// remain world-owned and are returned separately for `World#scheduleBlockUpdate`.
#[derive(Debug, Clone)]
pub struct LoadedChunk {
    pub chunk: Chunk,
    pub scheduledTicks: Vec<NextTickListEntry>,
}

/// MCP 1.12.2 `AnvilChunkLoader`.
///
/// Unlike Batch 122's synchronous storage helper, this version owns the source
/// pending-write map, in-flight set, DataFixer boundary and global threaded-I/O
/// queue identity. Entity/TileEntity data is not replaced by client stand-ins:
/// Chunk currently owns authoritative NBT snapshots until the concrete server
/// registries are ported, so asynchronous saves are lossless.
#[derive(Debug, Clone)]
pub struct AnvilChunkLoader {
    inner: Arc<AnvilChunkLoaderInner>,
}

impl AnvilChunkLoader {
    pub fn new(chunkSaveLocationIn: impl AsRef<Path>) -> io::Result<Self> {
        // MCP `DataFixesManager#createFixer` is the composition root. Keeping
        // the loader on that shared registration path prevents Chunk loads
        // from silently skipping legacy entity/tile migrations.
        Self::newWithDataFixer(chunkSaveLocationIn, Arc::new(DataFixesManager::createFixer()))
    }

    pub fn newWithDataFixer(
        chunkSaveLocationIn: impl AsRef<Path>,
        dataFixer: Arc<DataFixer>,
    ) -> io::Result<Self> {
        let chunkSaveLocation = chunkSaveLocationIn.as_ref().to_path_buf();
        std::fs::create_dir_all(&chunkSaveLocation)?;
        Ok(Self {
            inner: Arc::new(AnvilChunkLoaderInner {
                chunkSaveLocation,
                dataFixer,
                pending: Mutex::new(PendingChunkIO::default()),
                savingExtraData: AtomicBool::new(false),
            }),
        })
    }

    /// MCP `registerFixes`: CHUNK walks nested entity and block-entity lists.
    pub fn registerFixes(fixer: &mut DataFixer) {
        fixer.registerWalker(FixTypes::Chunk, Arc::new(ChunkDataWalker));
    }

    /// MCP `func_191063_a`: queued data counts as generated before disk flush.
    pub fn isChunkGeneratedAt(&self, x: i32, z: i32) -> io::Result<bool> {
        let pos = ChunkPos::new(x, z);
        if self.pending().chunksToRemove.contains_key(&pos) { return Ok(true); }
        RegionFileCache::isChunkSaved(&self.inner.chunkSaveLocation, x, z)
    }

    /// Source `loadChunk` NBT acquisition: pending data wins over the region
    /// file, then the CHUNK DataFixer is applied.
    pub fn loadChunkNBT(&self, x: i32, z: i32) -> io::Result<Option<NBTTagCompound>> {
        let pos = ChunkPos::new(x, z);
        let pending = self.pending().chunksToRemove.get(&pos).cloned();
        let root = if let Some(root) = pending {
            root
        } else {
            let Some(bytes) = RegionFileCache::readChunkData(&self.inner.chunkSaveLocation, x, z)? else {
                return Ok(None);
            };
            let mut input = bytes.as_slice();
            CompressedStreamTools::readRoot(&mut input)?
        };
        Ok(Some(self.inner.dataFixer.process(FixTypes::Chunk, root)))
    }

    /// Low-level synchronous region write retained for RegionFile/NBT tests.
    /// Normal world saves use `saveChunk` -> `addChunkToPending`.
    pub fn saveChunkNBT(&self, x: i32, z: i32, root: &NBTTagCompound) -> io::Result<()> {
        self.writeChunkData(ChunkPos::new(x, z), root)
    }

    /// MCP `checkedReadChunkFromNBT`: validate Level/Sections and relocate a
    /// chunk whose stored xPos/zPos disagree with the requested region slot.
    pub fn checkedReadChunkFromNBT(
        &self,
        x: i32,
        z: i32,
        mut root: NBTTagCompound,
        hasSkyLight: bool,
        currentWorldTime: i64,
    ) -> io::Result<LoadedChunk> {
        if !root.hasKeyWithType("Level", TAG_COMPOUND) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Chunk file at {x},{z} is missing level data, skipping")));
        }
        let mut level = root.getCompoundTag("Level");
        if !level.hasKeyWithType("Sections", TAG_LIST) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Chunk file at {x},{z} is missing block data, skipping")));
        }
        let mut loaded = Self::readChunkFromNBT(&level, hasSkyLight, currentWorldTime)?;
        if !loaded.chunk.isAtLocation(x, z) {
            log::error!(
                "Chunk file at {},{} is in the wrong location; relocating. (Expected {}, {}, got {}, {})",
                x, z, x, z, loaded.chunk.xPosition, loaded.chunk.zPosition
            );
            level.setInteger("xPos", x);
            level.setInteger("zPos", z);
            root.setCompoundTag("Level", level.clone());
            loaded = Self::readChunkFromNBT(&level, hasSkyLight, currentWorldTime)?;
        }
        Ok(loaded)
    }

    /// Source-shaped `loadChunk`. The complete World parameter is split into
    /// only the two values this persistence layer actually reads at this stage:
    /// provider skylight and total world time. Scheduled ticks are returned for
    /// the future WorldServer owner rather than executed in the loader.
    pub fn loadChunk(
        &self,
        x: i32,
        z: i32,
        hasSkyLight: bool,
        currentWorldTime: i64,
    ) -> io::Result<Option<LoadedChunk>> {
        let Some(root) = self.loadChunkNBT(x, z)? else { return Ok(None); };
        self.checkedReadChunkFromNBT(x, z, root, hasSkyLight, currentWorldTime).map(Some)
    }

    /// MCP `saveChunk` storage contract. `World#checkSessionLock` is supplied
    /// by the already-ported SaveHandler until World owns that handler directly.
    /// The generated root is always current 1.12.2 DataVersion 1343 and is
    /// queued, not synchronously written.
    pub fn saveChunk(
        &self,
        saveHandler: &SaveHandler,
        chunk: &mut Chunk,
        totalWorldTime: i64,
        hasSkyLight: bool,
        pendingTicks: Option<&[NextTickListEntry]>,
    ) -> io::Result<()> {
        saveHandler.checkSessionLock()?;
        let mut root = NBTTagCompound::new();
        let mut level = NBTTagCompound::new();
        root.setInteger("DataVersion", 1343);
        Self::writeChunkToNBT(chunk, totalWorldTime, hasSkyLight, pendingTicks, &mut level);
        root.setCompoundTag("Level", level);
        self.addChunkToPending(ChunkPos::new(chunk.xPosition, chunk.zPosition), root);
        Ok(())
    }

    /// MCP `addChunkToPending`. A save arriving while the same position is in
    /// `field_193415_c` is intentionally not inserted, exactly matching 1.12.2.
    pub fn addChunkToPending(&self, pos: ChunkPos, compound: NBTTagCompound) {
        {
            let mut pending = self.pending();
            if !pending.chunksBeingWritten.contains(&pos) {
                pending.chunksToRemove.insert(pos, compound);
            }
        }
        let io: Arc<dyn IThreadedFileIO> = Arc::new(self.clone());
        ThreadedFileIOBase::getThreadedIOInstance().queueIO(io);
    }

    /// Public source method used by `ThreadedFileIOBase` and `saveExtraData`.
    pub fn writeNextIO(&self) -> bool { self.writeNextIOImpl() }

    fn writeNextIOImpl(&self) -> bool {
        let (pos, compound) = {
            let mut pending = self.pending();
            if pending.chunksToRemove.is_empty() {
                if self.inner.savingExtraData.load(Ordering::Acquire) {
                    let name = self.inner.chunkSaveLocation.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    log::info!("ThreadedAnvilChunkStorage ({}): All chunks are saved", name);
                }
                return false;
            }
            let pos = *pending.chunksToRemove.keys().next().expect("non-empty pending chunk map");
            pending.chunksBeingWritten.insert(pos);
            let compound = pending.chunksToRemove.remove(&pos);
            (pos, compound)
        };

        if let Some(compound) = compound {
            if let Err(error) = self.writeChunkData(pos, &compound) {
                log::error!("Failed to save chunk: {}", error);
            }
        }
        self.pending().chunksBeingWritten.remove(&pos);
        true
    }

    fn writeChunkData(&self, pos: ChunkPos, compound: &NBTTagCompound) -> io::Result<()> {
        let mut bytes = Vec::new();
        CompressedStreamTools::writeRoot(compound, &mut bytes)?;
        RegionFileCache::writeChunkData(
            &self.inner.chunkSaveLocation,
            pos.chunkXPos,
            pos.chunkZPos,
            &bytes,
        )
    }

    /// MCP methods are deliberately empty in 1.12.2.
    pub fn saveExtraChunkData(&self, _chunk: &Chunk) -> io::Result<()> { Ok(()) }
    pub fn chunkTick(&self) {}

    /// MCP `saveExtraData`: synchronously drain all pending chunks while
    /// setting the logging flag used by `writeNextIO`.
    pub fn saveExtraData(&self) {
        self.inner.savingExtraData.store(true, Ordering::Release);
        while self.writeNextIOImpl() {}
        self.inner.savingExtraData.store(false, Ordering::Release);
    }

    pub fn pendingChunkCount(&self) -> usize { self.pending().chunksToRemove.len() }
    pub fn writingChunkCount(&self) -> usize { self.pending().chunksBeingWritten.len() }

    /// MCP `readChunkFromNBT` block/section half.
    pub fn readChunkCoreFromNBT(level: &NBTTagCompound, hasSkyLight: bool) -> io::Result<Chunk> {
        let x = level.getInteger("xPos");
        let z = level.getInteger("zPos");
        let mut chunk = Chunk::new(x, z);

        let heightMap = level.getIntArray("HeightMap");
        chunk.setHeightMap(&heightMap);
        chunk.setTerrainPopulated(level.getBoolean("TerrainPopulated"));
        chunk.setLightPopulated(level.getBoolean("LightPopulated"));
        chunk.setInhabitedTime(level.getLong("InhabitedTime"));

        let sections = level.getTagList("Sections", TAG_COMPOUND);
        let mut storageArrays: Vec<Option<ExtendedBlockStorage>> = vec![None; 16];
        for sectionIndex in 0..sections.tagCount() {
            let sectionTag = sections.getCompoundTagAt(sectionIndex);
            let yIndex = sectionTag.getByte("Y") as i32;
            if !(0..16).contains(&yIndex) {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Anvil section Y out of range: {yIndex}")));
            }
            let blocks = sectionTag.getByteArray("Blocks");
            if blocks.len() != 4096 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Anvil Blocks should be 4096 bytes not: {}", blocks.len())));
            }
            let data = NibbleArray::fromStorage(sectionTag.getByteArray("Data")).map_err(invalid_data)?;
            let add = if sectionTag.hasKeyWithType("Add", TAG_BYTE_ARRAY) {
                Some(NibbleArray::fromStorage(sectionTag.getByteArray("Add")).map_err(invalid_data)?)
            } else { None };

            let mut storage = ExtendedBlockStorage::new(yIndex << 4, hasSkyLight);
            storage.getDataMut().setDataFromNBT(&blocks, &data, add.as_ref()).map_err(invalid_data)?;
            storage.setBlocklightArray(NibbleArray::fromStorage(sectionTag.getByteArray("BlockLight")).map_err(invalid_data)?);
            if hasSkyLight {
                storage.setSkylightArray(NibbleArray::fromStorage(sectionTag.getByteArray("SkyLight")).map_err(invalid_data)?);
            }
            storage.removeInvalidBlocks();
            storageArrays[yIndex as usize] = Some(storage);
        }
        chunk.setStorageArrays(storageArrays);
        if level.hasKeyWithType("Biomes", TAG_BYTE_ARRAY) {
            chunk.setBiomeArray(&level.getByteArray("Biomes"));
        }
        chunk.setModified(false);
        Ok(chunk)
    }

    /// MCP `writeChunkToNBT` block/section half.
    pub fn writeChunkCoreToNBT(
        chunk: &Chunk,
        totalWorldTime: i64,
        hasSkyLight: bool,
        level: &mut NBTTagCompound,
    ) {
        level.setInteger("xPos", chunk.xPosition);
        level.setInteger("zPos", chunk.zPosition);
        level.setLong("LastUpdate", totalWorldTime);
        level.setIntArray("HeightMap", chunk.getHeightMap().to_vec());
        level.setBoolean("TerrainPopulated", chunk.isTerrainPopulated());
        level.setBoolean("LightPopulated", chunk.isLightPopulated());
        level.setLong("InhabitedTime", chunk.getInhabitedTime());

        let mut sections = NBTTagList::new();
        for storage in chunk.getBlockStorageArray().iter().flatten() {
            let mut sectionTag = NBTTagCompound::new();
            sectionTag.setByte("Y", ((storage.getYLocation() >> 4) & 255) as i8);
            let mut blocks = [0_u8; 4096];
            let mut data = NibbleArray::new();
            let add = storage.getData().getDataForNBT(&mut blocks, &mut data);
            sectionTag.setByteArray("Blocks", blocks.to_vec());
            sectionTag.setByteArray("Data", data.getData().to_vec());
            if let Some(add) = add { sectionTag.setByteArray("Add", add.getData().to_vec()); }
            sectionTag.setByteArray("BlockLight", storage.getBlocklightArray().getData().to_vec());
            if hasSkyLight {
                let skylight = storage.getSkylightArray().cloned().unwrap_or_else(NibbleArray::new);
                sectionTag.setByteArray("SkyLight", skylight.getData().to_vec());
            } else {
                sectionTag.setByteArray("SkyLight", vec![0; 2048]);
            }
            sections.appendTag(NBTBase::Compound(sectionTag));
        }
        level.setTagList("Sections", sections);
        level.setByteArray("Biomes", chunk.getBiomeArray().to_vec());
    }

    /// Persistence-stage object layer of MCP `readChunkFromNBT`. The NBT is
    /// already CHUNK/ENTITY/BLOCK_ENTITY DataFixed by `loadChunkNBT`.
    pub fn readChunkObjectDataFromNBT(chunk: &mut Chunk, level: &NBTTagCompound) {
        chunk.clearEntityData();
        let entities = level.getTagList("Entities", TAG_COMPOUND);
        for index in 0..entities.tagCount() {
            chunk.addEntityData(entities.getCompoundTagAt(index));
        }
        chunk.setHasEntities(entities.tagCount() > 0);

        chunk.clearTileEntityData();
        let tileEntities = level.getTagList("TileEntities", TAG_COMPOUND);
        for index in 0..tileEntities.tagCount() {
            chunk.addTileEntityData(tileEntities.getCompoundTagAt(index));
        }
    }

    /// Persistence-stage object layer of MCP `writeChunkToNBT`. Source writes
    /// both lists even when empty and recomputes `hasEntities` while writing.
    pub fn writeChunkObjectDataToNBT(chunk: &mut Chunk, level: &mut NBTTagCompound) {
        let mut entities = NBTTagList::new();
        let mut hasEntities = false;
        for list in chunk.getEntityListsData() {
            for entity in list {
                entities.appendTag(NBTBase::Compound(entity.clone()));
                hasEntities = true;
            }
        }
        chunk.setHasEntities(hasEntities);
        level.setTagList("Entities", entities);

        let mut tileEntities = NBTTagList::new();
        for tileEntity in chunk.getTileEntityMapData().values() {
            tileEntities.appendTag(NBTBase::Compound(tileEntity.clone()));
        }
        level.setTagList("TileEntities", tileEntities);
    }

    /// MCP TileTicks decoder. `t` is relative to current total world time.
    pub fn readTileTicks(level: &NBTTagCompound, currentWorldTime: i64) -> io::Result<Vec<NextTickListEntry>> {
        if !level.hasKeyWithType("TileTicks", TAG_LIST) { return Ok(Vec::new()); }
        let tags = level.getTagList("TileTicks", TAG_COMPOUND);
        let mut result = Vec::with_capacity(tags.tagCount());
        for index in 0..tags.tagCount() {
            let tag = tags.getCompoundTagAt(index);
            let block = if tag.hasKeyWithType("i", TAG_STRING) {
                Block::getBlockFromName(&tag.getString("i")).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, format!("Unknown block in TileTicks: {}", tag.getString("i")))
                })?
            } else {
                Block::getBlockById(tag.getInteger("i"))
            };
            let mut entry = NextTickListEntry::new(
                BlockPos::new(tag.getInteger("x"), tag.getInteger("y"), tag.getInteger("z")),
                block,
            ).setScheduledTime(currentWorldTime.wrapping_add(tag.getInteger("t") as i64));
            entry.setPriority(tag.getInteger("p"));
            result.push(entry);
        }
        Ok(result)
    }

    /// Batch-122 helper semantics retained: an empty slice does not erase an
    /// opaque pre-existing TileTicks tag. Full `writeChunkToNBT` uses the exact
    /// Option-based source branch below.
    pub fn writeTileTicks(level: &mut NBTTagCompound, currentWorldTime: i64, entries: &[NextTickListEntry]) {
        if entries.is_empty() { return; }
        Self::writeTileTicksList(level, currentWorldTime, entries);
    }

    fn writeTileTicksList(level: &mut NBTTagCompound, currentWorldTime: i64, entries: &[NextTickListEntry]) {
        let mut tags = NBTTagList::new();
        for entry in entries {
            let mut tag = NBTTagCompound::new();
            tag.setString("i", entry.getBlock().getRegistryName().to_string());
            tag.setInteger("x", entry.position.x);
            tag.setInteger("y", entry.position.y);
            tag.setInteger("z", entry.position.z);
            tag.setInteger("t", entry.scheduledTime.wrapping_sub(currentWorldTime) as i32);
            tag.setInteger("p", entry.priority);
            tags.appendTag(NBTBase::Compound(tag));
        }
        level.setTagList("TileTicks", tags);
    }

    pub fn readChunkFromNBT(
        level: &NBTTagCompound,
        hasSkyLight: bool,
        currentWorldTime: i64,
    ) -> io::Result<LoadedChunk> {
        let mut chunk = Self::readChunkCoreFromNBT(level, hasSkyLight)?;
        Self::readChunkObjectDataFromNBT(&mut chunk, level);
        let scheduledTicks = Self::readTileTicks(level, currentWorldTime)?;
        Ok(LoadedChunk { chunk, scheduledTicks })
    }

    pub fn writeChunkToNBT(
        chunk: &mut Chunk,
        totalWorldTime: i64,
        hasSkyLight: bool,
        pendingTicks: Option<&[NextTickListEntry]>,
        level: &mut NBTTagCompound,
    ) {
        Self::writeChunkCoreToNBT(chunk, totalWorldTime, hasSkyLight, level);
        Self::writeChunkObjectDataToNBT(chunk, level);
        if let Some(entries) = pendingTicks {
            Self::writeTileTicksList(level, totalWorldTime, entries);
        }
    }

    /// Compatibility helper retained for code that only needs the complete
    /// static/object Chunk without scheduling ticks into a World yet.
    pub fn loadChunkCore(&self, x: i32, z: i32, hasSkyLight: bool) -> io::Result<Option<Chunk>> {
        Ok(self.loadChunk(x, z, hasSkyLight, 0)?.map(|loaded| loaded.chunk))
    }

    pub fn chunkSaveLocation(&self) -> &Path { &self.inner.chunkSaveLocation }
    pub fn dataFixer(&self) -> &DataFixer { self.inner.dataFixer.as_ref() }

    fn pending(&self) -> std::sync::MutexGuard<'_, PendingChunkIO> {
        self.inner.pending.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl IThreadedFileIO for AnvilChunkLoader {
    fn writeNextIO(&self) -> bool { self.writeNextIOImpl() }
    fn ioIdentity(&self) -> usize { Arc::as_ptr(&self.inner) as usize }
}

impl IChunkLoader for AnvilChunkLoader {
    fn loadChunk(&self, worldIn: &mut crate::net::minecraft::world::WorldServer::WorldServer, x: i32, z: i32) -> io::Result<Option<Chunk>> {
        let currentWorldTime = worldIn.getTotalWorldTime();
        let hasSkyLight = worldIn.provider.hasSkyLight();
        Ok(AnvilChunkLoader::loadChunk(self, x, z, hasSkyLight, currentWorldTime)?
            .map(|loaded| worldIn.acceptLoadedChunk(loaded)))
    }

    fn saveChunk(&self, worldIn: &mut crate::net::minecraft::world::WorldServer::WorldServer, chunkIn: &mut Chunk) -> io::Result<()> {
        let pending = worldIn.getPendingBlockUpdates(chunkIn, false);
        AnvilChunkLoader::saveChunk(
            self,
            worldIn.saveHandler().base(),
            chunkIn,
            worldIn.getTotalWorldTime(),
            worldIn.provider.hasSkyLight(),
            Some(pending.as_slice()),
        )
    }

    fn saveExtraChunkData(&self, _worldIn: &mut crate::net::minecraft::world::WorldServer::WorldServer, chunkIn: &Chunk) -> io::Result<()> {
        AnvilChunkLoader::saveExtraChunkData(self, chunkIn)
    }

    fn chunkTick(&self) { AnvilChunkLoader::chunkTick(self); }
    fn saveExtraData(&self) { AnvilChunkLoader::saveExtraData(self); }
    fn func_191063_a(&self, x: i32, z: i32) -> io::Result<bool> { AnvilChunkLoader::isChunkGeneratedAt(self, x, z) }
}

struct ChunkDataWalker;
impl IDataWalker for ChunkDataWalker {
    fn process(&self, fixer: &dyn IDataFixer, mut compound: NBTTagCompound, versionIn: i32) -> NBTTagCompound {
        if !compound.hasKeyWithType("Level", TAG_COMPOUND) { return compound; }
        let mut level = compound.getCompoundTag("Level");
        if level.hasKeyWithType("Entities", TAG_LIST) {
            let mut entities = level.getTagList("Entities", TAG_COMPOUND);
            for index in 0..entities.tagCount() {
                let fixed = fixer.processVersioned(FixTypes::Entity, entities.getCompoundTagAt(index), versionIn);
                entities.set(index, NBTBase::Compound(fixed));
            }
            level.setTagList("Entities", entities);
        }
        if level.hasKeyWithType("TileEntities", TAG_LIST) {
            let mut tileEntities = level.getTagList("TileEntities", TAG_COMPOUND);
            for index in 0..tileEntities.tagCount() {
                let fixed = fixer.processVersioned(FixTypes::BlockEntity, tileEntities.getCompoundTagAt(index), versionIn);
                tileEntities.set(index, NBTBase::Compound(fixed));
            }
            level.setTagList("TileEntities", tileEntities);
        }
        compound.setCompoundTag("Level", level);
        compound
    }
}

fn invalid_data(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::block::state::IBlockState::IBlockState;
    use crate::net::minecraft::nbt::NBTBase::{TAG_COMPOUND, TAG_DOUBLE};
    use crate::net::minecraft::world::GameType::GameType;
    use crate::net::minecraft::world::WorldSettings::WorldSettings;
    use crate::net::minecraft::world::WorldType::WorldType;

    fn temp_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("mc1122-{name}-{}", std::process::id()))
    }

    fn minimal_level(x: i32, z: i32) -> NBTTagCompound {
        let mut level = NBTTagCompound::new();
        level.setInteger("xPos", x);
        level.setInteger("zPos", z);
        level.setIntArray("HeightMap", vec![0; 256]);
        level.setTagList("Sections", NBTTagList::new());
        level.setByteArray("Biomes", vec![1; 256]);
        level
    }

    #[test]
    fn complete_chunk_nbt_roundtrips_through_anvil_region_storage() {
        RegionFileCache::clearRegionFileReferences();
        let rootDir = temp_root("anvil-chunk-loader");
        let _ = std::fs::remove_dir_all(&rootDir);
        let loader = AnvilChunkLoader::new(&rootDir).unwrap();
        let mut level = minimal_level(2, -3);
        level.setLong("InhabitedTime", 12345);
        let mut root = NBTTagCompound::new();
        root.setInteger("DataVersion", 1343);
        root.setCompoundTag("Level", level);
        assert!(!loader.isChunkGeneratedAt(2, -3).unwrap());
        loader.saveChunkNBT(2, -3, &root).unwrap();
        assert!(loader.isChunkGeneratedAt(2, -3).unwrap());
        let decoded = loader.loadChunkNBT(2, -3).unwrap().unwrap();
        assert_eq!(decoded.getInteger("DataVersion"), 1343);
        assert_eq!(decoded.getCompoundTag("Level").getLong("InhabitedTime"), 12345);
        RegionFileCache::clearRegionFileReferences();
        let _ = std::fs::remove_dir_all(rootDir);
    }

    #[test]
    fn chunk_core_and_object_layers_roundtrip_without_data_loss() {
        let mut chunk = Chunk::new(4, -7);
        let heights: Vec<_> = (0..256).map(|i| (i % 96) as i32).collect();
        chunk.setHeightMap(&heights);
        chunk.setTerrainPopulated(true);
        chunk.setLightPopulated(true);
        chunk.setInhabitedTime(987654321);
        chunk.setBiomeArray(&vec![4_u8; 256]);
        chunk.setBlockState(1, 34, 3, IBlockState::fromGlobalStateId(2 << 4), true).unwrap();
        chunk.setBlockState(2, 34, 3, IBlockState::fromGlobalStateId(1 << 4), true).unwrap();

        let mut entity = NBTTagCompound::new();
        entity.setString("id", "minecraft:pig");
        let mut pos = NBTTagList::new();
        pos.appendTag(NBTBase::Double(1.5));
        pos.appendTag(NBTBase::Double(34.0));
        pos.appendTag(NBTBase::Double(2.5));
        entity.setTagList("Pos", pos);
        chunk.addEntityData(entity.clone());

        let mut tile = NBTTagCompound::new();
        tile.setString("id", "minecraft:chest");
        tile.setInteger("x", 65); tile.setInteger("y", 35); tile.setInteger("z", -111);
        chunk.addTileEntityData(tile.clone());

        let mut level = NBTTagCompound::new();
        AnvilChunkLoader::writeChunkToNBT(&mut chunk, 1234, true, Some(&[]), &mut level);
        assert_eq!(level.getTagList("Entities", TAG_COMPOUND).getCompoundTagAt(0), entity);
        assert_eq!(level.getTagList("TileEntities", TAG_COMPOUND).getCompoundTagAt(0), tile);
        assert!(level.hasKeyWithType("TileTicks", TAG_LIST));

        let loaded = AnvilChunkLoader::readChunkFromNBT(&level, true, 1234).unwrap();
        assert_eq!(loaded.chunk.getHeightMap(), chunk.getHeightMap());
        assert!(loaded.chunk.hasEntities());
        assert_eq!(loaded.chunk.getEntityListsData()[2][0], entity);
        assert_eq!(loaded.chunk.getTileEntityMapData().get(&BlockPos::new(65, 35, -111)), Some(&tile));
        assert_eq!(loaded.chunk.getGlobalStateId(1, 34, 3), 2 << 4);
        assert_eq!(loaded.chunk.getGlobalStateId(2, 34, 3), 1 << 4);
        let section = loaded.chunk.getBlockStorageArray()[2].as_ref().unwrap();
        assert_eq!(section.getBlockRefCount(), 2);
        assert_eq!(section.getTickRefCount(), 1);
    }

    #[test]
    fn pending_chunk_shadows_older_region_data_until_threaded_flush() {
        RegionFileCache::clearRegionFileReferences();
        let rootDir = temp_root("anvil-pending-shadow");
        let _ = std::fs::remove_dir_all(&rootDir);
        let loader = AnvilChunkLoader::new(&rootDir).unwrap();

        let mut oldRoot = NBTTagCompound::new();
        oldRoot.setInteger("DataVersion", 1343);
        oldRoot.setCompoundTag("Level", minimal_level(8, 9));
        loader.saveChunkNBT(8, 9, &oldRoot).unwrap();

        let mut newRoot = oldRoot.clone();
        let mut level = newRoot.getCompoundTag("Level");
        level.setLong("InhabitedTime", 99);
        newRoot.setCompoundTag("Level", level);
        loader.addChunkToPending(ChunkPos::new(8, 9), newRoot);
        assert_eq!(loader.loadChunkNBT(8, 9).unwrap().unwrap().getCompoundTag("Level").getLong("InhabitedTime"), 99);
        loader.saveExtraData();
        ThreadedFileIOBase::getThreadedIOInstance().waitForFinish();
        assert_eq!(loader.loadChunkNBT(8, 9).unwrap().unwrap().getCompoundTag("Level").getLong("InhabitedTime"), 99);
        RegionFileCache::clearRegionFileReferences();
        let _ = std::fs::remove_dir_all(rootDir);
    }

    #[test]
    fn checked_read_relocates_wrong_chunk_coordinates() {
        let loader = AnvilChunkLoader::new(temp_root("anvil-relocate")).unwrap();
        let mut root = NBTTagCompound::new();
        root.setInteger("DataVersion", 1343);
        root.setCompoundTag("Level", minimal_level(1, 2));
        let loaded = loader.checkedReadChunkFromNBT(3, 4, root, true, 0).unwrap();
        assert!(loaded.chunk.isAtLocation(3, 4));
    }

    #[test]
    fn save_chunk_checks_session_lock_and_queues_current_1122_root() {
        RegionFileCache::clearRegionFileReferences();
        let saves = temp_root("anvil-save-session");
        let _ = std::fs::remove_dir_all(&saves);
        let handler = SaveHandler::new(&saves, "World", false).unwrap();
        let loader = AnvilChunkLoader::new(handler.getWorldDirectory()).unwrap();
        let mut chunk = Chunk::new(0, 0);
        chunk.setBiomeArray(&vec![1; 256]);
        loader.saveChunk(&handler, &mut chunk, 77, true, None).unwrap();
        assert!(loader.isChunkGeneratedAt(0, 0).unwrap());
        let queued = loader.loadChunkNBT(0, 0).unwrap().unwrap();
        assert_eq!(queued.getInteger("DataVersion"), 1343);
        assert_eq!(queued.getCompoundTag("Level").getLong("LastUpdate"), 77);
        loader.saveExtraData();
        ThreadedFileIOBase::getThreadedIOInstance().waitForFinish();
        RegionFileCache::clearRegionFileReferences();
        let _ = std::fs::remove_dir_all(saves);
    }

    #[test]
    fn nether_chunk_core_writes_zero_skylight_like_anvil_chunk_loader() {
        let mut chunk = Chunk::new(0, 0);
        chunk.setBlockState(0, 0, 0, IBlockState::fromGlobalStateId(1 << 4), false).unwrap();
        let mut level = NBTTagCompound::new();
        AnvilChunkLoader::writeChunkCoreToNBT(&chunk, 0, false, &mut level);
        let sections = level.getTagList("Sections", TAG_COMPOUND);
        assert_eq!(sections.getCompoundTagAt(0).getByteArray("SkyLight"), vec![0; 2048]);
        let loaded = AnvilChunkLoader::readChunkCoreFromNBT(&level, false).unwrap();
        assert!(loaded.getBlockStorageArray()[0].as_ref().unwrap().getSkylightArray().is_none());
    }

    #[test]
    fn tile_ticks_use_registry_names_relative_delay_and_priority() {
        let mut level = NBTTagCompound::new();
        let mut first = NextTickListEntry::new(BlockPos::new(3, 64, -5), Block::getBlockById(2)).setScheduledTime(1230);
        first.setPriority(2);
        AnvilChunkLoader::writeTileTicks(&mut level, 1200, &[first.clone()]);
        let tag = level.getTagList("TileTicks", TAG_COMPOUND).getCompoundTagAt(0);
        assert_eq!(tag.getString("i"), "minecraft:grass");
        assert_eq!(tag.getInteger("t"), 30);
        assert_eq!(tag.getInteger("p"), 2);
        let decoded = AnvilChunkLoader::readTileTicks(&level, 1200).unwrap();
        assert_eq!(decoded[0].position, first.position);
        assert_eq!(decoded[0].getBlock(), first.getBlock());
        assert_eq!(decoded[0].scheduledTime, 1230);
        assert_eq!(decoded[0].priority, 2);
    }

    #[test]
    fn nbt_numeric_lists_expose_source_double_access_for_entity_slicing() {
        let mut list = NBTTagList::new();
        list.appendTag(NBTBase::Double(1.0));
        list.appendTag(NBTBase::Double(32.5));
        assert_eq!(list.getTagType(), TAG_DOUBLE);
        assert_eq!(list.getDoubleAt(1), 32.5);
        assert_eq!(list.getDoubleAt(99), 0.0);
    }

    #[test]
    fn world_settings_import_is_not_required_by_loader_runtime() {
        // Keep the test module linked against existing world-creation types so
        // persistence changes cannot accidentally orphan that module graph.
        let settings = WorldSettings::new(1, GameType::Survival, true, false, WorldType::Default);
        assert_eq!(settings.getSeed(), 1);
    }
}
