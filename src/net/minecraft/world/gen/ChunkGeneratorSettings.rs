use serde_json::{Map, Value};

/// MCP 1.12.2 `ChunkGeneratorSettings`. Values are copied from the nested
/// `Factory` exactly at build time; this is the authoritative customized-world
/// configuration consumed by GenLayer and ChunkGeneratorOverworld.

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkGeneratorSettings {
    pub coordinateScale: f32,
    pub heightScale: f32,
    pub upperLimitScale: f32,
    pub lowerLimitScale: f32,
    pub depthNoiseScaleX: f32,
    pub depthNoiseScaleZ: f32,
    pub depthNoiseScaleExponent: f32,
    pub mainNoiseScaleX: f32,
    pub mainNoiseScaleY: f32,
    pub mainNoiseScaleZ: f32,
    pub baseSize: f32,
    pub stretchY: f32,
    pub biomeDepthWeight: f32,
    pub biomeDepthOffSet: f32,
    pub biomeScaleWeight: f32,
    pub biomeScaleOffset: f32,
    pub seaLevel: i32,
    pub useCaves: bool,
    pub useDungeons: bool,
    pub dungeonChance: i32,
    pub useStrongholds: bool,
    pub useVillages: bool,
    pub useMineShafts: bool,
    pub useTemples: bool,
    pub useMonuments: bool,
    pub field_191077_z: bool,
    pub useRavines: bool,
    pub useWaterLakes: bool,
    pub waterLakeChance: i32,
    pub useLavaLakes: bool,
    pub lavaLakeChance: i32,
    pub useLavaOceans: bool,
    pub fixedBiome: i32,
    pub biomeSize: i32,
    pub riverSize: i32,
    pub dirtSize: i32,
    pub dirtCount: i32,
    pub dirtMinHeight: i32,
    pub dirtMaxHeight: i32,
    pub gravelSize: i32,
    pub gravelCount: i32,
    pub gravelMinHeight: i32,
    pub gravelMaxHeight: i32,
    pub graniteSize: i32,
    pub graniteCount: i32,
    pub graniteMinHeight: i32,
    pub graniteMaxHeight: i32,
    pub dioriteSize: i32,
    pub dioriteCount: i32,
    pub dioriteMinHeight: i32,
    pub dioriteMaxHeight: i32,
    pub andesiteSize: i32,
    pub andesiteCount: i32,
    pub andesiteMinHeight: i32,
    pub andesiteMaxHeight: i32,
    pub coalSize: i32,
    pub coalCount: i32,
    pub coalMinHeight: i32,
    pub coalMaxHeight: i32,
    pub ironSize: i32,
    pub ironCount: i32,
    pub ironMinHeight: i32,
    pub ironMaxHeight: i32,
    pub goldSize: i32,
    pub goldCount: i32,
    pub goldMinHeight: i32,
    pub goldMaxHeight: i32,
    pub redstoneSize: i32,
    pub redstoneCount: i32,
    pub redstoneMinHeight: i32,
    pub redstoneMaxHeight: i32,
    pub diamondSize: i32,
    pub diamondCount: i32,
    pub diamondMinHeight: i32,
    pub diamondMaxHeight: i32,
    pub lapisSize: i32,
    pub lapisCount: i32,
    pub lapisCenterHeight: i32,
    pub lapisSpread: i32,
}

