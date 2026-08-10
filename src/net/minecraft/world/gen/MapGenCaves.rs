use crate::compat::Java::JavaRandom;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::math::MathHelper::{cos,floor_f64,sin,PI};
use crate::net::minecraft::world::biome::BiomeProviderKind::BiomeProviderKind;
use crate::net::minecraft::world::chunk::ChunkPrimer::ChunkPrimer;
use crate::net::minecraft::world::gen::MapGenBase::MapGenBase;

/// Exact MCP 1.12.2 `MapGenCaves` carving algorithm.
#[derive(Debug,Clone)]
pub struct MapGenCaves{base:MapGenBase}
impl Default for MapGenCaves{fn default()->Self{Self{base:MapGenBase::new()}}}
impl MapGenCaves{
    pub fn new()->Self{Self::default()}
    pub fn generate(&mut self,world_seed:i64,provider:&BiomeProviderKind,current_top:&[IBlockState;256],target_x:i32,target_z:i32,primer:&mut ChunkPrimer){
        let range=self.base.range;self.base.rand.set_seed(world_seed);let j=self.base.rand.next_i64();let k=self.base.rand.next_i64();
        for source_x in target_x-range..=target_x+range{for source_z in target_z-range..=target_z+range{
            let j1=(source_x as i64).wrapping_mul(j);let k1=(source_z as i64).wrapping_mul(k);self.base.rand.set_seed(j1^k1^world_seed);
            self.recursiveGenerate(provider,current_top,source_x,source_z,target_x,target_z,primer);
        }}
    }
    fn addRoom(&mut self,provider:&BiomeProviderKind,current_top:&[IBlockState;256],seed:i64,target_x:i32,target_z:i32,primer:&mut ChunkPrimer,x:f64,y:f64,z:f64){
        let size=1.0+self.base.rand.next_f32()*6.0;self.addTunnel(provider,current_top,seed,target_x,target_z,primer,x,y,z,size,0.0,0.0,-1,-1,0.5);
    }
    #[allow(clippy::too_many_arguments)]
    fn addTunnel(&mut self,provider:&BiomeProviderKind,current_top:&[IBlockState;256],seed:i64,target_x:i32,target_z:i32,primer:&mut ChunkPrimer,mut x:f64,mut y:f64,mut z:f64,size:f32,mut yaw:f32,mut pitch:f32,mut step:i32,mut max_steps:i32,vertical_scale:f64){
        let center_x=(target_x*16+8)as f64;let center_z=(target_z*16+8)as f64;let mut yaw_vel=0.0_f32;let mut pitch_vel=0.0_f32;let mut random=JavaRandom::new(seed);
        if max_steps<=0{let i=self.base.range*16-16;max_steps=i-random.next_i32_bound(i/4);}
        let mut room=false;if step==-1{step=max_steps/2;room=true;}
        let branch=random.next_i32_bound(max_steps/2)+max_steps/4;let gentle=random.next_i32_bound(6)==0;
        while step<max_steps{
            let radius=1.5+sin(step as f32*PI/max_steps as f32)as f64*size as f64;let vertical_radius=radius*vertical_scale;
            let cp=cos(pitch);let sp=sin(pitch);x+=(cos(yaw)*cp)as f64;y+=sp as f64;z+=(sin(yaw)*cp)as f64;
            pitch*=if gentle{0.92}else{0.7};pitch+=pitch_vel*0.1;yaw+=yaw_vel*0.1;pitch_vel*=0.9;yaw_vel*=0.75;
            pitch_vel+=(random.next_f32()-random.next_f32())*random.next_f32()*2.0;yaw_vel+=(random.next_f32()-random.next_f32())*random.next_f32()*4.0;
            if !room&&step==branch&&size>1.0&&max_steps>0{
                self.addTunnel(provider,current_top,random.next_i64(),target_x,target_z,primer,x,y,z,random.next_f32()*0.5+0.5,yaw-PI/2.0,pitch/3.0,step,max_steps,1.0);
                self.addTunnel(provider,current_top,random.next_i64(),target_x,target_z,primer,x,y,z,random.next_f32()*0.5+0.5,yaw+PI/2.0,pitch/3.0,step,max_steps,1.0);return;
            }
            if room||random.next_i32_bound(4)!=0{
                let dx=x-center_x;let dz=z-center_z;let remain=(max_steps-step)as f64;let max=(size+2.0+16.0)as f64;if dx*dx+dz*dz-remain*remain>max*max{return;}
                if x>=center_x-16.0-radius*2.0&&z>=center_z-16.0-radius*2.0&&x<=center_x+16.0+radius*2.0&&z<=center_z+16.0+radius*2.0{
                    let mut min_x=floor_f64(x-radius)-target_x*16-1;let mut max_x=floor_f64(x+radius)-target_x*16+1;let mut min_y=floor_f64(y-vertical_radius)-1;let mut max_y=floor_f64(y+vertical_radius)+1;let mut min_z=floor_f64(z-radius)-target_z*16-1;let mut max_z=floor_f64(z+radius)-target_z*16+1;
                    min_x=min_x.max(0);max_x=max_x.min(16);min_y=min_y.max(1);max_y=max_y.min(248);min_z=min_z.max(0);max_z=max_z.min(16);
                    let mut water=false;let mut scan_x=min_x;
                    while !water&&scan_x<max_x{let mut scan_z=min_z;while !water&&scan_z<max_z{let mut scan_y=max_y+1;while !water&&scan_y>=min_y-1{if (0..256).contains(&scan_y){let id=primer.getBlockState(scan_x as usize,scan_y as usize,scan_z as usize).getBlockId();if id==8||id==9{water=true;}if scan_y!=min_y-1&&scan_x!=min_x&&scan_x!=max_x-1&&scan_z!=min_z&&scan_z!=max_z-1{scan_y=min_y;}}scan_y-=1;}scan_z+=1;}scan_x+=1;}
                    if !water{
                        for carve_x in min_x..max_x{let ndx=((carve_x+target_x*16)as f64+0.5-x)/radius;for carve_z in min_z..max_z{let ndz=((carve_z+target_z*16)as f64+0.5-z)/radius;let mut surface=false;if ndx*ndx+ndz*ndz<1.0{for carve_y in ((min_y+1)..=max_y).rev(){let ndy=((carve_y-1)as f64+0.5-y)/vertical_radius;if ndy>-0.7&&ndx*ndx+ndy*ndy+ndz*ndz<1.0{let state=primer.getBlockState(carve_x as usize,carve_y as usize,carve_z as usize);let above=if carve_y<255{primer.getBlockState(carve_x as usize,(carve_y+1)as usize,carve_z as usize)}else{IBlockState::fromGlobalStateId(0)};if state.getBlockId()==2||state.getBlockId()==110{surface=true;}if Self::canReplaceBlock(state,above){if carve_y-1<10{primer.setBlockState(carve_x as usize,carve_y as usize,carve_z as usize,IBlockState::fromGlobalStateId(11<<4));}else{primer.setBlockState(carve_x as usize,carve_y as usize,carve_z as usize,IBlockState::fromGlobalStateId(0));if surface&&primer.getBlockState(carve_x as usize,(carve_y-1)as usize,carve_z as usize).getBlockId()==3{let biome=provider.getBiome(BlockPos::new(carve_x+target_x*16,0,carve_z+target_z*16));let top=current_top[biome.getId()as usize];primer.setBlockState(carve_x as usize,(carve_y-1)as usize,carve_z as usize,IBlockState::fromGlobalStateId(top.getBlockId()<<4));}}}}}}}}
                        if room{break;}
                    }
                }
            }
            step+=1;
        }
    }
    fn canReplaceBlock(state:IBlockState,above:IBlockState)->bool{
        match state.getBlockId(){1|2|3|24|78|110|159|172|179=>true,12|13=>!matches!(above.getBlockId(),8|9),_=>false}
    }
    fn recursiveGenerate(&mut self,provider:&BiomeProviderKind,current_top:&[IBlockState;256],source_x:i32,source_z:i32,target_x:i32,target_z:i32,primer:&mut ChunkPrimer){
        let a=self.base.rand.next_i32_bound(15)+1;let b=self.base.rand.next_i32_bound(a)+1;let mut count=self.base.rand.next_i32_bound(b);if self.base.rand.next_i32_bound(7)!=0{count=0;}
        for _ in 0..count{let x=(source_x*16+self.base.rand.next_i32_bound(16))as f64;let inner=self.base.rand.next_i32_bound(120)+8;let y=self.base.rand.next_i32_bound(inner)as f64;let z=(source_z*16+self.base.rand.next_i32_bound(16))as f64;let mut tunnels=1;if self.base.rand.next_i32_bound(4)==0{let seed=self.base.rand.next_i64();self.addRoom(provider,current_top,seed,target_x,target_z,primer,x,y,z);tunnels+=self.base.rand.next_i32_bound(4);}for _ in 0..tunnels{let yaw=self.base.rand.next_f32()*(PI*2.0);let pitch=(self.base.rand.next_f32()-0.5)*2.0/8.0;let mut size=self.base.rand.next_f32()*2.0+self.base.rand.next_f32();if self.base.rand.next_i32_bound(10)==0{size*=self.base.rand.next_f32()*self.base.rand.next_f32()*3.0+1.0;}let seed=self.base.rand.next_i64();self.addTunnel(provider,current_top,seed,target_x,target_z,primer,x,y,z,size,yaw,pitch,0,0,1.0);}}
    }
}
