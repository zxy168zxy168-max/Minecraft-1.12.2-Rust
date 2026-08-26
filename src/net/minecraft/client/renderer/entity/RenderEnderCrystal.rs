use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// Renderer constants and source animation calculations from MCP 1.12.2
/// `RenderEnderCrystal`. Vulkan submission remains outside this class just as
/// OpenGL submission remains outside the entity class in vanilla.
pub struct RenderEnderCrystal;

impl RenderEnderCrystal {
    pub const SHADOW_SIZE: f32 = 0.5;

    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/endercrystal/endercrystal.png")
    }

    pub fn beamTexture() -> ResourceLocation {
        ResourceLocation::new(
            "minecraft",
            "textures/entity/endercrystal/endercrystal_beam.png",
        )
    }

    /// Returns `(f, f1)` from `doRender`: the interpolated inner rotation and
    /// the squared-plus-linear bob term used by both model and beam origin.
    pub fn animation(innerRotation: i32, partialTicks: f32) -> (f32, f32) {
        let f = innerRotation as f32 + partialTicks;
        let mut f1 = (f * 0.2).sin() / 2.0 + 0.5;
        f1 = f1 * f1 + f1;
        (f, f1)
    }
}
