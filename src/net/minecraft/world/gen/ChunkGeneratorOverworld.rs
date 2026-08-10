use std::collections::HashMap;

use crate::compat::Java::JavaRandom;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::world::WorldType::WorldType;
use crate::net::minecraft::world::biome::Biome::Biome;
use crate::net::minecraft::world::biome::BiomeHills::BiomeHills;
use crate::net::minecraft::world::biome::BiomeMesa::BiomeMesa;
use crate::net::minecraft::world::biome::BiomeProviderKind::BiomeProviderKind;
use crate::net::minecraft::world::biome::BiomeSavannaMutated::BiomeSavannaMutated;
use crate::net::minecraft::world::biome::BiomeSwamp::BiomeSwamp;
use crate::net::minecraft::world::biome::BiomeTaiga::BiomeTaiga;
use crate::net::minecraft::world::chunk::Chunk::Chunk;
use crate::net::minecraft::world::chunk::ChunkPrimer::ChunkPrimer;
use crate::net::minecraft::world::gen::ChunkGeneratorSettings::{ChunkGeneratorSettings,Factory};
use crate::net::minecraft::world::gen::IChunkGenerator::IChunkGenerator;
use crate::net::minecraft::world::gen::NoiseGeneratorOctaves::NoiseGeneratorOctaves;
use crate::net::minecraft::world::gen::NoiseGeneratorPerlin::NoiseGeneratorPerlin;
use crate::net::minecraft::world::gen::MapGenCaves::MapGenCaves;
use crate::net::minecraft::world::gen::MapGenRavine::MapGenRavine;

/// MCP 1.12.2 `ChunkGeneratorOverworld` terrain tranche.
///
/// The base density field, biome blending, source biome surfaces, caves,
/// ravines and Chunk construction follow the 1.12.2 generator path. Structure
/// MapGen classes and population/decorator hooks remain explicit follow-on
/// responsibilities; this generator never substitutes flat or synthetic
/// terrain when those are absent.
#[derive(Debug)]
pub struct ChunkGeneratorOverworld {
    rand: JavaRandom,
    minLimitPerlinNoise: NoiseGeneratorOctaves,
    maxLimitPerlinNoise: NoiseGeneratorOctaves,
    mainPerlinNoise: NoiseGeneratorOctaves,
    surfaceNoise: NoiseGeneratorPerlin,
    pub scaleNoise: NoiseGeneratorOctaves,
    pub depthNoise: NoiseGeneratorOctaves,
    pub forestNoise: NoiseGeneratorOctaves,
    seed:i64,
    mapFeaturesEnabled:bool,
    terrainType:WorldType,
    biomeProvider:BiomeProviderKind,
    heightMap:Vec<f64>,
    biomeWeights:[f32;25],
    settings:ChunkGeneratorSettings,
    oceanBlock:IBlockState,
    depthBuffer:Option<Vec<f64>>,
    biomesForGeneration:Option<Vec<Biome>>,
    mainNoiseRegion:Option<Vec<f64>>,
    minLimitRegion:Option<Vec<f64>>,
    maxLimitRegion:Option<Vec<f64>>,
    depthRegion:Option<Vec<f64>>,
    mesaStates:HashMap<u8,BiomeMesa>,
    currentBiomeTop:[IBlockState;256],
    caveGenerator:MapGenCaves,
    ravineGenerator:MapGenRavine,
}
impl ChunkGeneratorOverworld {
    pub fn new(seed:i64,mapFeaturesEnabled:bool,terrainType:WorldType,generatorOptions:&str,biomeProvider:BiomeProviderKind)->Self{
        let mut rand=JavaRandom::new(seed);
        // Constructor allocation order is protocol-visible through Java Random
        // consumption and must match MCP exactly.
        let minLimitPerlinNoise=NoiseGeneratorOctaves::new(&mut rand,16);
        let maxLimitPerlinNoise=NoiseGeneratorOctaves::new(&mut rand,16);
        let mainPerlinNoise=NoiseGeneratorOctaves::new(&mut rand,8);
        let surfaceNoise=NoiseGeneratorPerlin::new(&mut rand,4);
        let scaleNoise=NoiseGeneratorOctaves::new(&mut rand,10);
        let depthNoise=NoiseGeneratorOctaves::new(&mut rand,16);
        let forestNoise=NoiseGeneratorOctaves::new(&mut rand,8);
        let settings=Factory::jsonToFactory(generatorOptions).build();
        let oceanBlock=IBlockState::fromGlobalStateId((if settings.useLavaOceans{11}else{9})<<4);
        let mut biomeWeights=[0.0_f32;25];
        for i in -2_i32..=2{for j in -2_i32..=2{
            biomeWeights[(i+2+(j+2)*5)as usize]=10.0_f32/(((i*i+j*j)as f32+0.2_f32).sqrt());
        }}
        let currentBiomeTop=std::array::from_fn(|id|Biome::getBiome(id as u8).terrainTopFiller().0);
        Self{rand,minLimitPerlinNoise,maxLimitPerlinNoise,mainPerlinNoise,surfaceNoise,scaleNoise,depthNoise,forestNoise,seed,mapFeaturesEnabled,terrainType,biomeProvider,heightMap:vec![0.0;825],biomeWeights,settings,oceanBlock,depthBuffer:None,biomesForGeneration:None,mainNoiseRegion:None,minLimitRegion:None,maxLimitRegion:None,depthRegion:None,mesaStates:HashMap::new(),currentBiomeTop,caveGenerator:MapGenCaves::new(),ravineGenerator:MapGenRavine::new()}
    }

