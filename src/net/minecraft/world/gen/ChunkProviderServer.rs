use std::collections::{HashMap, HashSet};
use std::io;

use crate::net::minecraft::util::math::ChunkPos::ChunkPos;
use crate::net::minecraft::world::WorldServer::WorldServer;
use crate::net::minecraft::world::chunk::Chunk::Chunk;
use crate::net::minecraft::world::chunk::storage::AnvilChunkLoader::AnvilChunkLoader;
use crate::net::minecraft::world::gen::IChunkGenerator::IChunkGenerator;

/// MCP 1.12.2 `ChunkProviderServer` at the first concrete generator boundary.
///
/// The resident cache and Anvil lifecycle remain authoritative; on a cache/disk
/// miss, `provideChunk` delegates only to the concrete `IChunkGenerator` owned
/// by this provider. Unsupported world types never substitute an empty Chunk.
pub struct ChunkProviderServer {
    droppedChunksSet: HashSet<i64>,
    chunkLoader: AnvilChunkLoader,
    chunkGenerator: Option<Box<dyn IChunkGenerator>>,
    id2ChunkMap: HashMap<i64, Chunk>,
}

impl std::fmt::Debug for ChunkProviderServer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("ChunkProviderServer")
            .field("droppedChunksSet", &self.droppedChunksSet.len())
            .field("chunkLoader", &self.chunkLoader)
            .field("chunkGenerator", &self.chunkGenerator.as_ref().map(|generator| generator.generatorName()))
            .field("id2ChunkMap", &self.id2ChunkMap.len())
            .finish()
    }
}

impl ChunkProviderServer {
    pub fn new(chunkLoaderIn: AnvilChunkLoader) -> Self {
        Self {
            droppedChunksSet: HashSet::new(),
            chunkLoader: chunkLoaderIn,
            chunkGenerator: None,
            id2ChunkMap: HashMap::with_capacity(8192),
        }
    }

    pub fn withGenerator(chunkLoaderIn: AnvilChunkLoader, chunkGeneratorIn: Box<dyn IChunkGenerator>) -> Self {
        Self {
            droppedChunksSet: HashSet::new(),
            chunkLoader: chunkLoaderIn,
            chunkGenerator: Some(chunkGeneratorIn),
            id2ChunkMap: HashMap::with_capacity(8192),
        }
    }

    pub fn getLoadedChunks(&self) -> impl Iterator<Item = &Chunk> { self.id2ChunkMap.values() }
    pub fn getLoadedChunkRef(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.id2ChunkMap.get(&ChunkPos::asLong(x, z))
    }
    pub fn getLoadedChunksMut(&mut self) -> impl Iterator<Item = &mut Chunk> { self.id2ChunkMap.values_mut() }

    /// MCP `getLoadedChunk`; touching a loaded entry clears its `unloaded` flag.
    pub fn getLoadedChunk(&mut self, x: i32, z: i32) -> Option<&mut Chunk> {
        let key = ChunkPos::asLong(x, z);
        let chunk = self.id2ChunkMap.get_mut(&key)?;
        chunk.unloaded = false;
        Some(chunk)
    }

    /// MCP `loadChunk` through the file path only. `populateChunk` and
    /// `recreateStructures` remain generator-owned and are therefore not
    /// guessed here.
    pub fn loadChunk(&mut self, world: &mut WorldServer, x: i32, z: i32) -> bool {
        let key = ChunkPos::asLong(x, z);
        if let Some(chunk) = self.id2ChunkMap.get_mut(&key) {
            chunk.unloaded = false;
            return true;
        }
        let loaded = match world.loadChunkFromLoader(&self.chunkLoader, x, z) {
            Ok(loaded) => loaded,
            Err(error) => {
                log::error!("Couldn't load chunk: {}", error);
                return false;
            }
        };
        let Some(mut chunk) = loaded else { return false; };
        chunk.setLastSaveTime(world.getTotalWorldTime());
        // `Chunk#onChunkLoad` sets the source loaded flag before adding entity
        // and TileEntity objects to World. Object runtime registration remains
        // pending; the persisted NBT ownership is already retained by Chunk.
        chunk.setLoaded(true);
        chunk.unloaded = false;
        self.id2ChunkMap.insert(key, chunk);
        true
    }

