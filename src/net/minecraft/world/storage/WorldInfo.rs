use std::sync::Arc;
use crate::net::minecraft::util::datafix::DataFixer::DataFixer;
use crate::net::minecraft::util::datafix::FixTypes::FixTypes;
use crate::net::minecraft::util::datafix::IDataFixer::IDataFixer;
use crate::net::minecraft::util::datafix::IDataWalker::IDataWalker;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::world::EnumDifficulty::EnumDifficulty;
use crate::net::minecraft::world::GameRules::GameRules;
use crate::net::minecraft::world::GameType::GameType;
use crate::net::minecraft::world::WorldSettings::WorldSettings;
use crate::net::minecraft::world::WorldType::WorldType;

/// MCP 1.12.2 `WorldInfo` data subset required by creation, save discovery and
/// later `IntegratedServer` bootstrap. NBT keys and defaults follow the source.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldInfo {
    versionName: String,
    versionId: i32,
    versionSnapshot: bool,
    randomSeed: i64,
    terrainType: WorldType,
    generatorOptions: String,
    spawnX: i32,
    spawnY: i32,
    spawnZ: i32,
    totalTime: i64,
    worldTime: i64,
    lastTimePlayed: i64,
    sizeOnDisk: i64,
    levelName: String,
    saveVersion: i32,
    cleanWeatherTime: i32,
    raining: bool,
    rainTime: i32,
    thundering: bool,
    thunderTime: i32,
    theGameType: GameType,
    mapFeaturesEnabled: bool,
    hardcore: bool,
    allowCommands: bool,
    initialized: bool,
    difficulty: EnumDifficulty,
    difficultyLocked: bool,
    borderCenterX: f64,
    borderCenterZ: f64,
    borderSize: f64,
    borderSizeLerpTime: i64,
    borderSizeLerpTarget: f64,
    borderSafeZone: f64,
    borderDamagePerBlock: f64,
    borderWarningDistance: i32,
    borderWarningTime: i32,
    theGameRules: GameRules,
    playerTag: Option<NBTTagCompound>,
}


struct WorldInfoPlayerDataWalker;
impl IDataWalker for WorldInfoPlayerDataWalker {
    fn process(&self, fixer: &dyn IDataFixer, mut compound: NBTTagCompound, versionIn: i32) -> NBTTagCompound {
        if compound.hasKeyWithType("Player", 10) {
            let player = fixer.processVersioned(FixTypes::Player, compound.getCompoundTag("Player"), versionIn);
            compound.setCompoundTag("Player", player);
        }
        compound
    }
}

impl WorldInfo {
    /// MCP 1.12.2 `WorldInfo#registerFixes`: LEVEL owns the nested Player
    /// compound and delegates it to the PLAYER fixer chain.
    pub fn registerFixes(fixer: &mut DataFixer) {
        fixer.registerWalker(FixTypes::Level, Arc::new(WorldInfoPlayerDataWalker));
    }

    pub const SAVE_VERSION_ANVIL: i32 = 19133;
    pub const DATA_VERSION_1_12_2: i32 = 1343;

    pub fn new(settings: &WorldSettings, name: impl Into<String>) -> Self {
        Self {
            versionName: String::new(),
            versionId: 0,
            versionSnapshot: false,
            randomSeed: settings.getSeed(),
            terrainType: settings.getTerrainType(),
            generatorOptions: settings.getGeneratorOptions().to_owned(),
            spawnX: 0,
            spawnY: 0,
            spawnZ: 0,
            totalTime: 0,
            worldTime: 0,
            lastTimePlayed: 0,
            sizeOnDisk: 0,
            levelName: name.into(),
            saveVersion: 0,
            cleanWeatherTime: 0,
            raining: false,
            rainTime: 0,
            thundering: false,
            thunderTime: 0,
            theGameType: settings.getGameType(),
            mapFeaturesEnabled: settings.isMapFeaturesEnabled(),
            hardcore: settings.getHardcoreEnabled(),
            allowCommands: settings.areCommandsAllowed(),
            initialized: false,
            difficulty: EnumDifficulty::Normal,
            difficultyLocked: false,
            borderCenterX: 0.0,
            borderCenterZ: 0.0,
            borderSize: 6.0E7,
            borderSizeLerpTime: 0,
            borderSizeLerpTarget: 0.0,
            borderSafeZone: 5.0,
            borderDamagePerBlock: 0.2,
            borderWarningDistance: 5,
            borderWarningTime: 15,
            theGameRules: GameRules::new(),
            playerTag: None,
        }
    }

