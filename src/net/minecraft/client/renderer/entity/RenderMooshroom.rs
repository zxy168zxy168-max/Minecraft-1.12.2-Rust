use crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
pub struct RenderMooshroom;
impl RenderMooshroom {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "mooshroom"
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/cow/mooshroom.png")
    }
}
