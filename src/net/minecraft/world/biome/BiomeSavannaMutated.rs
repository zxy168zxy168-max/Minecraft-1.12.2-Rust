use crate::compat::Java::JavaRandom;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::world::biome::Biome::Biome;
use crate::net::minecraft::world::chunk::ChunkPrimer::ChunkPrimer;

/// MCP 1.12.2 `BiomeSavannaMutated#genTerrainBlocks`.
pub struct BiomeSavannaMutated;
impl BiomeSavannaMutated {
    #[allow(non_snake_case)]
    pub fn genTerrainBlocks(
        biome: Biome,
        seaLevel: i32,
        rand: &mut JavaRandom,
        primer: &mut ChunkPrimer,
        x: i32,
        z: i32,
        noiseVal: f64,
    ) {
        let grass = IBlockState::fromGlobalStateId(2 << 4);
        let dirt = IBlockState::fromGlobalStateId(3 << 4);
        let coarse = IBlockState::fromGlobalStateId((3 << 4) | 1);
        let stone = IBlockState::fromGlobalStateId(1 << 4);
        let (top, filler) = if noiseVal > 1.75 {
            (stone, stone)
        } else if noiseVal > -0.5 {
            (coarse, dirt)
        } else {
            (grass, dirt)
        };
        biome.generateBiomeTerrain(seaLevel, rand, primer, x, z, noiseVal, top, filler);
    }
}
