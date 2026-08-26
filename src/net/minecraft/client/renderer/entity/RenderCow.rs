use crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderCow;

impl RenderCow {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "cow"
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/cow/cow.png")
    }
}
