use super::GenLayer::{GenLayer,GenLayerSeed,Layer};
#[derive(Debug)] pub struct GenLayerAddIsland{seed:GenLayerSeed,parent:Layer}
impl GenLayerAddIsland{pub fn new(seed:i64,parent:Layer)->Self{Self{seed:GenLayerSeed::new(seed),parent}}}
impl GenLayer for GenLayerAddIsland{
 fn initWorldGenSeed(&mut self,s:i64){self.parent.lock().unwrap().initWorldGenSeed(s);self.seed.initWorldGenSeed(s)}
 fn getInts(&mut self,x:i32,z:i32,w:i32,h:i32)->Vec<i32>{let px=x-1;let pz=z-1;let pw=w+2;let ph=h+2;let p=self.parent.lock().unwrap().getInts(px,pz,pw,ph);let mut o=vec![0;(w*h)as usize];for yy in 0..h{for xx in 0..w{let nw=p[(xx+(yy)*pw)as usize];let ne=p[(xx+2+yy*pw)as usize];let sw=p[(xx+(yy+2)*pw)as usize];let se=p[(xx+2+(yy+2)*pw)as usize];let c=p[(xx+1+(yy+1)*pw)as usize];self.seed.initChunkSeed((xx+x)as i64,(yy+z)as i64);let v=if c!=0 || (nw==0&&ne==0&&sw==0&&se==0){if c>0&&(nw==0||ne==0||sw==0||se==0){if self.seed.nextInt(5)==0{if c==4{4}else{0}}else{c}}else{c}}else{let mut count=1;let mut pick=1;for n in [nw,ne,sw,se]{if n!=0{if self.seed.nextInt(count)==0{pick=n;}count+=1;}}if self.seed.nextInt(3)==0{pick}else if pick==4{4}else{0}};o[(xx+yy*w)as usize]=v;}}o}
}
