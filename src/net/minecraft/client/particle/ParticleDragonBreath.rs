use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::client::particle::Particle::Particle;
use rand::Rng;

/// MCP 1.12.2 `ParticleDragonBreath`.
#[derive(Debug, Clone)]
pub struct ParticleDragonBreath {
    pub particle: Particle,
    originalScale: f32,
    hasHitGround: bool,
}
impl ParticleDragonBreath {
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
        particle.motionX = xSpeed;
        particle.motionY = ySpeed;
        particle.motionZ = zSpeed;
        particle.particleRed = 0.7176471 + random.gen::<f32>() * (0.8745098 - 0.7176471);
        particle.particleGreen = 0.0;
        particle.particleBlue = 0.8235294 + random.gen::<f32>() * (0.9764706 - 0.8235294);
        particle.particleScale *= 0.75;
        let originalScale = particle.particleScale;
        particle.particleMaxAge = (20.0 / (random.gen::<f64>() * 0.8 + 0.2)) as i32;
        Self {
            particle,
            originalScale,
            hasHitGround: false,
        }
    }
    pub fn renderScale(&self, partialTicks: f32) -> f32 {
        self.originalScale
            * (((self.particle.particleAge as f32 + partialTicks)
                / self.particle.particleMaxAge.max(1) as f32)
                * 32.0)
                .clamp(0.0, 1.0)
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
        if !p.isExpired {
            p.setParticleTextureIndex(3 * p.particleAge / p.particleMaxAge.max(1) + 5);
            if p.isCollided {
                p.motionY = 0.0;
                self.hasHitGround = true;
            }
            if self.hasHitGround {
                p.motionY += 0.002;
            }
            p.moveEntity(world, p.motionX, p.motionY, p.motionZ);
            if p.posY == p.prevPosY {
                p.motionX *= 1.1;
                p.motionZ *= 1.1;
            }
            p.motionX *= 0.9599999785423279;
            p.motionZ *= 0.9599999785423279;
            if self.hasHitGround {
                p.motionY *= 0.9599999785423279;
            }
        }
    }
}
