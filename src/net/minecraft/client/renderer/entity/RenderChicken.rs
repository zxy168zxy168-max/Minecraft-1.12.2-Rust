use crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderChicken;

impl RenderChicken {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "chicken"
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/chicken.png")
    }
}
