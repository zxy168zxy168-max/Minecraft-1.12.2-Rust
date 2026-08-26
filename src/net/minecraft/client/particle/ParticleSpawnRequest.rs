use crate::net::minecraft::util::EnumParticleTypes::EnumParticleTypes;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleSpawnRequest {
    pub particleType: EnumParticleTypes,
    pub position: [f64; 3],
    pub speed: [f64; 3],
    pub parameters: [i32; 2],
    pub ignoreRange: bool,
    pub alwaysRender: bool,
}
impl ParticleSpawnRequest {
    pub const fn new(
        particleType: EnumParticleTypes,
        position: [f64; 3],
        speed: [f64; 3],
        parameters: [i32; 2],
    ) -> Self {
        Self {
            particleType,
            position,
            speed,
            parameters,
            ignoreRange: particleType.shouldIgnoreRange(),
            alwaysRender: false,
        }
    }
    pub const fn withVisibility(mut self, ignoreRange: bool, alwaysRender: bool) -> Self {
        self.ignoreRange = self.particleType.shouldIgnoreRange() || ignoreRange;
        self.alwaysRender = alwaysRender;
        self
    }
}