    pub fn fromNBT(nbt: &NBTTagCompound) -> Self {
        let version = nbt.getCompoundTag("Version");
        let terrain = if nbt.hasKey("generatorName") {
            let parsed = WorldType::parseWorldType(&nbt.getString("generatorName"));
            parsed.getWorldTypeForGeneratorVersion(nbt.getInteger("generatorVersion"))
        } else {
            WorldType::Default
        };
        let mut rules = GameRules::new();
        if nbt.hasKey("GameRules") {
            rules.readFromNBT(&nbt.getCompoundTag("GameRules"));
        }
        Self {
            versionName: version.getString("Name"),
            versionId: version.getInteger("Id"),
            versionSnapshot: version.getBoolean("Snapshot"),
            randomSeed: nbt.getLong("RandomSeed"),
            terrainType: terrain,
            generatorOptions: nbt.getString("generatorOptions"),
            spawnX: nbt.getInteger("SpawnX"),
            spawnY: nbt.getInteger("SpawnY"),
            spawnZ: nbt.getInteger("SpawnZ"),
            totalTime: nbt.getLong("Time"),
            worldTime: if nbt.hasKey("DayTime") { nbt.getLong("DayTime") } else { nbt.getLong("Time") },
            lastTimePlayed: nbt.getLong("LastPlayed"),
            sizeOnDisk: nbt.getLong("SizeOnDisk"),
            levelName: nbt.getString("LevelName"),
            saveVersion: nbt.getInteger("version"),
            cleanWeatherTime: nbt.getInteger("clearWeatherTime"),
            raining: nbt.getBoolean("raining"),
            rainTime: nbt.getInteger("rainTime"),
            thundering: nbt.getBoolean("thundering"),
            thunderTime: nbt.getInteger("thunderTime"),
            theGameType: GameType::getByID(nbt.getInteger("GameType")),
            mapFeaturesEnabled: if nbt.hasKey("MapFeatures") { nbt.getBoolean("MapFeatures") } else { true },
            hardcore: nbt.getBoolean("hardcore"),
            allowCommands: if nbt.hasKey("allowCommands") { nbt.getBoolean("allowCommands") } else { GameType::getByID(nbt.getInteger("GameType")) == GameType::Creative },
            initialized: if nbt.hasKey("initialized") { nbt.getBoolean("initialized") } else { true },
            difficulty: if nbt.hasKey("Difficulty") { EnumDifficulty::getDifficultyEnum(nbt.getByte("Difficulty") as u8) } else { EnumDifficulty::Normal },
            difficultyLocked: nbt.getBoolean("DifficultyLocked"),
            borderCenterX: if nbt.hasKey("BorderCenterX") { nbt.getDouble("BorderCenterX") } else { 0.0 },
            borderCenterZ: if nbt.hasKey("BorderCenterZ") { nbt.getDouble("BorderCenterZ") } else { 0.0 },
            borderSize: if nbt.hasKey("BorderSize") { nbt.getDouble("BorderSize") } else { 6.0E7 },
            borderSizeLerpTime: nbt.getLong("BorderSizeLerpTime"),
            borderSizeLerpTarget: if nbt.hasKey("BorderSizeLerpTarget") { nbt.getDouble("BorderSizeLerpTarget") } else { 0.0 },
            borderSafeZone: if nbt.hasKey("BorderSafeZone") { nbt.getDouble("BorderSafeZone") } else { 5.0 },
            borderDamagePerBlock: if nbt.hasKey("BorderDamagePerBlock") { nbt.getDouble("BorderDamagePerBlock") } else { 0.2 },
            borderWarningDistance: if nbt.hasKey("BorderWarningBlocks") { nbt.getDouble("BorderWarningBlocks") as i32 } else { 5 },
            borderWarningTime: if nbt.hasKey("BorderWarningTime") { nbt.getDouble("BorderWarningTime") as i32 } else { 15 },
            theGameRules: rules,
            playerTag: nbt.hasKeyWithType("Player", 10).then(|| nbt.getCompoundTag("Player")),
        }
    }

