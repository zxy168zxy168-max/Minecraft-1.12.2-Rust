use super::GenLayer::{GenLayer,GenLayerSeed,Layer};
#[derive(Debug)] pub struct GenLayerRiverMix{seed:GenLayerSeed,biome:Layer,river:Layer}
impl GenLayerRiverMix{pub fn new(seed:i64,biome:Layer,river:Layer)->Self{Self{seed:GenLayerSeed::new(seed),biome,river}}}
impl GenLayer for GenLayerRiverMix{fn initWorldGenSeed(&mut self,s:i64){self.biome.lock().unwrap().initWorldGenSeed(s);self.river.lock().unwrap().initWorldGenSeed(s);self.seed.initWorldGenSeed(s)}fn getInts(&mut self,x:i32,z:i32,w:i32,h:i32)->Vec<i32>{let b=self.biome.lock().unwrap().getInts(x,z,w,h);let r=self.river.lock().unwrap().getInts(x,z,w,h);let mut o=vec![0;(w*h)as usize];for i in 0..(w*h)as usize{o[i]=if !matches!(b[i],0|24)&&r[i]==7{if b[i]==12{11}else if matches!(b[i],14|15){15}else{r[i]&255}}else{b[i]};}o}}
