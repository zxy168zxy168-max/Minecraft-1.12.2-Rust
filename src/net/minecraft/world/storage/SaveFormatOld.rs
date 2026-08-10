use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use crate::net::minecraft::nbt::CompressedStreamTools;
use crate::net::minecraft::world::storage::SaveHandler::SaveHandler;
use crate::net::minecraft::world::storage::WorldInfo::WorldInfo;

/// MCP 1.12.2 `SaveFormatOld` base implementation used by AnvilSaveConverter.
#[derive(Debug, Clone)]
pub struct SaveFormatOld { savesDirectory: PathBuf }
impl SaveFormatOld {
    pub fn new(savesDirectory: impl AsRef<Path>) -> Self { Self { savesDirectory: savesDirectory.as_ref().to_path_buf() } }
    pub fn getName(&self) -> &'static str { "Old Format" }
    pub fn getWorldInfo(&self, saveName: &str) -> io::Result<Option<WorldInfo>> {
        let dir = self.savesDirectory.join(saveName);
        if !dir.is_dir() { return Ok(None); }
        for name in ["level.dat", "level.dat_old"] {
            let path = dir.join(name);
            if !path.is_file() { continue; }
            let root = CompressedStreamTools::readCompressed(File::open(path)?)?;
            return Ok(Some(WorldInfo::fromNBT(&root.getCompoundTag("Data"))));
        }
        Ok(None)
    }
    pub fn isNewLevelIdAcceptable(&self, saveName: &str) -> bool {
        let path = self.savesDirectory.join(saveName);
        if path.exists() { return false; }
        if std::fs::create_dir_all(&path).is_err() { return false; }
        std::fs::remove_dir(&path).is_ok()
    }
    pub fn getSaveLoader(&self, saveName: &str, storePlayerdata: bool) -> io::Result<SaveHandler> {
        SaveHandler::new(&self.savesDirectory, saveName, storePlayerdata)
    }
    pub fn savesDirectory(&self) -> &Path { &self.savesDirectory }
}