    /// MCP `WorldInfo#cloneNBTCompound(null)`: null reuses the retained
    /// playerTag loaded from level.dat.
    pub fn cloneNBTCompound(&self) -> NBTTagCompound { self.cloneNBTCompoundWithPlayer(None) }

    pub fn cloneNBTCompoundWithPlayer(&self, player: Option<&NBTTagCompound>) -> NBTTagCompound {
        let mut nbt = NBTTagCompound::new();
        let mut version = NBTTagCompound::new();
        version.setString("Name", "1.12.2");
        version.setInteger("Id", Self::DATA_VERSION_1_12_2);
        version.setBoolean("Snapshot", false);
        nbt.setCompoundTag("Version", version);
        nbt.setInteger("DataVersion", Self::DATA_VERSION_1_12_2);
        nbt.setLong("RandomSeed", self.randomSeed);
        nbt.setString("generatorName", self.terrainType.getWorldTypeName());
        nbt.setInteger("generatorVersion", self.terrainType.getGeneratorVersion());
        nbt.setString("generatorOptions", self.generatorOptions.clone());
        nbt.setInteger("GameType", self.theGameType.getID());
        nbt.setBoolean("MapFeatures", self.mapFeaturesEnabled);
        nbt.setInteger("SpawnX", self.spawnX);
        nbt.setInteger("SpawnY", self.spawnY);
        nbt.setInteger("SpawnZ", self.spawnZ);
        nbt.setLong("Time", self.totalTime);
        nbt.setLong("DayTime", self.worldTime);
        nbt.setLong("SizeOnDisk", self.sizeOnDisk);
        nbt.setLong("LastPlayed", current_time_millis());
        nbt.setString("LevelName", self.levelName.clone());
        nbt.setInteger("version", self.saveVersion);
        nbt.setInteger("clearWeatherTime", self.cleanWeatherTime);
        nbt.setInteger("rainTime", self.rainTime);
        nbt.setBoolean("raining", self.raining);
        nbt.setInteger("thunderTime", self.thunderTime);
        nbt.setBoolean("thundering", self.thundering);
        nbt.setBoolean("hardcore", self.hardcore);
        nbt.setBoolean("allowCommands", self.allowCommands);
        nbt.setBoolean("initialized", self.initialized);
        nbt.setDouble("BorderCenterX", self.borderCenterX);
        nbt.setDouble("BorderCenterZ", self.borderCenterZ);
        nbt.setDouble("BorderSize", self.borderSize);
        nbt.setLong("BorderSizeLerpTime", self.borderSizeLerpTime);
        nbt.setDouble("BorderSafeZone", self.borderSafeZone);
        nbt.setDouble("BorderDamagePerBlock", self.borderDamagePerBlock);
        nbt.setDouble("BorderSizeLerpTarget", self.borderSizeLerpTarget);
        nbt.setDouble("BorderWarningBlocks", self.borderWarningDistance as f64);
        nbt.setDouble("BorderWarningTime", self.borderWarningTime as f64);
        nbt.setByte("Difficulty", self.difficulty.getDifficultyId() as i8);
        nbt.setBoolean("DifficultyLocked", self.difficultyLocked);
        nbt.setCompoundTag("GameRules", self.theGameRules.writeToNBT());
        nbt.setCompoundTag("DimensionData", NBTTagCompound::new());
        if let Some(player)=player.or(self.playerTag.as_ref()) { nbt.setCompoundTag("Player",player.clone()); }
        nbt
    }

    pub fn getPlayerNBTTagCompound(&self) -> Option<&NBTTagCompound> { self.playerTag.as_ref() }
    pub fn setPlayerNBTTagCompound(&mut self, player: Option<NBTTagCompound>) { self.playerTag=player; }