    /// MCP `ChunkProviderServer#provideChunk`: resident -> Anvil -> concrete
    /// generator. Vanilla's final `EmptyChunk` fallback is only reachable when
    /// a provider was constructed with a null generator; this semantic port
    /// rejects that incomplete server state rather than presenting empty terrain
    /// as a successfully generated single-player world.
    pub fn provideChunk(&mut self, world: &mut WorldServer, x: i32, z: i32) -> Result<&mut Chunk, String> {
        let key = ChunkPos::asLong(x, z);
        if !self.id2ChunkMap.contains_key(&key) {
            let loaded = world.loadChunkFromLoader(&self.chunkLoader, x, z)
                .map_err(|error| format!("Couldn't load chunk {x},{z}: {error}"))?;
            let mut chunk = if let Some(chunk) = loaded {
                chunk
            } else {
                self.chunkGenerator.as_mut()
                    .ok_or_else(|| format!("No IChunkGenerator available for chunk {x},{z}"))?
                    .provideChunk(x, z)?
            };
            chunk.setLastSaveTime(world.getTotalWorldTime());
            chunk.setLoaded(true);
            chunk.unloaded = false;
            self.id2ChunkMap.insert(key, chunk);
        }
        let chunk = self.id2ChunkMap.get_mut(&key).expect("provided chunk inserted");
        chunk.unloaded = false;
        Ok(chunk)
    }

    pub fn getChunkSnapshot(&self, x: i32, z: i32) -> Option<Chunk> {
        self.id2ChunkMap.get(&ChunkPos::asLong(x, z)).cloned()
    }

    pub fn isChunkLoaded(&self, x:i32, z:i32)->bool {
        self.id2ChunkMap.contains_key(&ChunkPos::asLong(x,z))
    }

    pub fn generatorSeaLevelOverride(&self) -> Option<i32> {
        self.chunkGenerator.as_ref().and_then(|generator| generator.seaLevelOverride())
    }

    /// Source `unload(Chunk)` decision without holding two aliases into the
    /// Rust HashMap. Surface spawn chunks are protected by WorldServer's exact
    /// `World#isSpawnChunk` bounds; Nether/End use the base provider behaviour.
    pub fn unload(&mut self, world: &WorldServer, x: i32, z: i32) {
        if !world.canDropChunk(x, z) { return; }
        let key = ChunkPos::asLong(x, z);
        if let Some(chunk) = self.id2ChunkMap.get_mut(&key) {
            self.droppedChunksSet.insert(key);
            chunk.unloaded = true;
        }
    }

    pub fn unloadAllChunks(&mut self, world: &WorldServer) {
        let coords: Vec<_> = self.id2ChunkMap.values().map(|c| (c.xPosition, c.zPosition)).collect();
        for (x, z) in coords { self.unload(world, x, z); }
    }

    fn saveChunkDataWithLoader(
        loader: &AnvilChunkLoader,
        world: &mut WorldServer,
        chunk: &mut Chunk,
    ) {
        chunk.setLastSaveTime(world.getTotalWorldTime());
        let pending = world.getPendingBlockUpdates(chunk, false);
        if let Err(error) = loader.saveChunk(
            world.saveHandler().base(),
            chunk,
            world.getTotalWorldTime(),
            world.provider.hasSkyLight(),
            Some(pending.as_slice()),
        ) {
            // MCP catches both IOException and MinecraftException here and
            // keeps the server running after reporting the failed save.
            log::error!("Couldn't save chunk: {}", error);
        }
    }

    fn saveChunkExtraDataWithLoader(loader: &AnvilChunkLoader, chunk: &Chunk) {
        if let Err(error) = loader.saveExtraChunkData(chunk) {
            log::error!("Couldn't save entities: {}", error);
        }
    }

    /// MCP `saveChunks`: a normal autosave writes at most 24 chunks; forced
    /// save writes all and invokes the loader's extra-data hook first.
    pub fn saveChunks(&mut self, world: &mut WorldServer, force: bool) -> bool {
        let loader = self.chunkLoader.clone();
        let totalWorldTime = world.getTotalWorldTime();
        let keys: Vec<i64> = self.id2ChunkMap.keys().copied().collect();
        let mut saved = 0usize;
        for key in keys {
            let Some(chunk) = self.id2ChunkMap.get_mut(&key) else { continue; };
            if force {
                Self::saveChunkExtraDataWithLoader(&loader, chunk);
            }
            if chunk.needsSaving(force, totalWorldTime) {
                Self::saveChunkDataWithLoader(&loader, world, chunk);
                chunk.setModified(false);
                saved += 1;
                if saved == 24 && !force { return false; }
            }
        }
        true
    }

