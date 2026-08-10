use crate::compat::Java::JavaRandom;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::world::biome::Biome::Biome;
use crate::net::minecraft::world::chunk::ChunkPrimer::ChunkPrimer;
use crate::net::minecraft::world::gen::NoiseGeneratorPerlin::NoiseGeneratorPerlin;

/// Stateful MCP 1.12.2 `BiomeMesa` terrain generator. Rust keeps the mutable
/// source caches with the owning `ChunkGeneratorOverworld` instead of global
/// singleton biome instances so independent integrated worlds cannot race.
#[derive(Debug,Clone)]
pub struct BiomeMesa{
    clayBands:Option<[IBlockState;64]>,
    worldSeed:i64,
    pillarNoise:Option<NoiseGeneratorPerlin>,
    pillarRoofNoise:Option<NoiseGeneratorPerlin>,
    clayBandsOffsetNoise:Option<NoiseGeneratorPerlin>,
    brycePillars:bool,
    hasForest:bool,
}
impl BiomeMesa{
    pub fn forBiome(biome:Biome)->Self{Self{clayBands:None,worldSeed:0,pillarNoise:None,pillarRoofNoise:None,clayBandsOffsetNoise:None,brycePillars:biome.getId()==165,hasForest:matches!(biome.getId(),38|166)}}

    fn generateBands(&mut self,seed:i64){
        const HARD:IBlockState=IBlockState::fromGlobalStateId(172<<4);
        const STAINED:i32=159<<4;
        const ORANGE:IBlockState=IBlockState::fromGlobalStateId(STAINED|1);
        let mut bands=[HARD;64];
        let mut random=JavaRandom::new(seed);
        self.clayBandsOffsetNoise=Some(NoiseGeneratorPerlin::new(&mut random,1));
        let mut l1=0_i32;
        while l1<64{
            l1+=random.next_i32_bound(5)+1;
            if l1<64{bands[l1 as usize]=ORANGE;}
            l1+=1; // Java for-loop post increment after body
        }
        let i2=random.next_i32_bound(4)+2;
        for _ in 0..i2{
            let width=random.next_i32_bound(3)+1;let start=random.next_i32_bound(64);
            for off in 0..width{if start+off<64{bands[(start+off)as usize]=IBlockState::fromGlobalStateId(STAINED|4);}}
        }
        let j2=random.next_i32_bound(4)+2;
        for _ in 0..j2{
            let width=random.next_i32_bound(3)+2;let start=random.next_i32_bound(64);
            for off in 0..width{if start+off<64{bands[(start+off)as usize]=IBlockState::fromGlobalStateId(STAINED|12);}}
        }
        let l2=random.next_i32_bound(4)+2;
        for _ in 0..l2{
            let width=random.next_i32_bound(3)+1;let start=random.next_i32_bound(64);
            for off in 0..width{if start+off<64{bands[(start+off)as usize]=IBlockState::fromGlobalStateId(STAINED|14);}}
        }
        let k3=random.next_i32_bound(3)+3;
        let mut pos=0_i32;
        for _ in 0..k3{
            pos+=random.next_i32_bound(16)+4;
            if pos<64{
                bands[pos as usize]=IBlockState::fromGlobalStateId(STAINED);
                if pos>1&&random.next_bool(){bands[(pos-1)as usize]=IBlockState::fromGlobalStateId(STAINED|8);}
                if pos<63&&random.next_bool(){bands[(pos+1)as usize]=IBlockState::fromGlobalStateId(STAINED|8);}
            }
        }
        self.clayBands=Some(bands);
    }

    fn getBand(&self,x:i32,y:i32,_z:i32)->IBlockState{
        let noise=self.clayBandsOffsetNoise.as_ref().expect("mesa clay offset initialized").getValue(x as f64/512.0,x as f64/512.0)*2.0;
        // Java Math.round(double) == floor(value + 0.5).
        let offset=(noise+0.5).floor() as i32;
        self.clayBands.as_ref().expect("mesa bands initialized")[((y+offset+64)%64)as usize]
    }

