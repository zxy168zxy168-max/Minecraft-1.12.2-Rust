use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `RenderShulkerBullet`.
pub struct RenderShulkerBullet;

impl RenderShulkerBullet {
    pub const MODEL_Y_OFFSET: f32 = 0.15;
    pub const OUTER_SCALE: f32 = 1.5;
    pub const OUTER_ALPHA: f32 = 0.5;

    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/shulker/spark.png")
    }

    pub fn rotLerp(previous: f32, current: f32, partialTicks: f32) -> f32 {
        let mut difference = current - previous;
        while difference < -180.0 {
            difference += 360.0;
        }
        while difference >= 180.0 {
            difference -= 360.0;
        }
        previous + partialTicks.clamp(0.0, 1.0) * difference
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_lerp_uses_shortest_arc() {
        assert!((RenderShulkerBullet::rotLerp(170.0, -170.0, 0.5) - 180.0).abs() < 1.0e-5);
    }
}
