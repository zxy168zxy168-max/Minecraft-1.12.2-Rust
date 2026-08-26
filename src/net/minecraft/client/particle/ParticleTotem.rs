use rand::Rng;

use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::client::particle::Particle::Particle;

/// MCP 1.12.2 `ParticleTotem`, a `ParticleSimpleAnimated` specialization.
#[derive(Debug, Clone)]
pub struct ParticleTotem {
    pub particle: Particle,
}

impl ParticleTotem {
    pub fn new(
        x: f64,
        y: f64,
        z: f64,
        x_speed: f64,
        y_speed: f64,
        z_speed: f64,
        random: &mut impl Rng,
    ) -> Self {
        let mut particle = Particle::new_position(x, y, z, random);
        particle.motionX = x_speed;
        particle.motionY = y_speed;
        particle.motionZ = z_speed;
        particle.particleScale *= 0.75;
        particle.particleMaxAge = 60 + random.gen_range(0..12);

        if random.gen_range(0..4) == 0 {
            particle.setRBGColorF(
                0.6 + random.gen::<f32>() * 0.2,
                0.6 + random.gen::<f32>() * 0.3,
                random.gen::<f32>() * 0.2,
            );
        } else {
            particle.setRBGColorF(
                0.1 + random.gen::<f32>() * 0.2,
                0.4 + random.gen::<f32>() * 0.3,
                random.gen::<f32>() * 0.2,
            );
        }

        Self { particle }
    }

    pub fn onUpdate(&mut self, world: &WorldClient) {
        let particle = &mut self.particle;
        particle.prevPosX = particle.posX;
        particle.prevPosY = particle.posY;
        particle.prevPosZ = particle.posZ;

        if particle.particleAge >= particle.particleMaxAge {
            particle.isExpired = true;
        }
        particle.particleAge += 1;

        if particle.particleAge > particle.particleMaxAge / 2 {
            particle.setAlphaF(
                1.0 - (particle.particleAge as f32 - (particle.particleMaxAge / 2) as f32)
                    / particle.particleMaxAge.max(1) as f32,
            );
        }

        // ParticleSimpleAnimated(textureIdx=176, numFrames=8, yAccel=-0.05F).
        particle.setParticleTextureIndex(
            176 + (7 - particle.particleAge * 8 / particle.particleMaxAge.max(1)),
        );
        particle.motionY += -0.05;
        particle.moveEntity(world, particle.motionX, particle.motionY, particle.motionZ);
        particle.motionX *= 0.6;
        particle.motionY *= 0.6;
        particle.motionZ *= 0.6;
        if particle.isCollided {
            particle.motionX *= 0.699999988079071;
            particle.motionZ *= 0.699999988079071;
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use super::ParticleTotem;

    #[test]
    fn constructor_matches_vanilla_frame_and_lifetime_ranges() {
        let mut random = StdRng::seed_from_u64(31100);
        let particle = ParticleTotem::new(0.0, 0.0, 0.0, 1.0, 2.0, 3.0, &mut random);
        assert_eq!(particle.particle.motionX, 1.0);
        assert_eq!(particle.particle.motionY, 2.0);
        assert_eq!(particle.particle.motionZ, 3.0);
        assert!((60..=71).contains(&particle.particle.particleMaxAge));
    }
}
