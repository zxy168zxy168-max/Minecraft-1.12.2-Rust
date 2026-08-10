use super::GenLayer::{GenLayer,GenLayerSeed,Layer};
#[derive(Debug)] pub struct GenLayerRiverInit{seed:GenLayerSeed,parent:Layer}
impl GenLayerRiverInit{pub fn new(seed:i64,parent:Layer)->Self{Self{seed:GenLayerSeed::new(seed),parent}}}
impl GenLayer for GenLayerRiverInit{
 fn initWorldGenSeed(&mut self,s:i64){self.parent.lock().unwrap().initWorldGenSeed(s);self.seed.initWorldGenSeed(s)}
 fn getInts(&mut self,x:i32,z:i32,w:i32,h:i32)->Vec<i32>{let p=self.parent.lock().unwrap().getInts(x,z,w,h);let mut o=vec![0;(w*h)as usize];for yy in 0..h{for xx in 0..w{self.seed.initChunkSeed((xx+x)as i64,(yy+z)as i64);let c=p[(xx+yy*w)as usize];o[(xx+yy*w)as usize]=if c>0{self.seed.nextInt(299999)+2}else{0};}}o}
}
