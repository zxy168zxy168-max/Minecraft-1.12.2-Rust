use rand::Rng;

use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::client::particle::Particle::Particle;

/// MCP 1.12.2 `ParticleBubble`.
#[derive(Debug, Clone)]
pub struct ParticleBubble {
    pub particle: Particle,
}

impl ParticleBubble {
    pub fn new(
        x: f64,
        y: f64,
        z: f64,
        xSpeed: f64,
        ySpeed: f64,
        zSpeed: f64,
        random: &mut impl Rng,
    ) -> Self {
        let mut particle = Particle::new(x, y, z, xSpeed, ySpeed, zSpeed, random);
        particle.setRBGColorF(1.0, 1.0, 1.0);
        particle.setParticleTextureIndex(32);
        particle.setSize(0.02, 0.02);
        particle.particleScale *= random.gen::<f32>() * 0.6 + 0.2;
        particle.motionX =
            xSpeed * 0.20000000298023224 + (random.gen::<f64>() * 2.0 - 1.0) * 0.019999999552965164;
        particle.motionY =
            ySpeed * 0.20000000298023224 + (random.gen::<f64>() * 2.0 - 1.0) * 0.019999999552965164;
        particle.motionZ =
            zSpeed * 0.20000000298023224 + (random.gen::<f64>() * 2.0 - 1.0) * 0.019999999552965164;
        particle.particleMaxAge = (8.0 / (random.gen::<f64>() * 0.8 + 0.2)) as i32;
        Self { particle }
    }

    pub fn onUpdate(&mut self, world: &WorldClient) {
        let p = &mut self.particle;
        p.prevPosX = p.posX;
        p.prevPosY = p.posY;
        p.prevPosZ = p.posZ;
        p.motionY += 0.002;
        p.moveEntity(world, p.motionX, p.motionY, p.motionZ);
        p.motionX *= 0.8500000238418579;
        p.motionY *= 0.8500000238418579;
        p.motionZ *= 0.8500000238418579;
        let blockId = world
            .getBlockState(crate::net::minecraft::util::math::BlockPos::BlockPos::new(
                p.posX.floor() as i32,
                p.posY.floor() as i32,
                p.posZ.floor() as i32,
            ))
            .getBlockId();
        if !matches!(blockId, 8 | 9) {
            p.isExpired = true;
        }
        let expired = p.particleMaxAge <= 0;
        p.particleMaxAge -= 1;
        if expired {
            p.isExpired = true;
        }
    }
}
