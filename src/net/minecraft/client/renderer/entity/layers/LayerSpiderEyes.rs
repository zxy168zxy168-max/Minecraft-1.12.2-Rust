use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `LayerSpiderEyes` texture/light owner.
pub struct LayerSpiderEyes;

impl LayerSpiderEyes {
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/spider_eyes.png")
    }
    pub const fn packedFullBright() -> u32 {
        (15 << 20) | (15 << 4)
    }
}
