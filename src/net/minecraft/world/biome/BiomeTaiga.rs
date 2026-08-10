use crate::compat::Java::JavaRandom;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::world::biome::Biome::Biome;
use crate::net::minecraft::world::chunk::ChunkPrimer::ChunkPrimer;

/// MCP 1.12.2 `BiomeTaiga#genTerrainBlocks` terrain responsibility.
pub struct BiomeTaiga;
impl BiomeTaiga {
    #[allow(non_snake_case)]
    pub fn genTerrainBlocks(biome:Biome, seaLevel:i32, rand:&mut JavaRandom, primer:&mut ChunkPrimer, x:i32, z:i32, noiseVal:f64){
        let grass=IBlockState::fromGlobalStateId(2<<4);
        let dirt=IBlockState::fromGlobalStateId(3<<4);
        let coarse=IBlockState::fromGlobalStateId((3<<4)|1);
        let podzol=IBlockState::fromGlobalStateId((3<<4)|2);
        let mega=matches!(biome.getId(),32|33|160|161);
        let (top,filler)=if mega {
            let top=if noiseVal>1.75{coarse}else if noiseVal>-0.95{podzol}else{grass};
            (top,dirt)
        }else{biome.terrainTopFiller()};
        biome.generateBiomeTerrain(seaLevel,rand,primer,x,z,noiseVal,top,filler);
    }
}
