use crate::net::minecraft::util::EnumFacing::EnumFacing;

/// Renderer-independent placement contract from MCP 1.12.2
/// `TileEntitySkullRenderer#renderSkull`. Vulkan owns the matrix and mesh,
/// while this class owns the source translation/yaw switch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkullRenderPlacement {
    /// Translation relative to the skull block origin.
    pub translation: [f32; 3],
    /// Model yaw in degrees passed to `ModelBase#render`.
    pub yaw: f32,
}

pub struct TileEntitySkullRenderer;

impl TileEntitySkullRenderer {
    pub fn getPlacement(facing: EnumFacing, rotation: i32) -> SkullRenderPlacement {
        let rotation_yaw = (rotation & 15) as f32 * 360.0 / 16.0;
        match facing {
            EnumFacing::Up => SkullRenderPlacement {
                translation: [0.5, 0.0, 0.5],
                yaw: rotation_yaw,
            },
            EnumFacing::North => SkullRenderPlacement {
                translation: [0.5, 0.25, 0.74],
                // Vanilla deliberately leaves the supplied rotation untouched.
                yaw: rotation_yaw,
            },
            EnumFacing::South => SkullRenderPlacement {
                translation: [0.5, 0.25, 0.26],
                yaw: 180.0,
            },
            EnumFacing::West => SkullRenderPlacement {
                translation: [0.74, 0.25, 0.5],
                yaw: 270.0,
            },
            // MCP switch `case EAST: default:` also covers DOWN/corrupt meta.
            EnumFacing::East | EnumFacing::Down => SkullRenderPlacement {
                translation: [0.26, 0.25, 0.5],
                yaw: 90.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_and_north_keep_tile_rotation() {
        assert_eq!(
            TileEntitySkullRenderer::getPlacement(EnumFacing::Up, 4),
            SkullRenderPlacement {
                translation: [0.5, 0.0, 0.5],
                yaw: 90.0
            },
        );
        assert_eq!(
            TileEntitySkullRenderer::getPlacement(EnumFacing::North, 12),
            SkullRenderPlacement {
                translation: [0.5, 0.25, 0.74],
                yaw: 270.0
            },
        );
    }

    #[test]
    fn other_walls_override_rotation_and_default_to_east() {
        assert_eq!(
            TileEntitySkullRenderer::getPlacement(EnumFacing::South, 7).yaw,
            180.0
        );
        assert_eq!(
            TileEntitySkullRenderer::getPlacement(EnumFacing::West, 7).yaw,
            270.0
        );
        assert_eq!(
            TileEntitySkullRenderer::getPlacement(EnumFacing::East, 7).yaw,
            90.0
        );
        assert_eq!(
            TileEntitySkullRenderer::getPlacement(EnumFacing::Down, 7).translation,
            [0.26, 0.25, 0.5]
        );
    }
}
