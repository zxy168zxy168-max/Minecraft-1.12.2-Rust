use crate::net::minecraft::util::EnumParticleTypes::EnumParticleTypes;

/// Client-visible synchronized contract of MCP 1.12.2 `EntityAreaEffectCloud`.
pub struct EntityAreaEffectCloud;

impl EntityAreaEffectCloud {
    pub const RADIUS_INDEX: u8 = 6;
    pub const COLOR_INDEX: u8 = 7;
    pub const IGNORE_RADIUS_INDEX: u8 = 8;
    pub const PARTICLE_INDEX: u8 = 9;
    pub const PARTICLE_PARAM_1_INDEX: u8 = 10;
    pub const PARTICLE_PARAM_2_INDEX: u8 = 11;
    pub const DEFAULT_RADIUS: f32 = 3.0;
    pub const DEFAULT_HEIGHT: f32 = 0.5;
    pub const DEFAULT_COLOR: i32 = 0;
    pub const DEFAULT_SYNC_RADIUS: f32 = 0.5;
    pub const DEFAULT_PARTICLE: EnumParticleTypes = EnumParticleTypes::SpellMob;

    pub const fn width(radius: f32) -> f32 {
        radius * 2.0
    }
    pub const fn particleArea(radius: f32) -> f32 {
        core::f32::consts::PI * radius * radius
    }
    pub const fn colorComponents(color: i32) -> [f64; 3] {
        [
            ((color >> 16) & 255) as f64 / 255.0,
            ((color >> 8) & 255) as f64 / 255.0,
            (color & 255) as f64 / 255.0,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn synchronized_indices_and_dimensions_match_source() {
        assert_eq!(EntityAreaEffectCloud::RADIUS_INDEX, 6);
        assert_eq!(EntityAreaEffectCloud::COLOR_INDEX, 7);
        assert_eq!(EntityAreaEffectCloud::PARTICLE_PARAM_2_INDEX, 11);
        assert_eq!(EntityAreaEffectCloud::width(3.0), 6.0);
        assert_eq!(EntityAreaEffectCloud::DEFAULT_PARTICLE.particleId(), 15);
    }
}
