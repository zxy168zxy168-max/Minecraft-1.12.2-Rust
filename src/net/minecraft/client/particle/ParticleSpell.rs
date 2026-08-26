use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::client::particle::Particle::Particle;
use crate::net::minecraft::util::EnumParticleTypes::EnumParticleTypes;
use rand::Rng;

/// MCP 1.12.2 `ParticleSpell` factory variants used by area-effect clouds.
#[derive(Debug, Clone)]
pub struct ParticleSpell {
    pub particle: Particle,
    baseSpellTextureIndex: i32,
}
impl ParticleSpell {
    pub fn new(
        kind: EnumParticleTypes,
        x: f64,
        y: f64,
        z: f64,
        xSpeed: f64,
        ySpeed: f64,
        zSpeed: f64,
        random: &mut impl Rng,
    ) -> Self {
        let mut p = Particle::new(
            x,
            y,
            z,
            0.5 - random.gen::<f64>(),
            ySpeed,
            0.5 - random.gen::<f64>(),
            random,
        );
        p.motionY *= 0.20000000298023224;
        if xSpeed == 0.0 && zSpeed == 0.0 {
            p.motionX *= 0.10000000149011612;
            p.motionZ *= 0.10000000149011612;
        }
        p.particleScale *= 0.75;
        p.particleMaxAge = (8.0 / (random.gen::<f64>() * 0.8 + 0.2)) as i32;
        let mut base = 128;
        match kind {
            EnumParticleTypes::SpellInstant => base = 144,
            EnumParticleTypes::SpellMob => {
                p.setRBGColorF(xSpeed as f32, ySpeed as f32, zSpeed as f32)
            }
            EnumParticleTypes::SpellMobAmbient => {
                p.setAlphaF(0.15);
                p.setRBGColorF(xSpeed as f32, ySpeed as f32, zSpeed as f32);
            }
            EnumParticleTypes::SpellWitch => {
                base = 144;
                let f = random.gen::<f32>() * 0.5 + 0.35;
                p.setRBGColorF(f, 0.0, f);
            }
            _ => {}
        }
        Self {
            particle: p,
            baseSpellTextureIndex: base,
        }
    }
    pub fn onUpdate(&mut self, world: &WorldClient) {
        let p = &mut self.particle;
        p.prevPosX = p.posX;
        p.prevPosY = p.posY;
        p.prevPosZ = p.posZ;
        if p.particleAge >= p.particleMaxAge {
            p.isExpired = true;
        }
        p.particleAge += 1;
        p.setParticleTextureIndex(
            self.baseSpellTextureIndex + (7 - p.particleAge * 8 / p.particleMaxAge.max(1)),
        );
        p.motionY += 0.004;
        p.moveEntity(world, p.motionX, p.motionY, p.motionZ);
        if p.posY == p.prevPosY {
            p.motionX *= 1.1;
            p.motionZ *= 1.1;
        }
        p.motionX *= 0.9599999785423279;
        p.motionY *= 0.9599999785423279;
        p.motionZ *= 0.9599999785423279;
        if p.isCollided {
            p.motionX *= 0.699999988079071;
            p.motionZ *= 0.699999988079071;
        }
    }
}
