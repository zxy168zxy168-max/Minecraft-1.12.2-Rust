use crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
pub struct RenderPolarBear;
impl RenderPolarBear {
    pub fn supports(t: MobEntityType) -> bool {
        t.registryName == "polar_bear"
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/bear/polarbear.png")
    }
}