/// MCP 1.12.2 `ChunkGeneratorSettings.Factory`.
#[derive(Debug, Clone, PartialEq)]
pub struct Factory {
    pub coordinateScale: f32,
    pub heightScale: f32,
    pub upperLimitScale: f32,
    pub lowerLimitScale: f32,
    pub depthNoiseScaleX: f32,
    pub depthNoiseScaleZ: f32,
    pub depthNoiseScaleExponent: f32,
    pub mainNoiseScaleX: f32,
    pub mainNoiseScaleY: f32,
    pub mainNoiseScaleZ: f32,
    pub baseSize: f32,
    pub stretchY: f32,
    pub biomeDepthWeight: f32,
    pub biomeDepthOffset: f32,
    pub biomeScaleWeight: f32,
    pub biomeScaleOffset: f32,
    pub seaLevel: i32,
    pub useCaves: bool,
    pub useDungeons: bool,
    pub dungeonChance: i32,
    pub useStrongholds: bool,
    pub useVillages: bool,
    pub useMineShafts: bool,
    pub useTemples: bool,
    pub useMonuments: bool,
    pub field_191076_A: bool,
    pub useRavines: bool,
    pub useWaterLakes: bool,
    pub waterLakeChance: i32,
    pub useLavaLakes: bool,
    pub lavaLakeChance: i32,
    pub useLavaOceans: bool,
    pub fixedBiome: i32,
    pub biomeSize: i32,
    pub riverSize: i32,
    pub dirtSize: i32,
    pub dirtCount: i32,
    pub dirtMinHeight: i32,
    pub dirtMaxHeight: i32,
    pub gravelSize: i32,
    pub gravelCount: i32,
    pub gravelMinHeight: i32,
    pub gravelMaxHeight: i32,
    pub graniteSize: i32,
    pub graniteCount: i32,
    pub graniteMinHeight: i32,
    pub graniteMaxHeight: i32,
    pub dioriteSize: i32,
    pub dioriteCount: i32,
    pub dioriteMinHeight: i32,
    pub dioriteMaxHeight: i32,
    pub andesiteSize: i32,
    pub andesiteCount: i32,
    pub andesiteMinHeight: i32,
    pub andesiteMaxHeight: i32,
    pub coalSize: i32,
    pub coalCount: i32,
    pub coalMinHeight: i32,
    pub coalMaxHeight: i32,
    pub ironSize: i32,
    pub ironCount: i32,
    pub ironMinHeight: i32,
    pub ironMaxHeight: i32,
    pub goldSize: i32,
    pub goldCount: i32,
    pub goldMinHeight: i32,
    pub goldMaxHeight: i32,
    pub redstoneSize: i32,
    pub redstoneCount: i32,
    pub redstoneMinHeight: i32,
    pub redstoneMaxHeight: i32,
    pub diamondSize: i32,
    pub diamondCount: i32,
    pub diamondMinHeight: i32,
    pub diamondMaxHeight: i32,
    pub lapisSize: i32,
    pub lapisCount: i32,
    pub lapisCenterHeight: i32,
    pub lapisSpread: i32,
}

impl Default for Factory { fn default() -> Self { Self::new() } }

impl Factory {
    pub fn new() -> Self { Self {
        coordinateScale: 684.412_f32,
        heightScale: 684.412_f32,
        upperLimitScale: 512.0_f32,
        lowerLimitScale: 512.0_f32,
        depthNoiseScaleX: 200.0_f32,
        depthNoiseScaleZ: 200.0_f32,
        depthNoiseScaleExponent: 0.5_f32,
        mainNoiseScaleX: 80.0_f32,
        mainNoiseScaleY: 160.0_f32,
        mainNoiseScaleZ: 80.0_f32,
        baseSize: 8.5_f32,
        stretchY: 12.0_f32,
        biomeDepthWeight: 1.0_f32,
        biomeDepthOffset: 0.0_f32,
        biomeScaleWeight: 1.0_f32,
        biomeScaleOffset: 0.0_f32,
        seaLevel: 63_i32,
        useCaves: true,
        useDungeons: true,
        dungeonChance: 8_i32,
        useStrongholds: true,
        useVillages: true,
        useMineShafts: true,
        useTemples: true,
        useMonuments: true,
        field_191076_A: true,
        useRavines: true,
        useWaterLakes: true,
        waterLakeChance: 4_i32,
        useLavaLakes: true,
        lavaLakeChance: 80_i32,
        useLavaOceans: false,
        fixedBiome: -1_i32,
        biomeSize: 4_i32,
        riverSize: 4_i32,
        dirtSize: 33_i32,
        dirtCount: 10_i32,
        dirtMinHeight: 0_i32,
        dirtMaxHeight: 256_i32,
        gravelSize: 33_i32,
        gravelCount: 8_i32,
        gravelMinHeight: 0_i32,
        gravelMaxHeight: 256_i32,
        graniteSize: 33_i32,
        graniteCount: 10_i32,
        graniteMinHeight: 0_i32,
        graniteMaxHeight: 80_i32,
        dioriteSize: 33_i32,
        dioriteCount: 10_i32,
        dioriteMinHeight: 0_i32,
        dioriteMaxHeight: 80_i32,
        andesiteSize: 33_i32,
        andesiteCount: 10_i32,
        andesiteMinHeight: 0_i32,
        andesiteMaxHeight: 80_i32,
        coalSize: 17_i32,
        coalCount: 20_i32,
        coalMinHeight: 0_i32,
        coalMaxHeight: 128_i32,
        ironSize: 9_i32,
        ironCount: 20_i32,
        ironMinHeight: 0_i32,
        ironMaxHeight: 64_i32,
        goldSize: 9_i32,
        goldCount: 2_i32,
        goldMinHeight: 0_i32,
        goldMaxHeight: 32_i32,
        redstoneSize: 8_i32,
        redstoneCount: 8_i32,
        redstoneMinHeight: 0_i32,
        redstoneMaxHeight: 16_i32,
        diamondSize: 8_i32,
        diamondCount: 1_i32,
        diamondMinHeight: 0_i32,
        diamondMaxHeight: 16_i32,
        lapisSize: 7_i32,
        lapisCount: 1_i32,
        lapisCenterHeight: 16_i32,
        lapisSpread: 16_i32,
    } }

