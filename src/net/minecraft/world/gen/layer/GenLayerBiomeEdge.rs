use super::GenLayer::{biomesEqualOrMesaPlateau,GenLayer,GenLayerSeed,Layer};
use crate::net::minecraft::world::biome::Biome::{Biome,TempCategory};
#[derive(Debug)] pub struct GenLayerBiomeEdge{seed:GenLayerSeed,parent:Layer}
impl GenLayerBiomeEdge{pub fn new(seed:i64,parent:Layer)->Self{Self{seed:GenLayerSeed::new(seed),parent}}
 fn neighbors(p:&[i32],x:i32,y:i32,w:i32)->[i32;4]{let pw=w+2;[p[(x+1+y*pw)as usize],p[(x+2+(y+1)*pw)as usize],p[(x+(y+1)*pw)as usize],p[(x+1+(y+2)*pw)as usize]]}
 fn canNeighbors(a:i32,b:i32)->bool{if biomesEqualOrMesaPlateau(a,b){return true}let(Some(ba),Some(bb))=(Biome::getBiomeForId(a),Biome::getBiomeForId(b))else{return false};let(a,b)=(ba.getTempCategory(),bb.getTempCategory());a==b||a==TempCategory::Medium||b==TempCategory::Medium}
 fn replaceNecessary(p:&[i32],x:i32,y:i32,w:i32,c:i32,target:i32,repl:i32)->Option<i32>{if !biomesEqualOrMesaPlateau(c,target){return None}let ns=Self::neighbors(p,x,y,w);Some(if ns.into_iter().all(|v|Self::canNeighbors(v,target)){c}else{repl})}
 fn replaceEdge(p:&[i32],x:i32,y:i32,w:i32,c:i32,target:i32,repl:i32)->Option<i32>{if c!=target{return None}let ns=Self::neighbors(p,x,y,w);Some(if ns.into_iter().all(|v|biomesEqualOrMesaPlateau(v,target)){c}else{repl})}
}
impl GenLayer for GenLayerBiomeEdge{
 fn initWorldGenSeed(&mut self,s:i64){self.parent.lock().unwrap().initWorldGenSeed(s);self.seed.initWorldGenSeed(s)}
 fn getInts(&mut self,x:i32,z:i32,w:i32,h:i32)->Vec<i32>{let p=self.parent.lock().unwrap().getInts(x-1,z-1,w+2,h+2);let mut o=vec![0;(w*h)as usize];for yy in 0..h{for xx in 0..w{self.seed.initChunkSeed((xx+x)as i64,(yy+z)as i64);let c=p[(xx+1+(yy+1)*(w+2))as usize];let v=if let Some(v)=Self::replaceNecessary(&p,xx,yy,w,c,3,20){v}else if let Some(v)=Self::replaceEdge(&p,xx,yy,w,c,38,37){v}else if let Some(v)=Self::replaceEdge(&p,xx,yy,w,c,39,37){v}else if let Some(v)=Self::replaceEdge(&p,xx,yy,w,c,32,5){v}else if c==2{let ns=Self::neighbors(&p,xx,yy,w);if ns.into_iter().any(|v|v==12){34}else{c}}else if c==6{let ns=Self::neighbors(&p,xx,yy,w);if ns.into_iter().any(|v|matches!(v,2|30|12)){1}else if ns.into_iter().any(|v|v==21){23}else{c}}else{c};o[(xx+yy*w)as usize]=v;}}o}
}