    pub fn settings(&self)->&ChunkGeneratorSettings{&self.settings}
    pub fn hasUnportedPostTerrainFeatures(&self)->bool{
        self.settings.useDungeons||self.settings.useWaterLakes||self.settings.useLavaLakes||
            (self.mapFeaturesEnabled&&(self.settings.useStrongholds||self.settings.useVillages||self.settings.useMineShafts||self.settings.useTemples||self.settings.useMonuments||self.settings.field_191077_z))
    }

    #[inline] fn clamped_lerp(lower:f64,upper:f64,slide:f64)->f64{if slide<0.0{lower}else if slide>1.0{upper}else{lower+(upper-lower)*slide}}

    fn generateHeightmap(&mut self,x:i32,y:i32,z:i32){
        self.depthRegion=Some(self.depthNoise.generateNoiseOctaves2D(self.depthRegion.take(),x,z,5,5,self.settings.depthNoiseScaleX as f64,self.settings.depthNoiseScaleZ as f64,self.settings.depthNoiseScaleExponent as f64));
        let f=self.settings.coordinateScale;
        let f1=self.settings.heightScale;
        self.mainNoiseRegion=Some(self.mainPerlinNoise.generateNoiseOctaves(self.mainNoiseRegion.take(),x,y,z,5,33,5,(f/self.settings.mainNoiseScaleX)as f64,(f1/self.settings.mainNoiseScaleY)as f64,(f/self.settings.mainNoiseScaleZ)as f64));
        self.minLimitRegion=Some(self.minLimitPerlinNoise.generateNoiseOctaves(self.minLimitRegion.take(),x,y,z,5,33,5,f as f64,f1 as f64,f as f64));
        self.maxLimitRegion=Some(self.maxLimitPerlinNoise.generateNoiseOctaves(self.maxLimitRegion.take(),x,y,z,5,33,5,f as f64,f1 as f64,f as f64));
        let depth=self.depthRegion.as_ref().unwrap();let main=self.mainNoiseRegion.as_ref().unwrap();let min=self.minLimitRegion.as_ref().unwrap();let max=self.maxLimitRegion.as_ref().unwrap();
        let biomes=self.biomesForGeneration.as_ref().expect("10x10 generation biomes initialized");
        let mut i=0usize;let mut j=0usize;
        for k in 0..5usize{for l in 0..5usize{
            let mut f2=0.0_f32;let mut f3=0.0_f32;let mut f4=0.0_f32;
            let biome=biomes[k+2+(l+2)*10];
            for j1 in -2_i32..=2{for k1 in -2_i32..=2{
                let biome1=biomes[(k as i32+j1+2+(l as i32+k1+2)*10)as usize];
                let mut f5=self.settings.biomeDepthOffSet+biome1.getBaseHeight()*self.settings.biomeDepthWeight;
                let mut f6=self.settings.biomeScaleOffset+biome1.getHeightVariation()*self.settings.biomeScaleWeight;
                if self.terrainType==WorldType::Amplified&&f5>0.0{f5=1.0+f5*2.0;f6=1.0+f6*4.0;}
                let mut f7=self.biomeWeights[(j1+2+(k1+2)*5)as usize]/(f5+2.0);
                if biome1.getBaseHeight()>biome.getBaseHeight(){f7/=2.0;}
                f2+=f6*f7;f3+=f5*f7;f4+=f7;
            }}
            f2/=f4;f3/=f4;f2=f2*0.9+0.1;f3=(f3*4.0-1.0)/8.0;
            let mut d7=depth[j]/8000.0;
            if d7<0.0{d7=-d7*0.3;}
            d7=d7*3.0-2.0;
            if d7<0.0{d7/=2.0;if d7< -1.0{d7=-1.0;}d7/=1.4;d7/=2.0;}else{if d7>1.0{d7=1.0;}d7/=8.0;}
            j+=1;
            let mut d8=f3 as f64;let d9=f2 as f64;d8+=d7*0.2;d8=d8*self.settings.baseSize as f64/8.0;let d0=self.settings.baseSize as f64+d8*4.0;
            for l1 in 0..33usize{
                let mut d1=(l1 as f64-d0)*self.settings.stretchY as f64*128.0/256.0/d9;if d1<0.0{d1*=4.0;}
                let d2=min[i]/self.settings.lowerLimitScale as f64;let d3=max[i]/self.settings.upperLimitScale as f64;let d4=(main[i]/10.0+1.0)/2.0;
                let mut d5=Self::clamped_lerp(d2,d3,d4)-d1;
                if l1>29{let d6=(l1 as f32-29.0)/3.0;d5=d5*(1.0-d6 as f64)+-10.0*d6 as f64;}
                self.heightMap[i]=d5;i+=1;
            }
        }}
    }

