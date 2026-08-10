use super::GenLayer::{isBiomeOceanic,GenLayer,GenLayerSeed,Layer};
use crate::net::minecraft::world::WorldType::WorldType;
use crate::net::minecraft::world::gen::ChunkGeneratorSettings::ChunkGeneratorSettings;
#[derive(Debug)] pub struct GenLayerBiome{seed:GenLayerSeed,parent:Layer,warm:[i32;6],settings:Option<ChunkGeneratorSettings>}
impl GenLayerBiome{
 pub fn new(seed:i64,parent:Layer,world_type:WorldType,settings:Option<ChunkGeneratorSettings>)->Self{let(warm,settings)=if world_type==WorldType::Default11{([2,4,3,6,1,5],None)}else{([2,2,2,35,35,1],settings)};Self{seed:GenLayerSeed::new(seed),parent,warm,settings}}
}
impl GenLayer for GenLayerBiome{
 fn initWorldGenSeed(&mut self,s:i64){self.parent.lock().unwrap().initWorldGenSeed(s);self.seed.initWorldGenSeed(s)}
 fn getInts(&mut self,x:i32,z:i32,w:i32,h:i32)->Vec<i32>{let p=self.parent.lock().unwrap().getInts(x,z,w,h);let mut o=vec![0;(w*h)as usize];let medium=[4,29,3,1,27,6];let cold=[4,3,5,1];let ice=[12,12,12,30];for yy in 0..h{for xx in 0..w{self.seed.initChunkSeed((xx+x)as i64,(yy+z)as i64);let mut k=p[(xx+yy*w)as usize];let special=(k&3840)>>8;k&=-3841;let v=if self.settings.as_ref().is_some_and(|s|s.fixedBiome>=0){self.settings.as_ref().unwrap().fixedBiome}else if isBiomeOceanic(k)||k==14{k}else{match k{1=>if special>0{if self.seed.nextInt(3)==0{39}else{38}}else{self.warm[self.seed.nextInt(self.warm.len()as i32)as usize]},2=>if special>0{21}else{medium[self.seed.nextInt(medium.len()as i32)as usize]},3=>if special>0{32}else{cold[self.seed.nextInt(cold.len()as i32)as usize]},4=>ice[self.seed.nextInt(ice.len()as i32)as usize],_=>14}};o[(xx+yy*w)as usize]=v;}}o}
}
