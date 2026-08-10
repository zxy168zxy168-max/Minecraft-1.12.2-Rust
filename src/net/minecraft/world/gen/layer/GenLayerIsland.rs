use super::GenLayer::{GenLayer, GenLayerSeed};
#[derive(Debug)] pub struct GenLayerIsland { seed: GenLayerSeed }
impl GenLayerIsland { pub fn new(seed:i64)->Self{Self{seed:GenLayerSeed::new(seed)}} }
impl GenLayer for GenLayerIsland {
 fn initWorldGenSeed(&mut self,seed:i64){self.seed.initWorldGenSeed(seed)}
 fn getInts(&mut self,areaX:i32,areaY:i32,w:i32,h:i32)->Vec<i32>{let mut out=vec![0;(w*h) as usize];for y in 0..h{for x in 0..w{self.seed.initChunkSeed((areaX+x) as i64,(areaY+y) as i64);out[(x+y*w)as usize]=if self.seed.nextInt(10)==0{1}else{0};}}if areaX>-w&&areaX<=0&&areaY>-h&&areaY<=0{out[(-areaX + -areaY*w)as usize]=1;}out}
}
