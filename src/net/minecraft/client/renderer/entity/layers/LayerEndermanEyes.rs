use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `LayerEndermanEyes` texture and full-bright lightmap state.
pub struct LayerEndermanEyes;
impl LayerEndermanEyes {
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/enderman/enderman_eyes.png")
    }
    pub const fn packedFullBright() -> u32 {
        (15 << 20) | (15 << 4)
    }
}
