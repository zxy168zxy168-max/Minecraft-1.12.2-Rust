use crate::compat::Java::JavaRandom;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::world::biome::Biome::Biome;
use crate::net::minecraft::world::chunk::ChunkPrimer::ChunkPrimer;

/// MCP 1.12.2 `BiomeHills#genTerrainBlocks` terrain responsibility.
pub struct BiomeHills;
impl BiomeHills {
    #[allow(non_snake_case)]
    pub fn genTerrainBlocks(biome:Biome, seaLevel:i32, rand:&mut JavaRandom, primer:&mut ChunkPrimer, x:i32, z:i32, noiseVal:f64){
        let grass=IBlockState::fromGlobalStateId(2<<4);
        let dirt=IBlockState::fromGlobalStateId(3<<4);
        let gravel=IBlockState::fromGlobalStateId(13<<4);
        let stone=IBlockState::fromGlobalStateId(1<<4);
        let mutated=matches!(biome.getId(),131|162);
        let extra_trees=matches!(biome.getId(),20|34);
        let (top,filler)=if (noiseVal < -1.0 || noiseVal > 2.0) && mutated {(gravel,gravel)}
            else if noiseVal > 1.0 && !extra_trees {(stone,stone)} else {(grass,dirt)};
        biome.generateBiomeTerrain(seaLevel,rand,primer,x,z,noiseVal,top,filler);
    }
}