    /// MCP `Factory#jsonToFactory`: empty/malformed JSON returns defaults;
    /// a type error inside a valid object preserves fields parsed before that
    /// point and skips the remainder, matching the source try/catch boundary.
    pub fn jsonToFactory(text: &str) -> Self {
        if text.is_empty() { return Self::new(); }
        let Ok(value) = serde_json::from_str::<Value>(text) else { return Self::new(); };
        let Some(object) = value.as_object() else { return Self::new(); };
        let mut factory = Self::new();
        let _ = factory.applyJson(object);
        factory
    }

    fn applyJson(&mut self, object: &Map<String, Value>) -> Result<(), ()> {
        if let Some(value) = object.get("coordinateScale") { self.coordinateScale = readFloat(value)?; }
        if let Some(value) = object.get("heightScale") { self.heightScale = readFloat(value)?; }
        if let Some(value) = object.get("lowerLimitScale") { self.lowerLimitScale = readFloat(value)?; }
        if let Some(value) = object.get("upperLimitScale") { self.upperLimitScale = readFloat(value)?; }
        if let Some(value) = object.get("depthNoiseScaleX") { self.depthNoiseScaleX = readFloat(value)?; }
        if let Some(value) = object.get("depthNoiseScaleZ") { self.depthNoiseScaleZ = readFloat(value)?; }
        if let Some(value) = object.get("depthNoiseScaleExponent") { self.depthNoiseScaleExponent = readFloat(value)?; }
        if let Some(value) = object.get("mainNoiseScaleX") { self.mainNoiseScaleX = readFloat(value)?; }
        if let Some(value) = object.get("mainNoiseScaleY") { self.mainNoiseScaleY = readFloat(value)?; }
        if let Some(value) = object.get("mainNoiseScaleZ") { self.mainNoiseScaleZ = readFloat(value)?; }
        if let Some(value) = object.get("baseSize") { self.baseSize = readFloat(value)?; }
        if let Some(value) = object.get("stretchY") { self.stretchY = readFloat(value)?; }
        if let Some(value) = object.get("biomeDepthWeight") { self.biomeDepthWeight = readFloat(value)?; }
        if let Some(value) = object.get("biomeDepthOffset") { self.biomeDepthOffset = readFloat(value)?; }
        if let Some(value) = object.get("biomeScaleWeight") { self.biomeScaleWeight = readFloat(value)?; }
        if let Some(value) = object.get("biomeScaleOffset") { self.biomeScaleOffset = readFloat(value)?; }
        if let Some(value) = object.get("seaLevel") { self.seaLevel = readInt(value)?; }
        if let Some(value) = object.get("useCaves") { self.useCaves = readBoolean(value)?; }
        if let Some(value) = object.get("useDungeons") { self.useDungeons = readBoolean(value)?; }
        if let Some(value) = object.get("dungeonChance") { self.dungeonChance = readInt(value)?; }
        if let Some(value) = object.get("useStrongholds") { self.useStrongholds = readBoolean(value)?; }
        if let Some(value) = object.get("useVillages") { self.useVillages = readBoolean(value)?; }
        if let Some(value) = object.get("useMineShafts") { self.useMineShafts = readBoolean(value)?; }
        if let Some(value) = object.get("useTemples") { self.useTemples = readBoolean(value)?; }
        if let Some(value) = object.get("useMonuments") { self.useMonuments = readBoolean(value)?; }
        if let Some(value) = object.get("useMansions") { self.field_191076_A = readBoolean(value)?; }
        if let Some(value) = object.get("useRavines") { self.useRavines = readBoolean(value)?; }
        if let Some(value) = object.get("useWaterLakes") { self.useWaterLakes = readBoolean(value)?; }
        if let Some(value) = object.get("waterLakeChance") { self.waterLakeChance = readInt(value)?; }
        if let Some(value) = object.get("useLavaLakes") { self.useLavaLakes = readBoolean(value)?; }
        if let Some(value) = object.get("lavaLakeChance") { self.lavaLakeChance = readInt(value)?; }
        if let Some(value) = object.get("useLavaOceans") { self.useLavaOceans = readBoolean(value)?; }
        if let Some(value) = object.get("fixedBiome") { self.fixedBiome = readInt(value)?; }
        if self.fixedBiome < 38 && self.fixedBiome >= -1 {
            if self.fixedBiome >= 8 { self.fixedBiome += 2; } // old customized-world biome ids omit Hell/Sky
        } else { self.fixedBiome = -1; }
        if let Some(value) = object.get("biomeSize") { self.biomeSize = readInt(value)?; }
        if let Some(value) = object.get("riverSize") { self.riverSize = readInt(value)?; }
        if let Some(value) = object.get("dirtSize") { self.dirtSize = readInt(value)?; }
        if let Some(value) = object.get("dirtCount") { self.dirtCount = readInt(value)?; }
        if let Some(value) = object.get("dirtMinHeight") { self.dirtMinHeight = readInt(value)?; }
        if let Some(value) = object.get("dirtMaxHeight") { self.dirtMaxHeight = readInt(value)?; }
        if let Some(value) = object.get("gravelSize") { self.gravelSize = readInt(value)?; }
        if let Some(value) = object.get("gravelCount") { self.gravelCount = readInt(value)?; }
        if let Some(value) = object.get("gravelMinHeight") { self.gravelMinHeight = readInt(value)?; }
        if let Some(value) = object.get("gravelMaxHeight") { self.gravelMaxHeight = readInt(value)?; }
        if let Some(value) = object.get("graniteSize") { self.graniteSize = readInt(value)?; }
        if let Some(value) = object.get("graniteCount") { self.graniteCount = readInt(value)?; }
        if let Some(value) = object.get("graniteMinHeight") { self.graniteMinHeight = readInt(value)?; }
        if let Some(value) = object.get("graniteMaxHeight") { self.graniteMaxHeight = readInt(value)?; }
        if let Some(value) = object.get("dioriteSize") { self.dioriteSize = readInt(value)?; }
        if let Some(value) = object.get("dioriteCount") { self.dioriteCount = readInt(value)?; }
        if let Some(value) = object.get("dioriteMinHeight") { self.dioriteMinHeight = readInt(value)?; }
        if let Some(value) = object.get("dioriteMaxHeight") { self.dioriteMaxHeight = readInt(value)?; }
        if let Some(value) = object.get("andesiteSize") { self.andesiteSize = readInt(value)?; }
        if let Some(value) = object.get("andesiteCount") { self.andesiteCount = readInt(value)?; }
        if let Some(value) = object.get("andesiteMinHeight") { self.andesiteMinHeight = readInt(value)?; }
        if let Some(value) = object.get("andesiteMaxHeight") { self.andesiteMaxHeight = readInt(value)?; }
        if let Some(value) = object.get("coalSize") { self.coalSize = readInt(value)?; }
        if let Some(value) = object.get("coalCount") { self.coalCount = readInt(value)?; }
        if let Some(value) = object.get("coalMinHeight") { self.coalMinHeight = readInt(value)?; }
        if let Some(value) = object.get("coalMaxHeight") { self.coalMaxHeight = readInt(value)?; }
        if let Some(value) = object.get("ironSize") { self.ironSize = readInt(value)?; }
        if let Some(value) = object.get("ironCount") { self.ironCount = readInt(value)?; }
        if let Some(value) = object.get("ironMinHeight") { self.ironMinHeight = readInt(value)?; }
        if let Some(value) = object.get("ironMaxHeight") { self.ironMaxHeight = readInt(value)?; }
        if let Some(value) = object.get("goldSize") { self.goldSize = readInt(value)?; }
        if let Some(value) = object.get("goldCount") { self.goldCount = readInt(value)?; }
        if let Some(value) = object.get("goldMinHeight") { self.goldMinHeight = readInt(value)?; }
        if let Some(value) = object.get("goldMaxHeight") { self.goldMaxHeight = readInt(value)?; }
        if let Some(value) = object.get("redstoneSize") { self.redstoneSize = readInt(value)?; }
        if let Some(value) = object.get("redstoneCount") { self.redstoneCount = readInt(value)?; }
        if let Some(value) = object.get("redstoneMinHeight") { self.redstoneMinHeight = readInt(value)?; }
        if let Some(value) = object.get("redstoneMaxHeight") { self.redstoneMaxHeight = readInt(value)?; }
        if let Some(value) = object.get("diamondSize") { self.diamondSize = readInt(value)?; }
        if let Some(value) = object.get("diamondCount") { self.diamondCount = readInt(value)?; }
        if let Some(value) = object.get("diamondMinHeight") { self.diamondMinHeight = readInt(value)?; }
        if let Some(value) = object.get("diamondMaxHeight") { self.diamondMaxHeight = readInt(value)?; }
        if let Some(value) = object.get("lapisSize") { self.lapisSize = readInt(value)?; }
        if let Some(value) = object.get("lapisCount") { self.lapisCount = readInt(value)?; }
        if let Some(value) = object.get("lapisCenterHeight") { self.lapisCenterHeight = readInt(value)?; }
        if let Some(value) = object.get("lapisSpread") { self.lapisSpread = readInt(value)?; }
        Ok(())
    }

