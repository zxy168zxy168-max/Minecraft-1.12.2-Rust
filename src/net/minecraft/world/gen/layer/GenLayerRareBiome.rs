use super::GenLayer::{GenLayer,GenLayerSeed,Layer};
#[derive(Debug)] pub struct GenLayerRareBiome{seed:GenLayerSeed,parent:Layer}
impl GenLayerRareBiome{pub fn new(seed:i64,parent:Layer)->Self{Self{seed:GenLayerSeed::new(seed),parent}}}
impl GenLayer for GenLayerRareBiome{fn initWorldGenSeed(&mut self,s:i64){self.parent.lock().unwrap().initWorldGenSeed(s);self.seed.initWorldGenSeed(s)}fn getInts(&mut self,x:i32,z:i32,w:i32,h:i32)->Vec<i32>{let pw=w+2;let p=self.parent.lock().unwrap().getInts(x-1,z-1,pw,h+2);let mut o=vec![0;(w*h)as usize];for yy in 0..h{for xx in 0..w{self.seed.initChunkSeed((xx+x)as i64,(yy+z)as i64);let c=p[(xx+1+(yy+1)*pw)as usize];o[(xx+yy*w)as usize]=if self.seed.nextInt(57)==0&&c==1{129}else{c};}}o}}
