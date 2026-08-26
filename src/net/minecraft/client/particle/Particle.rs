use rand::Rng;

use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;

/// Minimal exact base needed by MCP 1.12.2 `ParticleDigging`.
#[derive(Debug, Clone)]
pub struct Particle {
    pub prevPosX: f64,
    pub prevPosY: f64,
    pub prevPosZ: f64,
    pub posX: f64,
    pub posY: f64,
    pub posZ: f64,
    pub motionX: f64,
    pub motionY: f64,
    pub motionZ: f64,
    pub boundingBox: AxisAlignedBB,
    pub isCollided: bool,
    pub canCollide: bool,
    pub isExpired: bool,
    pub width: f32,
    pub height: f32,
    pub particleTextureJitterX: f32,
    pub particleTextureJitterY: f32,
    pub particleTextureIndexX: i32,
    pub particleTextureIndexY: i32,
    pub particleAngle: f32,
    pub prevParticleAngle: f32,
    pub particleAge: i32,
    pub particleMaxAge: i32,
    pub particleScale: f32,
    pub particleGravity: f32,
    pub particleRed: f32,
    pub particleGreen: f32,
    pub particleBlue: f32,
    pub particleAlpha: f32,
}

impl Particle {
    pub fn new_position(x: f64, y: f64, z: f64, random: &mut impl Rng) -> Self {
        let mut particle = Self {
            prevPosX: x,
            prevPosY: y,
            prevPosZ: z,
            posX: x,
            posY: y,
            posZ: z,
            motionX: 0.0,
            motionY: 0.0,
            motionZ: 0.0,
            boundingBox: AxisAlignedBB::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            isCollided: false,
            canCollide: true,
            isExpired: false,
            width: 0.6,
            height: 1.8,
            particleTextureJitterX: random.gen::<f32>() * 3.0,
            particleTextureJitterY: random.gen::<f32>() * 3.0,
            particleTextureIndexX: 0,
            particleTextureIndexY: 0,
            particleAngle: 0.0,
            prevParticleAngle: 0.0,
            particleAge: 0,
            particleMaxAge: (4.0 / (random.gen::<f32>() * 0.9 + 0.1)) as i32,
            particleScale: (random.gen::<f32>() * 0.5 + 0.5) * 2.0,
            particleGravity: 0.0,
            particleRed: 1.0,
            particleGreen: 1.0,
            particleBlue: 1.0,
            particleAlpha: 1.0,
        };
        particle.setSize(0.2, 0.2);
        particle.setPosition(x, y, z);
        particle
    }

    pub fn new(
        x: f64,
        y: f64,
        z: f64,
        xSpeed: f64,
        ySpeed: f64,
        zSpeed: f64,
        random: &mut impl Rng,
    ) -> Self {
        let mut particle = Self::new_position(x, y, z, random);
        particle.motionX = xSpeed + (random.gen::<f64>() * 2.0 - 1.0) * 0.4000000059604645;
        particle.motionY = ySpeed + (random.gen::<f64>() * 2.0 - 1.0) * 0.4000000059604645;
        particle.motionZ = zSpeed + (random.gen::<f64>() * 2.0 - 1.0) * 0.4000000059604645;
        let speed = (particle.motionX * particle.motionX
            + particle.motionY * particle.motionY
            + particle.motionZ * particle.motionZ)
            .sqrt();
        let magnitude = (random.gen::<f64>() + random.gen::<f64>() + 1.0) * 0.15;
        particle.motionX = particle.motionX / speed * magnitude * 0.4000000059604645;
        particle.motionY =
            particle.motionY / speed * magnitude * 0.4000000059604645 + 0.10000000149011612;
        particle.motionZ = particle.motionZ / speed * magnitude * 0.4000000059604645;
        particle
    }

    pub fn setParticleTextureIndex(&mut self, particleTextureIndex: i32) {
        self.particleTextureIndexX = particleTextureIndex % 16;
        self.particleTextureIndexY = particleTextureIndex / 16;
    }

    pub fn setRBGColorF(&mut self, red: f32, green: f32, blue: f32) {
        self.particleRed = red;
        self.particleGreen = green;
        self.particleBlue = blue;
    }

