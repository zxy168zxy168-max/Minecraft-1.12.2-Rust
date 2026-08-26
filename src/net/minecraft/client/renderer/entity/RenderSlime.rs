use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderSlime;
impl RenderSlime {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "slime"
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/slime/slime.png")
    }
    pub fn scale(entity: &EntityOtherClient, partialTicks: f32) -> [f32; 3] {
        let size = entity.slimeSize() as f32;
        let squish = (entity.prevSquishFactor
            + (entity.squishFactor - entity.prevSquishFactor) * partialTicks.clamp(0.0, 1.0))
            / (size * 0.5 + 1.0);
        let inv = 1.0 / (squish + 1.0);
        [
            inv * size * 0.999,
            (1.0 / inv) * size * 0.999,
            inv * size * 0.999,
        ]
    }
}
