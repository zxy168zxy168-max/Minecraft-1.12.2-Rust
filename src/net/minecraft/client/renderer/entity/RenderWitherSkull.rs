use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `RenderWitherSkull` texture and shortest-arc interpolation.
pub struct RenderWitherSkull;

impl RenderWitherSkull {
    pub fn texture(invulnerable: bool) -> ResourceLocation {
        if invulnerable {
            ResourceLocation::new(
                "minecraft",
                "textures/entity/wither/wither_invulnerable.png",
            )
        } else {
            ResourceLocation::new("minecraft", "textures/entity/wither/wither.png")
        }
    }

    pub fn allTextures() -> Vec<ResourceLocation> {
        vec![Self::texture(false), Self::texture(true)]
    }

    pub fn getRenderYaw(previous: f32, current: f32, partialTicks: f32) -> f32 {
        let mut delta = current - previous;
        while delta < -180.0 {
            delta += 360.0;
        }
        while delta >= 180.0 {
            delta -= 360.0;
        }
        previous + partialTicks * delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaw_interpolation_crosses_the_short_arc() {
        assert!((RenderWitherSkull::getRenderYaw(170.0, -170.0, 0.5) - 180.0).abs() < 1.0e-5);
    }
}
