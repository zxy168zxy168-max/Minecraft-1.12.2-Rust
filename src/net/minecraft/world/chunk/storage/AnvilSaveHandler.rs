use std::io;
use std::path::Path;

use crate::net::minecraft::world::chunk::storage::AnvilChunkLoader::AnvilChunkLoader;
use crate::net::minecraft::world::chunk::storage::RegionFileCache;
use crate::net::minecraft::world::storage::SaveHandler::SaveHandler;
use crate::net::minecraft::world::storage::ThreadedFileIOBase::ThreadedFileIOBase;
use crate::net::minecraft::world::storage::WorldInfo::WorldInfo;
use crate::net::minecraft::world::WorldProvider::WorldProvider;

/// MCP 1.12.2 `AnvilSaveHandler` storage responsibilities used by the
/// integrated-server launch prefix.  Chunk I/O itself remains a later
/// single-player tranche; this class already owns the source save-version
/// override instead of leaking that responsibility into `WorldInfo`.
#[derive(Debug, Clone)]
pub struct AnvilSaveHandler {
    base: SaveHandler,
}

impl AnvilSaveHandler {
    pub fn new(
        savesDirectory: impl AsRef<Path>,
        saveName: &str,
        storePlayerdata: bool,
    ) -> io::Result<Self> {
        Ok(Self {
            base: SaveHandler::new(savesDirectory, saveName, storePlayerdata)?,
        })
    }

    pub fn loadWorldInfo(&self) -> io::Result<Option<WorldInfo>> {
        self.base.loadWorldInfo()
    }

    /// MCP `AnvilSaveHandler#saveWorldInfoWithPlayer`: force the Anvil save
    /// version before delegating to the normal level.dat rotation.
    pub fn saveWorldInfo(&self, info: &mut WorldInfo) -> io::Result<()> {
        info.setSaveVersion(19133);
        self.base.saveWorldInfo(info)
    }

    /// MCP `AnvilSaveHandler#getChunkLoader`: dimensions -1 and +1 use
    /// `DIM-1` / `DIM1`; the surface world stores regions at the world root.
    pub fn getChunkLoader(&self, provider: &WorldProvider) -> io::Result<AnvilChunkLoader> {
        let root = self.base.getWorldDirectory();
        let location = match provider.getDimension() {
            -1 => root.join("DIM-1"),
            1 => root.join("DIM1"),
            _ => root.to_path_buf(),
        };
        AnvilChunkLoader::new(location)
    }

    /// Direct MCP `AnvilSaveHandler#flush`: wait until every queued threaded
    /// chunk write has completed before closing RegionFile handles.
    pub fn flush(&self) {
        ThreadedFileIOBase::getThreadedIOInstance().waitForFinish();
        RegionFileCache::clearRegionFileReferences();
    }

    pub fn base(&self) -> &SaveHandler {
        &self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::world::GameType::GameType;
    use crate::net::minecraft::world::WorldSettings::WorldSettings;
    use crate::net::minecraft::world::WorldType::WorldType;

    #[test]
    fn chunk_loader_uses_vanilla_dimension_directories() {
        let root = std::env::temp_dir().join("mc1122-anvil-dimension-loader-test");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let handler = AnvilSaveHandler::new(&root, "World", false).unwrap();
        assert!(handler
            .getChunkLoader(&WorldProvider::new(0))
            .unwrap()
            .chunkSaveLocation()
            .ends_with("World"));
        assert!(handler
            .getChunkLoader(&WorldProvider::new(-1))
            .unwrap()
            .chunkSaveLocation()
            .ends_with("World/DIM-1"));
        assert!(handler
            .getChunkLoader(&WorldProvider::new(1))
            .unwrap()
            .chunkSaveLocation()
            .ends_with("World/DIM1"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn save_world_info_assigns_anvil_version_before_rotation() {
        let root = std::env::temp_dir().join("mc1122-anvil-save-handler-test");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let handler = AnvilSaveHandler::new(&root, "World", false).unwrap();
        let settings = WorldSettings::new(123, GameType::Survival, true, false, WorldType::Default);
        let mut info = WorldInfo::new(&settings, "World");
        assert_eq!(info.getSaveVersion(), 0);
        handler.saveWorldInfo(&mut info).unwrap();
        assert_eq!(info.getSaveVersion(), 19133);
        assert_eq!(
            handler.loadWorldInfo().unwrap().unwrap().getSaveVersion(),
            19133
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
