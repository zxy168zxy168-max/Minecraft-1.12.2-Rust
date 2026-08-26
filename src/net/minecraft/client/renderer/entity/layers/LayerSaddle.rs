use crate::net::minecraft::client::entity::EntityOtherClient::EntityOtherClient;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `LayerSaddle`: the layer is present only while the pig's
/// synchronized SADDLED DataParameter is true.
pub struct LayerSaddle;

impl LayerSaddle {
    pub fn shouldRender(entity: &EntityOtherClient) -> bool {
        entity.pigSaddled()
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/pig/pig_saddle.png")
    }
    pub const fn modelScale() -> f32 {
        0.5
    }
}
