use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::client::particle::Particle::Particle;
use rand::Rng;

/// MCP 1.12.2 `ParticleSmokeNormal`.
#[derive(Debug, Clone)]
pub struct ParticleSmokeNormal {
    pub particle: Particle,
    pub smokeParticleScale: f32,
}
impl ParticleSmokeNormal {
    pub fn new(
        x: f64,
        y: f64,
        z: f64,
        xSpeed: f64,
        ySpeed: f64,
        zSpeed: f64,
        scale: f32,
        random: &mut impl Rng,
    ) -> Self {
        let mut particle = Particle::new(x, y, z, 0.0, 0.0, 0.0, random);
        particle.motionX = particle.motionX * 0.10000000149011612 + xSpeed;
        particle.motionY = particle.motionY * 0.10000000149011612 + ySpeed;
        particle.motionZ = particle.motionZ * 0.10000000149011612 + zSpeed;
        let shade = random.gen::<f32>() * 0.30000001192092896;
        particle.setRBGColorF(shade, shade, shade);
        particle.particleScale *= 0.75 * scale;
        let smokeParticleScale = particle.particleScale;
        particle.particleMaxAge = (8.0 / (random.gen::<f64>() * 0.8 + 0.2)) as i32;
        particle.particleMaxAge = (particle.particleMaxAge as f32 * scale) as i32;
        Self {
            particle,
            smokeParticleScale,
        }
    }
    pub fn renderScale(&self, partialTicks: f32) -> f32 {
        let f = (((self.particle.particleAge as f32 + partialTicks)
            / self.particle.particleMaxAge.max(1) as f32)
            * 32.0)
            .clamp(0.0, 1.0);
        self.smokeParticleScale * f
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
        p.setParticleTextureIndex(7 - p.particleAge * 8 / p.particleMaxAge.max(1));
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