    pub fn setBlocksInChunk(&mut self,x:i32,z:i32,primer:&mut ChunkPrimer){
        self.biomesForGeneration=Some(self.biomeProvider.getBiomesForGeneration(self.biomesForGeneration.take(),x*4-2,z*4-2,10,10));
        self.generateHeightmap(x*4,0,z*4);
        for i in 0..4usize{let j=i*5;let k=(i+1)*5;for l in 0..4usize{
            let i1=(j+l)*33;let j1=(j+l+1)*33;let k1=(k+l)*33;let l1=(k+l+1)*33;
            for i2 in 0..32usize{
                let mut d1=self.heightMap[i1+i2];let mut d2=self.heightMap[j1+i2];let mut d3=self.heightMap[k1+i2];let mut d4=self.heightMap[l1+i2];
                let d5=(self.heightMap[i1+i2+1]-d1)*0.125;let d6=(self.heightMap[j1+i2+1]-d2)*0.125;let d7=(self.heightMap[k1+i2+1]-d3)*0.125;let d8=(self.heightMap[l1+i2+1]-d4)*0.125;
                for j2 in 0..8usize{
                    let mut d10=d1;let mut d11=d2;let d12=(d3-d1)*0.25;let d13=(d4-d2)*0.25;
                    for k2 in 0..4usize{
                        let d16=(d11-d10)*0.25;let mut value=d10-d16;
                        for l2 in 0..4usize{
                            value+=d16;let yy=i2*8+j2;
                            if value>0.0{primer.setBlockState(i*4+k2,yy,l*4+l2,IBlockState::fromGlobalStateId(1<<4));}
                            else if (yy as i32)<self.settings.seaLevel{primer.setBlockState(i*4+k2,yy,l*4+l2,self.oceanBlock);}
                        }
                        d10+=d12;d11+=d13;
                    }
                    d1+=d5;d2+=d6;d3+=d7;d4+=d8;
                }
            }
        }}
    }

    fn generateBiomeTerrainColumn(&mut self,biome:Biome,primer:&mut ChunkPrimer,world_x:i32,world_z:i32,noise:f64){
        let id=biome.getId() as usize;
        match biome.getId(){
            3|20|34|131|162=>{
                let grass=IBlockState::fromGlobalStateId(2<<4);let gravel=IBlockState::fromGlobalStateId(13<<4);let stone=IBlockState::fromGlobalStateId(1<<4);
                let mutated=matches!(biome.getId(),131|162);let extra=matches!(biome.getId(),20|34);
                self.currentBiomeTop[id]=if (noise< -1.0||noise>2.0)&&mutated{gravel}else if noise>1.0&&!extra{stone}else{grass};
                BiomeHills::genTerrainBlocks(biome,self.settings.seaLevel,&mut self.rand,primer,world_x,world_z,noise)
            }
            5|19|30|31|32|33|133|158|160|161=>{
                let grass=IBlockState::fromGlobalStateId(2<<4);let coarse=IBlockState::fromGlobalStateId((3<<4)|1);let podzol=IBlockState::fromGlobalStateId((3<<4)|2);
                self.currentBiomeTop[id]=if matches!(biome.getId(),32|33|160|161){if noise>1.75{coarse}else if noise>-0.95{podzol}else{grass}}else{biome.terrainTopFiller().0};
                BiomeTaiga::genTerrainBlocks(biome,self.settings.seaLevel,&mut self.rand,primer,world_x,world_z,noise)
            }
            6|134=>{self.currentBiomeTop[id]=biome.terrainTopFiller().0;BiomeSwamp::genTerrainBlocks(biome,self.settings.seaLevel,&mut self.rand,primer,world_x,world_z,noise)}
            163|164=>{
                let grass=IBlockState::fromGlobalStateId(2<<4);let coarse=IBlockState::fromGlobalStateId((3<<4)|1);let stone=IBlockState::fromGlobalStateId(1<<4);
                self.currentBiomeTop[id]=if noise>1.75{stone}else if noise>-0.5{coarse}else{grass};
                BiomeSavannaMutated::genTerrainBlocks(biome,self.settings.seaLevel,&mut self.rand,primer,world_x,world_z,noise)
            }
            37|38|39|165|166|167=>{
                self.currentBiomeTop[id]=biome.terrainTopFiller().0;
                let state=self.mesaStates.entry(biome.getId()).or_insert_with(||BiomeMesa::forBiome(biome));
                state.genTerrainBlocks(biome,self.seed,self.settings.seaLevel,&mut self.rand,primer,world_x,world_z,noise);
            }
            _=>{let(top,filler)=biome.terrainTopFiller();self.currentBiomeTop[id]=top;biome.generateBiomeTerrain(self.settings.seaLevel,&mut self.rand,primer,world_x,world_z,noise,top,filler);}
        }
    }

