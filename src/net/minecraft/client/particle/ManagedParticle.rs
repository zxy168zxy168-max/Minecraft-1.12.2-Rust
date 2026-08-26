use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::client::particle::Particle::Particle;
use crate::net::minecraft::client::particle::ParticleBubble::ParticleBubble;
use crate::net::minecraft::client::particle::ParticleDragonBreath::ParticleDragonBreath;
use crate::net::minecraft::client::particle::ParticleEndRod::ParticleEndRod;
use crate::net::minecraft::client::particle::ParticleRenderState::ParticleRenderState;
use crate::net::minecraft::client::particle::ParticleSmokeNormal::ParticleSmokeNormal;
use crate::net::minecraft::client::particle::ParticleSpell::ParticleSpell;
use crate::net::minecraft::client::particle::ParticleTotem::ParticleTotem;

#[derive(Debug, Clone)]
pub enum ManagedParticle {
    Bubble(ParticleBubble),
    Smoke(ParticleSmokeNormal),
    DragonBreath(ParticleDragonBreath),
    EndRod(ParticleEndRod),
    Spell(ParticleSpell),
    Totem(ParticleTotem),
}
impl ManagedParticle {
    fn particle(&self) -> &Particle {
        match self {
            Self::Bubble(v) => &v.particle,
            Self::Smoke(v) => &v.particle,
            Self::DragonBreath(v) => &v.particle,
            Self::EndRod(v) => &v.particle,
            Self::Spell(v) => &v.particle,
            Self::Totem(v) => &v.particle,
        }
    }
    pub fn onUpdate(&mut self, world: &WorldClient) {
        match self {
            Self::Bubble(v) => v.onUpdate(world),
            Self::Smoke(v) => v.onUpdate(world),
            Self::DragonBreath(v) => v.onUpdate(world),
            Self::EndRod(v) => v.onUpdate(world),
            Self::Spell(v) => v.onUpdate(world),
            Self::Totem(v) => v.onUpdate(world),
        }
    }
    pub fn isExpired(&self) -> bool {
        self.particle().isExpired
    }
    pub fn isTransparent(&self) -> bool {
        matches!(self, Self::EndRod(_) | Self::Spell(_) | Self::Totem(_))
    }
    pub fn renderState(&self, partialTicks: f32) -> ParticleRenderState {
        let p = self.particle();
        let scale = match self {
            Self::Smoke(v) => v.renderScale(partialTicks),
            Self::DragonBreath(v) => v.renderScale(partialTicks),
            _ => p.particleScale,
        };
        ParticleRenderState {
            prevPosition: [p.prevPosX, p.prevPosY, p.prevPosZ],
            position: [p.posX, p.posY, p.posZ],
            textureIndex: p.particleTextureIndexY * 16 + p.particleTextureIndexX,
            scale,
            particleAngle: p.particleAngle,
            prevParticleAngle: p.prevParticleAngle,
            color: [
                p.particleRed,
                p.particleGreen,
                p.particleBlue,
                p.particleAlpha,
            ],
            fullBright: matches!(self, Self::EndRod(_) | Self::Totem(_)),
            transparent: self.isTransparent(),
        }
    }
}
