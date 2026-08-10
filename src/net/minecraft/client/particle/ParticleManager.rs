use std::time::{SystemTime, UNIX_EPOCH};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::compat::Java::JavaRandom;

use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockDoor;
use crate::net::minecraft::block::BlockFenceGate;
use crate::net::minecraft::block::BlockRedstoneWire;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::client::particle::ManagedParticle::ManagedParticle;
use crate::net::minecraft::client::particle::ParticleBubble::ParticleBubble;
use crate::net::minecraft::client::particle::ParticleDigging::{ParticleActualModel, ParticleDigging, ParticleDiggingRenderState};
use crate::net::minecraft::client::particle::ParticleDragonBreath::ParticleDragonBreath;
use crate::net::minecraft::client::particle::ParticleEndRod::ParticleEndRod;
use crate::net::minecraft::client::particle::ParticleRenderState::ParticleRenderState;
use crate::net::minecraft::client::particle::ParticleSmokeNormal::ParticleSmokeNormal;
use crate::net::minecraft::client::particle::ParticleSpawnRequest::ParticleSpawnRequest;
use crate::net::minecraft::client::particle::ParticleSpell::ParticleSpell;
use crate::net::minecraft::client::particle::ParticleTotem::ParticleTotem;
use crate::net::minecraft::client::renderer::color::BlockColors::BlockColors;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::EnumParticleTypes::EnumParticleTypes;
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// MCP 1.12.2 `ParticleManager` queues. Terrain particles retain their
/// TextureMap layer; source-backed misc particles use `particles.png` layer 0.
pub struct ParticleManager {
    blockParticles: Vec<ParticleDigging>,
    miscParticles: Vec<ManagedParticle>,
    random: StdRng,
    visibilityRandom: JavaRandom,
    blockColors: BlockColors,
}

