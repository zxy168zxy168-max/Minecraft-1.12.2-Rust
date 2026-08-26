use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// Source-owned parameters for MCP 1.12.2 `LayerCreeperCharge`. The current
/// Vulkan world pass consumes the geometry/texture/light/UV values here; its
/// separate ONE/ONE blend/depth-mask pipeline remains an explicit dependency.
pub struct LayerCreeperCharge;

impl LayerCreeperCharge {
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/creeper/creeper_armor.png")
    }
    pub const fn modelDelta() -> f32 {
        2.0
    }
    pub const fn packedFullBright() -> u32 {
        (15 << 20) | (15 << 4)
    }
    pub const fn tint() -> [f32; 4] {
        [0.5, 0.5, 0.5, 1.0]
    }
    pub fn uvOffset(ageInTicks: f32) -> [f32; 2] {
        [ageInTicks * 0.01, ageInTicks * 0.01]
    }
}
