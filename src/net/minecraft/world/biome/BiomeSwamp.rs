use crate::compat::Java::JavaRandom;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::world::biome::Biome::{grass_color_noise, Biome};
use crate::net::minecraft::world::chunk::ChunkPrimer::ChunkPrimer;

/// MCP 1.12.2 `BiomeSwamp#genTerrainBlocks`.
pub struct BiomeSwamp;
impl BiomeSwamp {
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
        let d0 = grass_color_noise().getValue(x as f64 * 0.25, z as f64 * 0.25);
        if d0 > 0.0 {
            let local_z = (x & 15) as usize;
            let local_x = (z & 15) as usize;
            for y in (0..=255usize).rev() {
                let state = primer.getBlockState(local_x, y, local_z);
                if !state.isAir() {
                    if y == 62 && state.getBlockId() != 9 {
                        primer.setBlockState(
                            local_x,
                            y,
                            local_z,
                            IBlockState::fromGlobalStateId(9 << 4),
                        );
                        if d0 < 0.12 {
                            primer.setBlockState(
                                local_x,
                                y + 1,
                                local_z,
                                IBlockState::fromGlobalStateId(111 << 4),
                            );
                        }
                    }
                    break;
                }
            }
        }
        let (top, filler) = biome.terrainTopFiller();
        biome.generateBiomeTerrain(seaLevel, rand, primer, x, z, noiseVal, top, filler);
    }
}
