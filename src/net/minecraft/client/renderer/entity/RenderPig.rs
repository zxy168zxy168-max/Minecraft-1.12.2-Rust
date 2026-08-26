use crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderPig;

impl RenderPig {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "pig"
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/pig/pig.png")
    }
}