    pub fn saveExtraData(&self) { self.chunkLoader.saveExtraData(); }

    /// MCP `unloadQueuedChunks`: at most 100 actually-loaded marked chunks are
    /// removed per call; each is saved before removal. Entity/TileEntity world
    /// deregistration remains coupled to the future concrete server objects,
    /// so only Chunk's source loaded flag is changed here.
    pub fn unloadQueuedChunks(&mut self, world: &mut WorldServer) -> bool {
        if world.disableLevelSaving { return false; }
        let loader = self.chunkLoader.clone();
        let queued: Vec<i64> = self.droppedChunksSet.iter().copied().collect();
        let mut unloadedCount = 0usize;
        for key in queued {
            if unloadedCount >= 100 { break; }
            self.droppedChunksSet.remove(&key);
            let shouldUnload = self.id2ChunkMap.get(&key).is_some_and(|chunk| chunk.unloaded);
            if !shouldUnload { continue; }
            if let Some(mut chunk) = self.id2ChunkMap.remove(&key) {
                chunk.setLoaded(false);
                Self::saveChunkDataWithLoader(&loader, world, &mut chunk);
                Self::saveChunkExtraDataWithLoader(&loader, &chunk);
                unloadedCount += 1;
            }
        }
        loader.chunkTick();
        false
    }

    pub fn canSave(&self, world: &WorldServer) -> bool { !world.disableLevelSaving }

    pub fn makeString(&self) -> String {
        format!("ServerChunkCache: {} Drop: {}", self.id2ChunkMap.len(), self.droppedChunksSet.len())
    }

    pub fn getLoadedChunkCount(&self) -> usize { self.id2ChunkMap.len() }
    pub fn chunkExists(&self, x: i32, z: i32) -> bool { self.id2ChunkMap.contains_key(&ChunkPos::asLong(x, z)) }

    /// MCP `func_191062_e`: resident or queued/on-disk Anvil chunk.
    pub fn isChunkGeneratedAt(&self, x: i32, z: i32) -> io::Result<bool> {
        Ok(self.chunkExists(x, z) || self.chunkLoader.isChunkGeneratedAt(x, z)?)
    }

    pub fn chunkLoader(&self) -> &AnvilChunkLoader { &self.chunkLoader }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::world::GameType::GameType;
    use crate::net::minecraft::world::WorldSettings::WorldSettings;
    use crate::net::minecraft::world::WorldType::WorldType;
    use crate::net::minecraft::world::chunk::storage::AnvilSaveHandler::AnvilSaveHandler;
    use crate::net::minecraft::world::storage::WorldInfo::WorldInfo;

    #[test]
    fn source_cache_and_spawn_unload_contract_are_preserved() {
        let root = std::env::temp_dir().join(format!("mc1122-chunk-provider-server-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let settings = WorldSettings::new(1, GameType::Survival, true, false, WorldType::Default);
        let info = WorldInfo::new(&settings, "World");
        let handler = AnvilSaveHandler::new(&root, "World", false).unwrap();
        let loader = handler.getChunkLoader(&crate::net::minecraft::world::WorldProvider::WorldProvider::new(0)).unwrap();
        let mut provider = ChunkProviderServer::new(loader);
        let world = WorldServer::new(handler, info, 0);
        provider.id2ChunkMap.insert(ChunkPos::asLong(0, 0), Chunk::new(0, 0));
        provider.unload(&world, 0, 0);
        assert!(!provider.id2ChunkMap.get(&ChunkPos::asLong(0, 0)).unwrap().unloaded);
        provider.id2ChunkMap.insert(ChunkPos::asLong(30, 30), Chunk::new(30, 30));
        provider.unload(&world, 30, 30);
        assert!(provider.id2ChunkMap.get(&ChunkPos::asLong(30, 30)).unwrap().unloaded);
        let _ = std::fs::remove_dir_all(root);
    }
}
