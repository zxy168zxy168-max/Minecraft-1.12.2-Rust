use crate::net::minecraft::world::storage::WorldInfo::WorldInfo;
use crate::net::minecraft::world::GameType::GameType;
use std::cmp::Ordering;

/// MCP 1.12.2 `WorldSummary` data used by the world-selection list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldSummary {
    fileName: String,
    displayName: String,
    lastTimePlayed: i64,
    sizeOnDisk: i64,
    requiresConversion: bool,
    theEnumGameType: GameType,
    hardcore: bool,
    cheatsEnabled: bool,
    versionName: String,
    versionId: i32,
    versionSnapshot: bool,
}

impl WorldSummary {
    pub fn new(
        info: &WorldInfo,
        fileName: impl Into<String>,
        displayName: impl Into<String>,
        sizeOnDisk: i64,
        requiresConversion: bool,
    ) -> Self {
        Self {
            fileName: fileName.into(),
            displayName: displayName.into(),
            lastTimePlayed: info.getLastTimePlayed(),
            sizeOnDisk,
            requiresConversion,
            theEnumGameType: info.getGameType(),
            hardcore: info.isHardcoreModeEnabled(),
            cheatsEnabled: info.areCommandsAllowed(),
            versionName: info.getVersionName().to_owned(),
            versionId: info.getVersionId(),
            versionSnapshot: info.isVersionSnapshot(),
        }
    }
    pub fn getFileName(&self) -> &str {
        &self.fileName
    }
    pub fn getDisplayName(&self) -> &str {
        &self.displayName
    }
    pub const fn getSizeOnDisk(&self) -> i64 {
        self.sizeOnDisk
    }
    pub const fn requiresConversion(&self) -> bool {
        self.requiresConversion
    }
    pub const fn getLastTimePlayed(&self) -> i64 {
        self.lastTimePlayed
    }
    pub const fn getEnumGameType(&self) -> GameType {
        self.theEnumGameType
    }
    pub const fn isHardcoreModeEnabled(&self) -> bool {
        self.hardcore
    }
    pub const fn getCheatsEnabled(&self) -> bool {
        self.cheatsEnabled
    }
    pub fn getVersionName(&self) -> &str {
        if self.versionName.is_empty() {
            "unknown"
        } else {
            &self.versionName
        }
    }
    pub const fn markVersionInList(&self) -> bool {
        self.versionId != 1343 || self.versionSnapshot
    }
    pub const fn askToOpenWorld(&self) -> bool {
        self.versionId > 1343
    }
}

impl Ord for WorldSummary {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .lastTimePlayed
            .cmp(&self.lastTimePlayed)
            .then_with(|| self.fileName.cmp(&other.fileName))
    }
}
impl PartialOrd for WorldSummary {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
