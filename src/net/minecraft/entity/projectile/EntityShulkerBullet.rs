/// Renderer-visible constants and overrides owned by MCP 1.12.2
/// `EntityShulkerBullet`. The synchronized instance itself remains stored in
/// the heterogeneous client entity table.
pub struct EntityShulkerBullet;

impl EntityShulkerBullet {
    pub const WIDTH: f32 = 0.3125;
    pub const HEIGHT: f32 = 0.3125;
    pub const ROTATION_INTERPOLATION: f32 = 0.5;
    pub const FULL_BRIGHT_LIGHT: u32 = 15_728_880;
    pub const MAX_RENDER_DISTANCE_SQUARED: f64 = 16_384.0;

    /// MCP `EntityShulkerBullet#isInRangeToRenderDist`; the argument is the
    /// squared camera distance supplied by `RenderManager`.
    pub fn isInRangeToRenderDist(distanceSquared: f64) -> bool {
        distanceSquared < Self::MAX_RENDER_DISTANCE_SQUARED
    }

    /// MCP `EntityShulkerBullet#getBrightnessForRender`.
    pub const fn getBrightnessForRender() -> u32 {
        Self::FULL_BRIGHT_LIGHT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_dimensions_brightness_and_render_range_are_exact() {
        assert_eq!(
            (EntityShulkerBullet::WIDTH, EntityShulkerBullet::HEIGHT),
            (0.3125, 0.3125)
        );
        assert_eq!(EntityShulkerBullet::getBrightnessForRender(), 15_728_880);
        assert!(EntityShulkerBullet::isInRangeToRenderDist(16_383.999));
        assert!(!EntityShulkerBullet::isInRangeToRenderDist(16_384.0));
    }
}
