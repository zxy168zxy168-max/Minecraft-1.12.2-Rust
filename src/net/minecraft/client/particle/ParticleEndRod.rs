use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::client::particle::Particle::Particle;
use rand::Rng;

/// MCP 1.12.2 `ParticleEndRod` / `ParticleSimpleAnimated` specialization.
#[derive(Debug, Clone)]
pub struct ParticleEndRod {
    pub particle: Particle,
    fadeTarget: [f32; 3],
}
impl ParticleEndRod {
    pub fn new(
        x: f64,
        y: f64,
        z: f64,
        xSpeed: f64,
        ySpeed: f64,
        zSpeed: f64,
        random: &mut impl Rng,
    ) -> Self {
        let mut p = Particle::new_position(x, y, z, random);
        p.motionX = xSpeed;
        p.motionY = ySpeed;
        p.motionZ = zSpeed;
        p.particleScale *= 0.75;
        p.particleMaxAge = 60 + random.gen_range(0..12);
        Self {
            particle: p,
            fadeTarget: [242.0 / 255.0, 222.0 / 255.0, 201.0 / 255.0],
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
        if p.particleAge > p.particleMaxAge / 2 {
            p.setAlphaF(
                1.0 - (p.particleAge as f32 - (p.particleMaxAge / 2) as f32)
                    / p.particleMaxAge.max(1) as f32,
            );
            p.particleRed += (self.fadeTarget[0] - p.particleRed) * 0.2;
            p.particleGreen += (self.fadeTarget[1] - p.particleGreen) * 0.2;
            p.particleBlue += (self.fadeTarget[2] - p.particleBlue) * 0.2;
        }
        p.setParticleTextureIndex(176 + (8 - 1 - p.particleAge * 8 / p.particleMaxAge.max(1)));
        p.motionY += -5.0e-4;
        // ParticleEndRod overrides moveEntity and deliberately ignores collisions.
        p.canCollide = false;
        p.moveEntity(world, p.motionX, p.motionY, p.motionZ);
        p.motionX *= 0.91;
        p.motionY *= 0.91;
        p.motionZ *= 0.91;
    }
}
