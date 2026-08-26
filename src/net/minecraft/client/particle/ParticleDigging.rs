use rand::Rng;

use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::client::particle::Particle::Particle;
use crate::net::minecraft::client::renderer::color::BlockColors::BlockColors;
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// Compact renderer-side equivalent of the extended properties produced by
/// `IBlockState#getActualState`. Vanilla resolves these properties when a
/// destroy effect is spawned, before the broken block is removed from World.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleActualModel {
    Door { blockId: i32, key: u8 },
    FenceGate { blockId: i32, key: u8 },
    RedstoneWire { key: u8 },
    DoublePlant { variant: u8, upper: bool },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleDiggingRenderState {
    pub sourceState: IBlockState,
    pub sourcePos: BlockPos,
    pub actualModel: Option<ParticleActualModel>,
    pub prevPosition: [f64; 3],
    pub position: [f64; 3],
    pub textureJitter: [f32; 2],
    pub scale: f32,
    pub color: [f32; 4],
}

/// MCP 1.12.2 `ParticleDigging`.
#[derive(Debug, Clone)]
pub struct ParticleDigging {
    pub particle: Particle,
    sourceState: IBlockState,
    sourcePos: BlockPos,
    actualModel: Option<ParticleActualModel>,
}

impl ParticleDigging {
    pub fn new(
        x: f64,
        y: f64,
        z: f64,
        xSpeed: f64,
        ySpeed: f64,
        zSpeed: f64,
        state: IBlockState,
        random: &mut impl Rng,
    ) -> Self {
        let mut particle = Particle::new(x, y, z, xSpeed, ySpeed, zSpeed, random);
        particle.particleGravity = 1.0;
        particle.particleRed = 0.6;
        particle.particleGreen = 0.6;
        particle.particleBlue = 0.6;
        particle.particleScale /= 2.0;
        Self {
            particle,
            sourceState: state,
            sourcePos: BlockPos::new(x.floor() as i32, y.floor() as i32, z.floor() as i32),
            actualModel: None,
        }
    }

    pub fn withActualModel(mut self, actualModel: Option<ParticleActualModel>) -> Self {
        self.actualModel = actualModel;
        self
    }

    pub fn setBlockPos(mut self, pos: BlockPos, colors: &BlockColors, world: &WorldClient) -> Self {
        self.sourcePos = pos;
        if self.sourceState.getBlockId() != 2 {
            let color = colors.colorMultiplier(self.sourceState, world, pos, 0);
            self.particle.particleRed *= ((color >> 16) & 255) as f32 / 255.0;
            self.particle.particleGreen *= ((color >> 8) & 255) as f32 / 255.0;
            self.particle.particleBlue *= (color & 255) as f32 / 255.0;
        }
        self
    }

    pub fn multiplyVelocity(mut self, multiplier: f32) -> Self {
        self.particle.multiplyVelocity(multiplier);
        self
    }

    pub fn multipleParticleScaleBy(mut self, scale: f32) -> Self {
        self.particle.multipleParticleScaleBy(scale);
        self
    }

    pub fn onUpdate(&mut self, world: &WorldClient) {
        self.particle.onUpdate(world);
    }
    pub const fn isExpired(&self) -> bool {
        self.particle.isExpired
    }

    pub fn renderState(&self) -> ParticleDiggingRenderState {
        ParticleDiggingRenderState {
            sourceState: self.sourceState,
            sourcePos: self.sourcePos,
            actualModel: self.actualModel,
            prevPosition: [
                self.particle.prevPosX,
                self.particle.prevPosY,
                self.particle.prevPosZ,
            ],
            position: [self.particle.posX, self.particle.posY, self.particle.posZ],
            textureJitter: [
                self.particle.particleTextureJitterX,
                self.particle.particleTextureJitterY,
            ],
            scale: self.particle.particleScale,
            color: [
                self.particle.particleRed,
                self.particle.particleGreen,
                self.particle.particleBlue,
                self.particle.particleAlpha,
            ],
        }
    }
}
