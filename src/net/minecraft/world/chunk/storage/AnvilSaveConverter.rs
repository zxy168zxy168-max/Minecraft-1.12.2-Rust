use std::io;
use std::path::Path;

use crate::net::minecraft::world::chunk::storage::AnvilSaveHandler::AnvilSaveHandler;
use crate::net::minecraft::world::storage::SaveFormatOld::SaveFormatOld;
use crate::net::minecraft::world::storage::WorldInfo::WorldInfo;
use crate::net::minecraft::world::storage::WorldSummary::WorldSummary;

/// MCP 1.12.2 `AnvilSaveConverter` subset for discovering and creating Anvil saves.
#[derive(Debug, Clone)]
pub struct AnvilSaveConverter {
    base: SaveFormatOld,
}
impl AnvilSaveConverter {
    pub const SAVE_VERSION: i32 = 19133;
    pub fn new(savesDirectory: impl AsRef<Path>) -> Self {
        Self {
            base: SaveFormatOld::new(savesDirectory),
        }
    }
    pub fn getName(&self) -> &'static str {
        "Anvil"
    }
    pub fn getWorldInfo(&self, name: &str) -> io::Result<Option<WorldInfo>> {
        self.base.getWorldInfo(name)
    }
    pub fn isNewLevelIdAcceptable(&self, name: &str) -> bool {
        self.base.isNewLevelIdAcceptable(name)
    }
    pub fn getSaveLoader(&self, name: &str, storePlayerdata: bool) -> io::Result<AnvilSaveHandler> {
        AnvilSaveHandler::new(self.base.savesDirectory(), name, storePlayerdata)
    }

    pub fn getSaveList(&self) -> io::Result<Vec<WorldSummary>> {
        let root = self.base.savesDirectory();
        if !root.exists() {
            std::fs::create_dir_all(root)?;
        }
        let mut result = Vec::new();
        for entry in std::fs::read_dir(root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let fileName = entry.file_name().to_string_lossy().into_owned();
            let Some(info) = self.getWorldInfo(&fileName)? else {
                continue;
            };
            if !matches!(info.getSaveVersion(), 19132 | 19133) {
                continue;
            }
            let display = if info.getWorldName().is_empty() {
                fileName.clone()
            } else {
                info.getWorldName().to_owned()
            };
            result.push(WorldSummary::new(
                &info,
                fileName,
                display,
                0,
                info.getSaveVersion() != Self::SAVE_VERSION,
            ));
        }
        result.sort();
        Ok(result)
    }
}
