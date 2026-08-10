use super::GenLayer::{isBiomeOceanic,GenLayer,GenLayerSeed,Layer};
use crate::net::minecraft::world::biome::Biome::Biome;
#[derive(Debug)] pub struct GenLayerShore{seed:GenLayerSeed,parent:Layer}
impl GenLayerShore{
 pub fn new(seed:i64,parent:Layer)->Self{Self{seed:GenLayerSeed::new(seed),parent}}
 fn neighbors(p:&[i32],x:i32,y:i32,w:i32)->[i32;4]{let pw=w+2;[p[(x+1+y*pw)as usize],p[(x+2+(y+1)*pw)as usize],p[(x+(y+1)*pw)as usize],p[(x+1+(y+2)*pw)as usize]]}
 fn replaceIfNeighborOcean(p:&[i32],x:i32,y:i32,w:i32,c:i32,repl:i32)->i32{if isBiomeOceanic(c){return c}let ns=Self::neighbors(p,x,y,w);if ns.into_iter().all(|v|!isBiomeOceanic(v)){c}else{repl}}
 fn isJungleCompatible(id:i32)->bool{Biome::getBiomeForId(id).is_some_and(|b|b.isJungleClass())||matches!(id,23|21|22|4|5)||isBiomeOceanic(id)}
 fn isMesa(id:i32)->bool{Biome::getBiomeForId(id).is_some_and(|b|b.isMesa())}
}
impl GenLayer for GenLayerShore{
 fn initWorldGenSeed(&mut self,s:i64){self.parent.lock().unwrap().initWorldGenSeed(s);self.seed.initWorldGenSeed(s)}
 fn getInts(&mut self,x:i32,z:i32,w:i32,h:i32)->Vec<i32>{let p=self.parent.lock().unwrap().getInts(x-1,z-1,w+2,h+2);let mut o=vec![0;(w*h)as usize];for yy in 0..h{for xx in 0..w{self.seed.initChunkSeed((xx+x)as i64,(yy+z)as i64);let c=p[(xx+1+(yy+1)*(w+2))as usize];let biome=Biome::getBiomeForId(c);let ns=Self::neighbors(&p,xx,yy,w);let v=if c==14{if ns.into_iter().all(|v|v!=0){c}else{15}}
 else if biome.is_some_and(|b|b.isJungleClass()){if ns.into_iter().all(Self::isJungleCompatible){if ns.into_iter().all(|v|!isBiomeOceanic(v)){c}else{16}}else{23}}
 else if matches!(c,3|34|20){Self::replaceIfNeighborOcean(&p,xx,yy,w,c,25)}
 else if biome.is_some_and(|b|b.isSnowyBiome()){Self::replaceIfNeighborOcean(&p,xx,yy,w,c,26)}
 else if !matches!(c,37|38){if !matches!(c,0|24|7|6){if ns.into_iter().all(|v|!isBiomeOceanic(v)){c}else{16}}else{c}}
 else {if ns.into_iter().all(|v|!isBiomeOceanic(v)){if ns.into_iter().all(Self::isMesa){c}else{2}}else{c}};o[(xx+yy*w)as usize]=v;}}o}
}
