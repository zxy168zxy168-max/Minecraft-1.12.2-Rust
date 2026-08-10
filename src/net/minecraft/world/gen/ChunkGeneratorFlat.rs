use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::world::chunk::Chunk::Chunk;
use crate::net::minecraft::world::chunk::ChunkPrimer::ChunkPrimer;
use crate::net::minecraft::world::gen::FlatGeneratorInfo::FlatGeneratorInfo;
use crate::net::minecraft::world::gen::IChunkGenerator::IChunkGenerator;

/// MCP 1.12.2 `ChunkGeneratorFlat` terrain tranche.
///
/// The layer cache, sea-level calculation, biome fill and Chunk construction
/// are source-equivalent. MapGenStructure/WorldGenLakes/dungeon/decorate hooks
/// stay explicitly pending until those concrete generator classes are ported;
/// they are never replaced with fabricated structures.
#[derive(Debug, Clone)]
pub struct ChunkGeneratorFlat {
    seed: i64,
    cachedBlockIDs: [Option<IBlockState>; 256],
    flatWorldGenInfo: FlatGeneratorInfo,
    hasDecoration: bool,
    hasDungeons: bool,
    mapFeaturesEnabled: bool,
    seaLevel: i32,
}

impl ChunkGeneratorFlat {
    pub fn new(seed: i64, generateStructures: bool, flatGeneratorSettings: &str) -> Self {
        let flatWorldGenInfo = FlatGeneratorInfo::createFlatGeneratorFromString(flatGeneratorSettings);
        let mut cachedBlockIDs = [None; 256];
        let mut seaLevel = 0;
        let mut pendingAir = 0;
        let mut allAir = true;
        for layer in flatWorldGenInfo.getFlatLayers() {
            let minY = layer.getMinY().clamp(0, 256) as usize;
            let maxY = (layer.getMinY() + layer.getLayerCount()).clamp(0, 256) as usize;
            let state = layer.getLayerMaterial();
            for y in minY..maxY {
                if !state.isAir() {
                    allAir = false;
                    cachedBlockIDs[y] = Some(state);
                }
            }
            if state.isAir() {
                pendingAir += layer.getLayerCount();
            } else {
                seaLevel += layer.getLayerCount() + pendingAir;
                pendingAir = 0;
            }
        }
        let hasDecoration = if allAir && flatWorldGenInfo.getBiome() != 127 {
            false
        } else {
            flatWorldGenInfo.getWorldFeatures().contains_key("decoration")
        };
        let hasDungeons = flatWorldGenInfo.getWorldFeatures().contains_key("dungeon");
        let generator=Self { seed, cachedBlockIDs, flatWorldGenInfo, hasDecoration, hasDungeons, mapFeaturesEnabled: generateStructures, seaLevel };
        if generator.hasUnportedPopulationFeatures() {
            log::warn!("ChunkGeneratorFlat terrain is active, but referenced vanilla population/structure generators are still pending; no substitute structures will be fabricated");
        }
        generator
    }

    pub const fn getSeaLevel(&self) -> i32 { self.seaLevel }
    pub const fn getSeed(&self) -> i64 { self.seed }
    pub const fn hasDecoration(&self) -> bool { self.hasDecoration }
    pub const fn hasDungeons(&self) -> bool { self.hasDungeons }
    pub fn flatWorldGenInfo(&self) -> &FlatGeneratorInfo { &self.flatWorldGenInfo }
    pub fn hasUnportedPopulationFeatures(&self) -> bool {
        let features = self.flatWorldGenInfo.getWorldFeatures();
        self.hasDecoration || self.hasDungeons || features.contains_key("lake") || features.contains_key("lava_lake")
            || (self.mapFeaturesEnabled && features.keys().any(|name| matches!(name.as_str(), "village" | "biome_1" | "mineshaft" | "stronghold" | "oceanmonument")))
    }

    fn buildPrimer(&self) -> ChunkPrimer {
        let mut primer = ChunkPrimer::new();
        for y in 0..256 {
            let Some(state) = self.cachedBlockIDs[y] else { continue; };
            for x in 0..16 {
                for z in 0..16 { primer.setBlockState(x, y, z, state); }
            }
        }
        primer
    }
}

impl IChunkGenerator for ChunkGeneratorFlat {
    fn provideChunk(&mut self, x: i32, z: i32) -> Result<Chunk, String> {
        let primer = self.buildPrimer();
        let mut chunk = Chunk::fromPrimer(&primer, x, z, true)?;
        let biome = self.flatWorldGenInfo.getBiome().clamp(0, 255) as u8;
        chunk.setBiomeArray(&[biome; 256]);
        chunk.generateSkylightMap(true);
        Ok(chunk)
    }

    fn populate(&mut self, _x: i32, _z: i32) -> Result<(), String> {
        if self.hasUnportedPopulationFeatures() {
            return Err("ChunkGeneratorFlat population requested before the referenced 1.12.2 MapGen/WorldGen feature classes are ported".to_owned());
        }
        Ok(())
    }

    fn generatorName(&self) -> &'static str { "flat" }
    fn seaLevelOverride(&self) -> Option<i32> { Some(self.seaLevel) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_flat_chunk_has_bedrock_dirt_dirt_grass_and_plains_biome() {
        let mut generator = ChunkGeneratorFlat::new(123, true, "");
        assert_eq!(generator.getSeaLevel(), 4);
        let chunk = generator.provideChunk(2, -3).unwrap();
        assert_eq!(chunk.getBlockState(0, 0, 0).getBlockId(), 7);
        assert_eq!(chunk.getBlockState(0, 1, 0).getBlockId(), 3);
        assert_eq!(chunk.getBlockState(0, 2, 0).getBlockId(), 3);
        assert_eq!(chunk.getBlockState(0, 3, 0).getBlockId(), 2);
        assert!(chunk.getBlockState(0, 4, 0).isAir());
        assert_eq!(chunk.getBiomeArray()[0], 1);
        assert_eq!(chunk.getHeightValue(0, 0), 4);
    }
}
