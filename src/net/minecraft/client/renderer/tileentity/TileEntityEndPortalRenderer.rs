use crate::compat::Java::JavaRandom;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// One pass of MCP 1.12.2 `TileEntityEndPortalRenderer`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndPortalLayer {
    pub index: i32,
    pub color: [f32; 4],
    pub additive: bool,
}

pub struct TileEntityEndPortalRenderer;

impl TileEntityEndPortalRenderer {
    pub const SURFACE_HEIGHT: f32 = 0.75;

    pub fn endSkyTexture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/environment/end_sky.png")
    }

    pub fn endPortalTexture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/end_portal.png")
    }

    /// Exact distance thresholds from `func_191286_a`.
    pub const fn layerCount(distanceSquared: f64) -> i32 {
        if distanceSquared > 36_864.0 {
            1
        } else if distanceSquared > 25_600.0 {
            3
        } else if distanceSquared > 16_384.0 {
            5
        } else if distanceSquared > 9_216.0 {
            7
        } else if distanceSquared > 4_096.0 {
            9
        } else if distanceSquared > 1_024.0 {
            11
        } else if distanceSquared > 576.0 {
            13
        } else if distanceSquared > 256.0 {
            14
        } else {
            15
        }
    }

    /// Replays the class-level `Random(31100L)` sequence used for pass colors.
    pub fn layers(distanceSquared: f64) -> Vec<EndPortalLayer> {
        let count = Self::layerCount(distanceSquared);
        let mut random = JavaRandom::new(31_100);
        (0..count)
            .map(|index| {
                let intensity = if index == 0 {
                    0.15
                } else {
                    2.0 / (18 - index) as f32
                };
                EndPortalLayer {
                    index,
                    color: [
                        (random.next_f32() * 0.5 + 0.1) * intensity,
                        (random.next_f32() * 0.5 + 0.4) * intensity,
                        (random.next_f32() * 0.5 + 0.5) * intensity,
                        1.0,
                    ],
                    additive: index >= 1,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_thresholds_match_mcp() {
        assert_eq!(TileEntityEndPortalRenderer::layerCount(0.0), 15);
        assert_eq!(TileEntityEndPortalRenderer::layerCount(257.0), 14);
        assert_eq!(TileEntityEndPortalRenderer::layerCount(36_865.0), 1);
    }

    #[test]
    fn first_layer_is_alpha_then_additive() {
        let layers = TileEntityEndPortalRenderer::layers(0.0);
        assert!(!layers[0].additive);
        assert!(layers[1].additive);
        assert_eq!(layers.len(), 15);
    }
}