impl ParticleManager {
    pub fn new(blockColors: BlockColors) -> Self {
        Self {
            blockParticles: Vec::new(),
            miscParticles: Vec::new(),
            random: StdRng::from_entropy(),
            visibilityRandom: JavaRandom::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as i64,
            ),
            blockColors,
        }
    }

    pub fn updateEffects(&mut self, world: &WorldClient) {
        for particle in &mut self.blockParticles { particle.onUpdate(world); }
        self.blockParticles.retain(|particle| !particle.isExpired());
        for particle in &mut self.miscParticles { particle.onUpdate(world); }
        self.miscParticles.retain(|particle| !particle.isExpired());
    }

    pub fn clearEffects(&mut self) { self.blockParticles.clear(); self.miscParticles.clear(); }

    fn addBlockEffect(&mut self, particle: ParticleDigging) {
        if self.blockParticles.len() >= 16_384 { self.blockParticles.remove(0); }
        self.blockParticles.push(particle);
    }

    fn addMiscEffect(&mut self, particle: ManagedParticle) {
        let transparent = particle.isTransparent();
        if self.miscParticles.iter().filter(|entry| entry.isTransparent() == transparent).count() >= 16_384 {
            if let Some(index) = self.miscParticles.iter().position(|entry| entry.isTransparent() == transparent) {
                self.miscParticles.remove(index);
            }
        }
        self.miscParticles.push(particle);
    }

    pub fn spawnEffectParticle(&mut self, request: ParticleSpawnRequest) {
        let [x,y,z]=request.position; let [xs,ys,zs]=request.speed;
        let particle=match request.particleType {
            EnumParticleTypes::WaterBubble => Some(ManagedParticle::Bubble(ParticleBubble::new(x,y,z,xs,ys,zs,&mut self.random))),
            EnumParticleTypes::SmokeNormal => Some(ManagedParticle::Smoke(ParticleSmokeNormal::new(x,y,z,xs,ys,zs,1.0,&mut self.random))),
            // MCP ParticleSmokeLarge.Factory -> ParticleSmokeNormal(..., 2.5F).
            EnumParticleTypes::SmokeLarge => Some(ManagedParticle::Smoke(ParticleSmokeNormal::new(x,y,z,xs,ys,zs,2.5,&mut self.random))),
            EnumParticleTypes::DragonBreath => Some(ManagedParticle::DragonBreath(ParticleDragonBreath::new(x,y,z,xs,ys,zs,&mut self.random))),
            EnumParticleTypes::EndRod => Some(ManagedParticle::EndRod(ParticleEndRod::new(x,y,z,xs,ys,zs,&mut self.random))),
            EnumParticleTypes::Spell | EnumParticleTypes::SpellInstant | EnumParticleTypes::SpellMob
            | EnumParticleTypes::SpellMobAmbient | EnumParticleTypes::SpellWitch => Some(ManagedParticle::Spell(ParticleSpell::new(request.particleType,x,y,z,xs,ys,zs,&mut self.random))),
            EnumParticleTypes::Totem => Some(ManagedParticle::Totem(ParticleTotem::new(x,y,z,xs,ys,zs,&mut self.random))),
            _ => None,
        };
        if let Some(particle)=particle { self.addMiscEffect(particle); }
    }

    pub fn spawnEffects(
        &mut self,
        requests: impl IntoIterator<Item=ParticleSpawnRequest>,
        viewPosition: [f64; 3],
        particleSetting: i32,
    ) {
        for request in requests {
            let mut effectiveSetting = particleSetting;
            if request.alwaysRender && effectiveSetting == 2 && self.visibilityRandom.next_i32_bound(10) == 0 {
                effectiveSetting = 1;
            }
            if effectiveSetting == 1 && self.visibilityRandom.next_i32_bound(3) == 0 {
                effectiveSetting = 2;
            }
            if !request.ignoreRange {
                let dx = viewPosition[0] - request.position[0];
                let dy = viewPosition[1] - request.position[1];
                let dz = viewPosition[2] - request.position[2];
                let maximumDistanceSquared = if request.particleType == EnumParticleTypes::Crit {
                    38_416.0
                } else {
                    1_024.0
                };
                if dx * dx + dy * dy + dz * dz > maximumDistanceSquared || effectiveSetting > 1 {
                    continue;
                }
            }
            self.spawnEffectParticle(request);
        }
    }

    pub fn addBlockDestroyEffects(&mut self, world: &WorldClient, pos: BlockPos, state: IBlockState) {
        if state.isAir() { return; }
        // MCP `ParticleManager#addBlockDestroyEffects` evaluates
        // `state.getActualState(world, pos)` before the block is removed.
        // IBlockState's compact Rust representation stores legacy metadata,
        // so retain the extended model key separately with each particle.
        let actualModel = if BlockDoor::isBlockDoor(state) {
            Some(ParticleActualModel::Door {
                blockId: state.getBlockId(),
                key: BlockDoor::modelKey(state, world, pos),
            })
        } else if BlockFenceGate::isBlockFenceGate(state) {
            Some(ParticleActualModel::FenceGate {
                blockId: state.getBlockId(),
                key: BlockFenceGate::modelKey(state, world, pos),
            })
        } else if state.getBlockId() == BlockRedstoneWire::BLOCK_ID {
            Some(ParticleActualModel::RedstoneWire {
                key: BlockRedstoneWire::modelKey(world, pos),
            })
        } else if state.getBlockId() == 175 {
            let upper = state.getMetadata() & 8 != 0;
            let lowerState = if upper { world.getBlockState(pos.down(1)) } else { state };
            Some(ParticleActualModel::DoublePlant {
                variant: if lowerState.getBlockId() == 175 {
                    (lowerState.getMetadata() & 7).clamp(0, 5) as u8
                } else {
                    0
                },
                upper,
            })
        } else {
            None
        };
        for i in 0..4 { for j in 0..4 { for k in 0..4 {
            let d0=(i as f64+0.5)/4.0; let d1=(j as f64+0.5)/4.0; let d2=(k as f64+0.5)/4.0;
            let particle=ParticleDigging::new(pos.x as f64+d0,pos.y as f64+d1,pos.z as f64+d2,d0-0.5,d1-0.5,d2-0.5,state,&mut self.random)
                .withActualModel(actualModel)
                .setBlockPos(pos,&self.blockColors,world);
            self.addBlockEffect(particle);
        }}}
    }

    pub fn addBlockHitEffects(&mut self, world:&WorldClient,pos:BlockPos,side:EnumFacing) {
        let state=world.getBlockState(pos);
        if matches!(state.getBlockId(),0|36|63|68|119|144|166|176|177|209|217){return;}
        let Some(bounds)=world.getSelectedBoundingBox(pos) else{return;};
        let inset=0.10000000149011612_f64;
        let random_axis=|random:&mut StdRng,min:f64,max:f64|{let span=max-min-inset*2.0;min+inset+random.gen::<f64>()*span};
        let mut x=random_axis(&mut self.random,bounds.min_x,bounds.max_x);
        let mut y=random_axis(&mut self.random,bounds.min_y,bounds.max_y);
        let mut z=random_axis(&mut self.random,bounds.min_z,bounds.max_z);
        match side{EnumFacing::Down=>y=bounds.min_y-inset,EnumFacing::Up=>y=bounds.max_y+inset,EnumFacing::North=>z=bounds.min_z-inset,EnumFacing::South=>z=bounds.max_z+inset,EnumFacing::West=>x=bounds.min_x-inset,EnumFacing::East=>x=bounds.max_x+inset}
        let particle=ParticleDigging::new(x,y,z,0.0,0.0,0.0,state,&mut self.random).setBlockPos(pos,&self.blockColors,world).multiplyVelocity(0.2).multipleParticleScaleBy(0.6);
        self.addBlockEffect(particle);
    }

    pub fn renderStates(&self)->Vec<ParticleDiggingRenderState>{self.blockParticles.iter().map(ParticleDigging::renderState).collect()}
    pub fn miscRenderStates(&self,partialTicks:f32)->Vec<ParticleRenderState>{self.miscParticles.iter().map(|particle|particle.renderState(partialTicks)).collect()}
    pub fn getStatistics(&self)->String{(self.blockParticles.len()+self.miscParticles.len()).to_string()}
}
