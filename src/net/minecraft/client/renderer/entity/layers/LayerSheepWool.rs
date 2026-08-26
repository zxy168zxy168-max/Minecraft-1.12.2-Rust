use crate::net::minecraft::client::entity::EntityOtherClient::EntityOtherClient;
use crate::net::minecraft::client::renderer::entity::RenderSheep::RenderSheep;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `LayerSheepWool`, including the synchronized sheared bit and
/// the `jeb_` color interpolation. OptiFine CustomColors remains outside this
/// vanilla layer until explicitly requested.
pub struct LayerSheepWool;

impl LayerSheepWool {
    pub fn shouldRender(entity: &EntityOtherClient) -> bool {
        !entity.sheepSheared()
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/sheep/sheep_fur.png")
    }
    pub fn color(entity: &EntityOtherClient, partialTicks: f32) -> [f32; 4] {
        if entity.customName() == Some("jeb_") {
            RenderSheep::jebColor(entity.entityId, entity.entity.ticksExisted, partialTicks)
        } else {
            RenderSheep::woolColor(entity.sheepFleeceColor())
        }
    }
}