    pub fn build(&self) -> ChunkGeneratorSettings { ChunkGeneratorSettings {
        coordinateScale: self.coordinateScale,
        heightScale: self.heightScale,
        upperLimitScale: self.upperLimitScale,
        lowerLimitScale: self.lowerLimitScale,
        depthNoiseScaleX: self.depthNoiseScaleX,
        depthNoiseScaleZ: self.depthNoiseScaleZ,
        depthNoiseScaleExponent: self.depthNoiseScaleExponent,
        mainNoiseScaleX: self.mainNoiseScaleX,
        mainNoiseScaleY: self.mainNoiseScaleY,
        mainNoiseScaleZ: self.mainNoiseScaleZ,
        baseSize: self.baseSize,
        stretchY: self.stretchY,
        biomeDepthWeight: self.biomeDepthWeight,
        biomeDepthOffSet: self.biomeDepthOffset,
        biomeScaleWeight: self.biomeScaleWeight,
        biomeScaleOffset: self.biomeScaleOffset,
        seaLevel: self.seaLevel,
        useCaves: self.useCaves,
        useDungeons: self.useDungeons,
        dungeonChance: self.dungeonChance,
        useStrongholds: self.useStrongholds,
        useVillages: self.useVillages,
        useMineShafts: self.useMineShafts,
        useTemples: self.useTemples,
        useMonuments: self.useMonuments,
        field_191077_z: self.field_191076_A,
        useRavines: self.useRavines,
        useWaterLakes: self.useWaterLakes,
        waterLakeChance: self.waterLakeChance,
        useLavaLakes: self.useLavaLakes,
        lavaLakeChance: self.lavaLakeChance,
        useLavaOceans: self.useLavaOceans,
        fixedBiome: self.fixedBiome,
        biomeSize: self.biomeSize,
        riverSize: self.riverSize,
        dirtSize: self.dirtSize,
        dirtCount: self.dirtCount,
        dirtMinHeight: self.dirtMinHeight,
        dirtMaxHeight: self.dirtMaxHeight,
        gravelSize: self.gravelSize,
        gravelCount: self.gravelCount,
        gravelMinHeight: self.gravelMinHeight,
        gravelMaxHeight: self.gravelMaxHeight,
        graniteSize: self.graniteSize,
        graniteCount: self.graniteCount,
        graniteMinHeight: self.graniteMinHeight,
        graniteMaxHeight: self.graniteMaxHeight,
        dioriteSize: self.dioriteSize,
        dioriteCount: self.dioriteCount,
        dioriteMinHeight: self.dioriteMinHeight,
        dioriteMaxHeight: self.dioriteMaxHeight,
        andesiteSize: self.andesiteSize,
        andesiteCount: self.andesiteCount,
        andesiteMinHeight: self.andesiteMinHeight,
        andesiteMaxHeight: self.andesiteMaxHeight,
        coalSize: self.coalSize,
        coalCount: self.coalCount,
        coalMinHeight: self.coalMinHeight,
        coalMaxHeight: self.coalMaxHeight,
        ironSize: self.ironSize,
        ironCount: self.ironCount,
        ironMinHeight: self.ironMinHeight,
        ironMaxHeight: self.ironMaxHeight,
        goldSize: self.goldSize,
        goldCount: self.goldCount,
        goldMinHeight: self.goldMinHeight,
        goldMaxHeight: self.goldMaxHeight,
        redstoneSize: self.redstoneSize,
        redstoneCount: self.redstoneCount,
        redstoneMinHeight: self.redstoneMinHeight,
        redstoneMaxHeight: self.redstoneMaxHeight,
        diamondSize: self.diamondSize,
        diamondCount: self.diamondCount,
        diamondMinHeight: self.diamondMinHeight,
        diamondMaxHeight: self.diamondMaxHeight,
        lapisSize: self.lapisSize,
        lapisCount: self.lapisCount,
        lapisCenterHeight: self.lapisCenterHeight,
        lapisSpread: self.lapisSpread,
    } }
}