    pub fn setAlphaF(&mut self, alpha: f32) {
        self.particleAlpha = alpha;
    }

    pub fn multiplyVelocity(&mut self, multiplier: f32) {
        let multiplier = multiplier as f64;
        self.motionX *= multiplier;
        self.motionY = (self.motionY - 0.10000000149011612) * multiplier + 0.10000000149011612;
        self.motionZ *= multiplier;
    }

    pub fn multipleParticleScaleBy(&mut self, scale: f32) {
        self.setSize(0.2 * scale, 0.2 * scale);
        self.particleScale *= scale;
    }

    pub fn setSize(&mut self, width: f32, height: f32) {
        if width == self.width && height == self.height {
            return;
        }
        let previousWidth = self.width;
        self.width = width;
        self.height = height;
        if width < previousWidth {
            // Entity#setSize keeps a shrinking entity centred on posX/posZ.
            let half = width as f64 / 2.0;
            self.boundingBox = AxisAlignedBB::new(
                self.posX - half,
                self.posY,
                self.posZ - half,
                self.posX + half,
                self.posY + height as f64,
                self.posZ + half,
            );
        } else {
            let bounds = self.boundingBox;
            self.boundingBox = AxisAlignedBB::new(
                bounds.min_x,
                bounds.min_y,
                bounds.min_z,
                bounds.min_x + width as f64,
                bounds.min_y + height as f64,
                bounds.min_z + width as f64,
            );
        }
    }

    pub fn setPosition(&mut self, x: f64, y: f64, z: f64) {
        self.posX = x;
        self.posY = y;
        self.posZ = z;
        let half = self.width as f64 / 2.0;
        self.boundingBox = AxisAlignedBB::new(
            x - half,
            y,
            z - half,
            x + half,
            y + self.height as f64,
            z + half,
        );
    }

    pub fn onUpdate(&mut self, world: &WorldClient) {
        self.prevPosX = self.posX;
        self.prevPosY = self.posY;
        self.prevPosZ = self.posZ;
        if self.particleAge >= self.particleMaxAge {
            self.isExpired = true;
        }
        self.particleAge += 1;
        self.motionY -= 0.04 * self.particleGravity as f64;
        self.moveEntity(world, self.motionX, self.motionY, self.motionZ);
        self.motionX *= 0.9800000190734863;
        self.motionY *= 0.9800000190734863;
        self.motionZ *= 0.9800000190734863;
        if self.isCollided {
            self.motionX *= 0.699999988079071;
            self.motionZ *= 0.699999988079071;
        }
    }

    pub(crate) fn moveEntity(&mut self, world: &WorldClient, mut x: f64, mut y: f64, mut z: f64) {
        let originalX = x;
        let originalY = y;
        let originalZ = z;
        if self.canCollide {
            let collisions = world.getCollisionBoxes(self.boundingBox.add_coord(x, y, z));
            for collision in &collisions {
                y = collision.calculate_y_offset(self.boundingBox, y);
            }
            self.boundingBox = self.boundingBox.offset(0.0, y, 0.0);
            for collision in &collisions {
                x = collision.calculate_x_offset(self.boundingBox, x);
            }
            self.boundingBox = self.boundingBox.offset(x, 0.0, 0.0);
            for collision in &collisions {
                z = collision.calculate_z_offset(self.boundingBox, z);
            }
            self.boundingBox = self.boundingBox.offset(0.0, 0.0, z);
        } else {
            self.boundingBox = self.boundingBox.offset(x, y, z);
        }
        self.resetPositionToBB();
        // Particle#moveEntity uses this flag as its downward-ground contact,
        // not Entity's aggregate horizontal/vertical collision state.
        self.isCollided = originalY != y && originalY < 0.0;
        if originalX != x {
            self.motionX = 0.0;
        }
        if originalZ != z {
            self.motionZ = 0.0;
        }
    }

    fn resetPositionToBB(&mut self) {
        self.posX = (self.boundingBox.min_x + self.boundingBox.max_x) * 0.5;
        self.posY = self.boundingBox.min_y;
        self.posZ = (self.boundingBox.min_z + self.boundingBox.max_z) * 0.5;
    }
}
