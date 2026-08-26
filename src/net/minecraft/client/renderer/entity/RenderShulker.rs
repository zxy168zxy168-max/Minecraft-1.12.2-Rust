use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShulkerTransformOp {
    Translate([f32; 3]),
    Rotate { degrees: f32, axis: [f32; 3] },
}

/// MCP 1.12.2 `RenderShulker` and its private `HeadLayer`.
pub struct RenderShulker;

impl RenderShulker {
    pub const PRE_SCALE: f32 = 0.999;
    pub const TELEPORT_INTERPOLATION_TICKS: i32 = 6;

    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "shulker"
    }

    pub fn texture(entity: &EntityOtherClient) -> ResourceLocation {
        Self::allTextures()[entity.shulkerColorMetadata() as usize].clone()
    }

    pub fn allTextures() -> Vec<ResourceLocation> {
        [
            "white",
            "orange",
            "magenta",
            "light_blue",
            "yellow",
            "lime",
            "pink",
            "gray",
            "silver",
            "cyan",
            "purple",
            "blue",
            "brown",
            "green",
            "red",
            "black",
        ]
        .into_iter()
        .map(|name| {
            ResourceLocation::new(
                "minecraft",
                format!("textures/entity/shulker/shulker_{name}.png"),
            )
        })
        .collect()
    }

    /// Translation applied by `doRender` while the six-tick client teleport
    /// interpolation is active. The entity itself already occupies the new
    /// attachment block; rendering moves quadratically back toward the old one.
    pub fn teleportRenderOffset(entity: &EntityOtherClient, partialTicks: f32) -> [f32; 3] {
        let Some(current) = entity.shulkerAttachmentPos() else {
            return [0.0; 3];
        };
        let Some(old) = entity.shulkerOldAttachPos() else {
            return [0.0; 3];
        };
        let ticks = entity.shulkerClientTeleportInterp();
        if ticks <= 0 || !entity.shulkerIsAttachedToBlock() {
            return [0.0; 3];
        }
        let mut fraction = (ticks as f32 - partialTicks.clamp(0.0, 1.0))
            / Self::TELEPORT_INTERPOLATION_TICKS as f32;
        fraction *= fraction;
        [
            -(current.x - old.x) as f32 * fraction,
            -(current.y - old.y) as f32 * fraction,
            -(current.z - old.z) as f32 * fraction,
        ]
    }

    /// Operations appended by `RenderShulker#rotateCorpse` after the inherited
    /// body-yaw rotation and before `prepareScale`.
    pub fn corpseTransform(facing: EnumFacing) -> &'static [ShulkerTransformOp] {
        use ShulkerTransformOp::{Rotate, Translate};
        match facing {
            EnumFacing::Down => &[],
            EnumFacing::East => &[
                Translate([0.5, 0.5, 0.0]),
                Rotate {
                    degrees: 90.0,
                    axis: [1.0, 0.0, 0.0],
                },
                Rotate {
                    degrees: 90.0,
                    axis: [0.0, 0.0, 1.0],
                },
            ],
            EnumFacing::West => &[
                Translate([-0.5, 0.5, 0.0]),
                Rotate {
                    degrees: 90.0,
                    axis: [1.0, 0.0, 0.0],
                },
                Rotate {
                    degrees: -90.0,
                    axis: [0.0, 0.0, 1.0],
                },
            ],
            EnumFacing::North => &[
                Translate([0.0, 0.5, -0.5]),
                Rotate {
                    degrees: 90.0,
                    axis: [1.0, 0.0, 0.0],
                },
            ],
            EnumFacing::South => &[
                Translate([0.0, 0.5, 0.5]),
                Rotate {
                    degrees: 90.0,
                    axis: [1.0, 0.0, 0.0],
                },
                Rotate {
                    degrees: 180.0,
                    axis: [0.0, 0.0, 1.0],
                },
            ],
            EnumFacing::Up => &[
                Translate([0.0, 1.0, 0.0]),
                Rotate {
                    degrees: 180.0,
                    axis: [1.0, 0.0, 0.0],
                },
            ],
        }
    }

    /// Operations appended by `RenderShulker.HeadLayer#doRenderLayer` after
    /// the living renderer's scale/translation matrix.
    pub fn headLayerTransform(facing: EnumFacing) -> &'static [ShulkerTransformOp] {
        use ShulkerTransformOp::{Rotate, Translate};
        match facing {
            EnumFacing::Down => &[],
            EnumFacing::East => &[
                Rotate {
                    degrees: 90.0,
                    axis: [0.0, 0.0, 1.0],
                },
                Rotate {
                    degrees: 90.0,
                    axis: [1.0, 0.0, 0.0],
                },
                Translate([1.0, -1.0, 0.0]),
                Rotate {
                    degrees: 180.0,
                    axis: [0.0, 1.0, 0.0],
                },
            ],
            EnumFacing::West => &[
                Rotate {
                    degrees: -90.0,
                    axis: [0.0, 0.0, 1.0],
                },
                Rotate {
                    degrees: 90.0,
                    axis: [1.0, 0.0, 0.0],
                },
                Translate([-1.0, -1.0, 0.0]),
                Rotate {
                    degrees: 180.0,
                    axis: [0.0, 1.0, 0.0],
                },
            ],
            EnumFacing::North => &[
                Rotate {
                    degrees: 90.0,
                    axis: [1.0, 0.0, 0.0],
                },
                Translate([0.0, -1.0, -1.0]),
            ],
            EnumFacing::South => &[
                Rotate {
                    degrees: 180.0,
                    axis: [0.0, 0.0, 1.0],
                },
                Rotate {
                    degrees: 90.0,
                    axis: [1.0, 0.0, 0.0],
                },
                Translate([0.0, -1.0, 1.0]),
            ],
            EnumFacing::Up => &[
                Rotate {
                    degrees: 180.0,
                    axis: [1.0, 0.0, 0.0],
                },
                Translate([0.0, -2.0, 0.0]),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_all_sixteen_dye_textures_in_metadata_order() {
        let textures = RenderShulker::allTextures();
        assert_eq!(textures.len(), 16);
        assert_eq!(
            textures[0].getPath(),
            "textures/entity/shulker/shulker_white.png"
        );
        assert_eq!(
            textures[10].getPath(),
            "textures/entity/shulker/shulker_purple.png"
        );
        assert_eq!(
            textures[15].getPath(),
            "textures/entity/shulker/shulker_black.png"
        );
    }
}