fn readFloat(value: &Value) -> Result<f32, ()> {
    match value { Value::Number(n) => n.as_f64().map(|v| v as f32).ok_or(()), Value::String(s) => s.parse::<f32>().map_err(|_| ()), _ => Err(()) }
}
fn readInt(value: &Value) -> Result<i32, ()> {
    match value { Value::Number(n) => n.as_i64().and_then(|v| i32::try_from(v).ok()).ok_or(()), Value::String(s) => s.parse::<i32>().map_err(|_| ()), _ => Err(()) }
}
fn readBoolean(value: &Value) -> Result<bool, ()> {
    match value { Value::Bool(v) => Ok(*v), Value::String(s) if s.eq_ignore_ascii_case("true") => Ok(true), Value::String(s) if s.eq_ignore_ascii_case("false") => Ok(false), _ => Err(()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn defaults_match_1122_factory() { let f=Factory::new(); assert_eq!(f.seaLevel,63); assert_eq!(f.fixedBiome,-1); assert_eq!(f.biomeSize,4); assert_eq!(f.riverSize,4); assert!(f.useCaves && f.useVillages && f.field_191076_A); assert_eq!(f.coordinateScale,684.412_f32); }
    #[test] fn json_uses_mansions_key_and_legacy_fixed_biome_remap() { let f=Factory::jsonToFactory(r#"{"useMansions":false,"fixedBiome":8,"biomeSize":6}"#); assert!(!f.field_191076_A); assert_eq!(f.fixedBiome,10); assert_eq!(f.biomeSize,6); }
    #[test] fn malformed_json_returns_defaults_and_field_type_error_stops_later_reads() { assert_eq!(Factory::jsonToFactory("{"),Factory::new()); let f=Factory::jsonToFactory(r#"{"coordinateScale":100.0,"heightScale":{},"seaLevel":22}"#); assert_eq!(f.coordinateScale,100.0); assert_eq!(f.heightScale,684.412); assert_eq!(f.seaLevel,63); }
    #[test] fn build_copies_obfuscated_mansion_and_offset_fields() { let mut f=Factory::new(); f.field_191076_A=false; f.biomeDepthOffset=2.5; let s=f.build(); assert!(!s.field_191077_z); assert_eq!(s.biomeDepthOffSet,2.5); }
}