    pub fn replaceBiomeBlocks(&mut self,x:i32,z:i32,primer:&mut ChunkPrimer,biomes:&[Biome]){
        self.depthBuffer=Some(self.surfaceNoise.getRegion(self.depthBuffer.take(),(x*16)as f64,(z*16)as f64,16,16,0.0625,0.0625,1.0));
        let noise=self.depthBuffer.as_ref().unwrap().clone();
        for i in 0..16usize{for j in 0..16usize{
            let biome=biomes[j+i*16];let d=noise[j+i*16];
            self.generateBiomeTerrainColumn(biome,primer,x*16+i as i32,z*16+j as i32,d);
        }}
    }
}

impl IChunkGenerator for ChunkGeneratorOverworld{
    fn provideChunk(&mut self,x:i32,z:i32)->Result<Chunk,String>{
        self.rand.set_seed((x as i64).wrapping_mul(341873128712_i64).wrapping_add((z as i64).wrapping_mul(132897987541_i64)));
        let mut primer=ChunkPrimer::new();
        self.setBlocksInChunk(x,z,&mut primer);
        let biomes=self.biomeProvider.getBiomes(self.biomesForGeneration.take(),x*16,z*16,16,16);
        self.replaceBiomeBlocks(x,z,&mut primer,&biomes);
        self.biomesForGeneration=Some(biomes.clone());
        if self.settings.useCaves { self.caveGenerator.generate(self.seed,&self.biomeProvider,&self.currentBiomeTop,x,z,&mut primer); }
        if self.settings.useRavines { self.ravineGenerator.generate(self.seed,&self.biomeProvider,&self.currentBiomeTop,x,z,&mut primer); }
        // MapGenStructure classes remain explicit next dependencies. No fake
        // village/mineshaft/stronghold/temple/monument/mansion is inserted.
        let mut chunk=Chunk::fromPrimer(&primer,x,z,true)?;
        let biome_bytes:Vec<u8>=biomes.iter().map(|b|b.getId()).collect();
        chunk.setBiomeArray(&biome_bytes);
        chunk.generateSkylightMap(true);
        Ok(chunk)
    }
    fn populate(&mut self,_x:i32,_z:i32)->Result<(),String>{
        if self.hasUnportedPostTerrainFeatures(){Err("ChunkGeneratorOverworld population/MapGen requested before the corresponding 1.12.2 structure/lake/dungeon/biome-decorator classes are ported".to_owned())}else{Ok(())}
    }
    fn generatorName(&self)->&'static str{"overworld"}
    fn seaLevelOverride(&self)->Option<i32>{Some(self.settings.seaLevel)}
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::net::minecraft::world::biome::BiomeProvider::BiomeProvider;
    #[test]
    fn default_overworld_terrain_is_not_flat_fallback(){
        let provider=BiomeProviderKind::Layered(BiomeProvider::new(12345,WorldType::Default,""));
        let mut generator=ChunkGeneratorOverworld::new(12345,true,WorldType::Default,"",provider);
        let chunk=generator.provideChunk(0,0).unwrap();
        let heights:Vec<i32>=(0..16).map(|x|chunk.getHeightValue(x,8)).collect();
        assert!(heights.iter().any(|h|*h!=heights[0]));
        assert!(heights.iter().any(|h|*h>4));
    }
    #[test]
    fn customized_sea_level_is_owned_by_settings(){
        let provider=BiomeProviderKind::Layered(BiomeProvider::new(1,WorldType::Customized,r#"{"seaLevel":40}"#));
        let generator=ChunkGeneratorOverworld::new(1,false,WorldType::Customized,r#"{"seaLevel":40}"#,provider);
        assert_eq!(generator.seaLevelOverride(),Some(40));
    }
}