    pub const fn getSeed(&self) -> i64 { self.randomSeed }
    pub const fn getGameType(&self) -> GameType { self.theGameType }
    pub const fn isMapFeaturesEnabled(&self) -> bool { self.mapFeaturesEnabled }
    pub const fn isHardcoreModeEnabled(&self) -> bool { self.hardcore }
    pub const fn getTerrainType(&self) -> WorldType { self.terrainType }
    pub fn getGeneratorOptions(&self) -> &str { &self.generatorOptions }
    pub const fn areCommandsAllowed(&self) -> bool { self.allowCommands }
    pub fn getWorldName(&self) -> &str { &self.levelName }
    pub fn setWorldName(&mut self, name: impl Into<String>) { self.levelName = name.into(); }
    pub const fn getWorldTotalTime(&self) -> i64 { self.totalTime }
    pub const fn getWorldTime(&self) -> i64 { self.worldTime }
    pub fn setWorldTotalTime(&mut self, time: i64) { self.totalTime = time; }
    pub fn setWorldTime(&mut self, time: i64) { self.worldTime = time; }
    pub const fn getSpawnX(&self) -> i32 { self.spawnX }
    pub const fn getSpawnY(&self) -> i32 { self.spawnY }
    pub const fn getSpawnZ(&self) -> i32 { self.spawnZ }
    pub fn setSpawnX(&mut self, x: i32) { self.spawnX = x; }
    pub fn setSpawnY(&mut self, y: i32) { self.spawnY = y; }
    pub fn setSpawnZ(&mut self, z: i32) { self.spawnZ = z; }
    pub fn setServerInitialized(&mut self, initializedIn: bool) { self.initialized = initializedIn; }
    pub const fn getLastTimePlayed(&self) -> i64 { self.lastTimePlayed }
    pub const fn getSizeOnDisk(&self) -> i64 { self.sizeOnDisk }
    pub const fn getSaveVersion(&self) -> i32 { self.saveVersion }
    pub fn setSaveVersion(&mut self, version: i32) { self.saveVersion = version; }
    pub fn getVersionName(&self) -> &str { &self.versionName }
    pub const fn getVersionId(&self) -> i32 { self.versionId }
    pub const fn isVersionSnapshot(&self) -> bool { self.versionSnapshot }
    pub const fn isInitialized(&self) -> bool { self.initialized }
    pub const fn getDifficulty(&self) -> EnumDifficulty { self.difficulty }
    pub fn setDifficulty(&mut self, difficulty: EnumDifficulty) { self.difficulty = difficulty; }
    pub const fn isDifficultyLocked(&self) -> bool { self.difficultyLocked }
    pub fn setDifficultyLocked(&mut self, locked: bool) { self.difficultyLocked = locked; }
    pub fn getGameRulesInstance(&self) -> &GameRules { &self.theGameRules }
    pub fn getGameRulesInstanceMut(&mut self) -> &mut GameRules { &mut self.theGameRules }
}

fn current_time_millis() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_world_nbt_matches_1122_version_and_creation_flags() {
        let settings = WorldSettings::new(123, GameType::Creative, true, false, WorldType::Flat)
            .enableCommands()
            .setGeneratorOptions("3;minecraft:bedrock,2*minecraft:dirt,minecraft:grass;1;");
        let info = WorldInfo::new(&settings, "Test World");
        let nbt = info.cloneNBTCompound();
        assert_eq!(nbt.getInteger("DataVersion"), 1343);
        // MCP WorldInfo(WorldSettings, name) leaves this field at the Java
        // default 0; AnvilSaveHandler owns the 19133 assignment on save.
        assert_eq!(nbt.getInteger("version"), 0);
        assert_eq!(nbt.getLong("RandomSeed"), 123);
        assert_eq!(nbt.getString("generatorName"), "flat");
        assert_eq!(nbt.getInteger("GameType"), 1);
        assert!(nbt.getBoolean("allowCommands"));
        assert!(!nbt.getBoolean("initialized"));
        assert_eq!(nbt.getString("LevelName"), "Test World");
    }
}
