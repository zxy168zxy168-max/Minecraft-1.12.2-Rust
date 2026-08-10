use crate::net::minecraft::world::GameType::GameType;
use crate::net::minecraft::world::WorldType::WorldType;
use crate::net::minecraft::world::storage::WorldInfo::WorldInfo;

/// MCP 1.12.2 `WorldSettings`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldSettings {
    seed: i64,
    theGameType: GameType,
    mapFeaturesEnabled: bool,
    hardcoreEnabled: bool,
    terrainType: WorldType,
    commandsAllowed: bool,
    bonusChestEnabled: bool,
    generatorOptions: String,
}

impl WorldSettings {
    pub fn new(seed: i64, gameType: GameType, enableMapFeatures: bool, hardcoreMode: bool, worldType: WorldType) -> Self {
        Self {
            seed,
            theGameType: gameType,
            mapFeaturesEnabled: enableMapFeatures,
            hardcoreEnabled: hardcoreMode,
            terrainType: worldType,
            commandsAllowed: false,
            bonusChestEnabled: false,
            generatorOptions: String::new(),
        }
    }

    pub fn fromWorldInfo(info: &WorldInfo) -> Self {
        Self::new(info.getSeed(), info.getGameType(), info.isMapFeaturesEnabled(), info.isHardcoreModeEnabled(), info.getTerrainType())
            .setGeneratorOptions(info.getGeneratorOptions().to_owned())
    }

    pub fn enableBonusChest(mut self) -> Self { self.bonusChestEnabled = true; self }
    pub fn enableCommands(mut self) -> Self { self.commandsAllowed = true; self }
    pub fn setGeneratorOptions(mut self, options: impl Into<String>) -> Self { self.generatorOptions = options.into(); self }
    pub const fn isBonusChestEnabled(&self) -> bool { self.bonusChestEnabled }
    pub const fn getSeed(&self) -> i64 { self.seed }
    pub const fn getGameType(&self) -> GameType { self.theGameType }
    pub const fn getHardcoreEnabled(&self) -> bool { self.hardcoreEnabled }
    pub const fn isMapFeaturesEnabled(&self) -> bool { self.mapFeaturesEnabled }
    pub const fn getTerrainType(&self) -> WorldType { self.terrainType }
    pub const fn areCommandsAllowed(&self) -> bool { self.commandsAllowed }
    pub fn getGeneratorOptions(&self) -> &str { &self.generatorOptions }
    pub const fn getGameTypeById(id: i32) -> GameType { GameType::getByID(id) }
}