    #[allow(non_snake_case)]
    pub fn genTerrainBlocks(&mut self,biome:Biome,worldSeed:i64,seaLevel:i32,rand:&mut JavaRandom,primer:&mut ChunkPrimer,x:i32,z:i32,noiseVal:f64){
        if self.clayBands.is_none()||self.worldSeed!=worldSeed{self.generateBands(worldSeed);}
        if self.pillarNoise.is_none()||self.pillarRoofNoise.is_none()||self.worldSeed!=worldSeed{
            // Source creates these from the previous cached worldSeed and only
            // assigns worldSeed afterwards. Preserve that ordering exactly.
            let mut random=JavaRandom::new(self.worldSeed);
            self.pillarNoise=Some(NoiseGeneratorPerlin::new(&mut random,4));
            self.pillarRoofNoise=Some(NoiseGeneratorPerlin::new(&mut random,1));
        }
        self.worldSeed=worldSeed;
        let mut pillar_height=0.0_f64;
        if self.brycePillars{
            let i=(x&-16)+(z&15);let j=(z&-16)+(x&15);
            let d0=noiseVal.abs().min(self.pillarNoise.as_ref().unwrap().getValue(i as f64*0.25,j as f64*0.25));
            if d0>0.0{
                let d2=self.pillarRoofNoise.as_ref().unwrap().getValue(i as f64*0.001953125,j as f64*0.001953125).abs();
                pillar_height=d0*d0*2.5;
                let cap=(d2*50.0).ceil()+14.0;
                if pillar_height>cap{pillar_height=cap;}
                pillar_height+=64.0;
            }
        }
        const AIR:IBlockState=IBlockState::fromGlobalStateId(0);
        const STONE:IBlockState=IBlockState::fromGlobalStateId(1<<4);
        const BEDROCK:IBlockState=IBlockState::fromGlobalStateId(7<<4);
        const WATER:IBlockState=IBlockState::fromGlobalStateId(9<<4);
        const GRASS:IBlockState=IBlockState::fromGlobalStateId(2<<4);
        const COARSE_DIRT:IBlockState=IBlockState::fromGlobalStateId((3<<4)|1);
        const HARD:IBlockState=IBlockState::fromGlobalStateId(172<<4);
        const STAINED:IBlockState=IBlockState::fromGlobalStateId(159<<4);
        const ORANGE:IBlockState=IBlockState::fromGlobalStateId((159<<4)|1);
        let (top,filler)=biome.terrainTopFiller();
        let local_z=(x&15)as usize;let local_x=(z&15)as usize;
        let mut current=STAINED;let mut filler_state=filler;
        let thickness=(noiseVal/3.0+3.0+rand.next_f64()*0.25)as i32;
        let flag=(noiseVal/3.0*std::f64::consts::PI).cos()>0.0;
        let mut remaining=-1_i32;let mut orange_cap=false;let mut stone_count=0_i32;
        for y in (0..=255_i32).rev(){
            if primer.getBlockState(local_x,y as usize,local_z).isAir() && y<(pillar_height as i32){primer.setBlockState(local_x,y as usize,local_z,STONE);}
            if y<=rand.next_i32_bound(5){primer.setBlockState(local_x,y as usize,local_z,BEDROCK);continue;}
            if stone_count>=15&&!self.brycePillars{continue;}
            let state=primer.getBlockState(local_x,y as usize,local_z);
            if state.isAir(){remaining=-1;continue;}
            if state.getBlockId()!=1{continue;}
            if remaining==-1{
                orange_cap=false;
                if thickness<=0{current=AIR;filler_state=STONE;}
                else if y>=seaLevel-4&&y<=seaLevel+1{current=STAINED;filler_state=filler;}
                if y<seaLevel&&current.isAir(){current=WATER;}
                remaining=thickness+(y-seaLevel).max(0);
                if y>=seaLevel-1{
                    if self.hasForest&&y>86+thickness*2{primer.setBlockState(local_x,y as usize,local_z,if flag{COARSE_DIRT}else{GRASS});}
                    else if y>seaLevel+3+thickness{
                        let surface=if (64..=127).contains(&y){if flag{HARD}else{self.getBand(x,y,z)}}else{ORANGE};
                        primer.setBlockState(local_x,y as usize,local_z,surface);
                    }else{primer.setBlockState(local_x,y as usize,local_z,top);orange_cap=true;}
                }else{
                    primer.setBlockState(local_x,y as usize,local_z,filler_state);
                    if filler_state.getBlockId()==159{primer.setBlockState(local_x,y as usize,local_z,ORANGE);}
                }
            }else if remaining>0{
                remaining-=1;
                primer.setBlockState(local_x,y as usize,local_z,if orange_cap{ORANGE}else{self.getBand(x,y,z)});
            }
            stone_count+=1;
        }
    }
}
