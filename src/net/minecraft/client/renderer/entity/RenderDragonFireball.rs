use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `RenderDragonFireball`.
pub struct RenderDragonFireball;

impl RenderDragonFireball {
    pub const SCALE: f32 = 2.0;

    pub fn texture() -> ResourceLocation {
        ResourceLocation::new(
            "minecraft",
            "textures/entity/enderdragon/dragon_fireball.png",
        )
    }
}
