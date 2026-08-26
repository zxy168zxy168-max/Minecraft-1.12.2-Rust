use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderMagmaCube;
impl RenderMagmaCube {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "magma_cube"
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/slime/magmacube.png")
    }
    pub fn interpolatedSquish(entity: &EntityOtherClient, partialTicks: f32) -> f32 {
        (entity.prevSquishFactor
            + (entity.squishFactor - entity.prevSquishFactor) * partialTicks.clamp(0.0, 1.0))
        .max(0.0)
    }
    pub fn scale(entity: &EntityOtherClient, partialTicks: f32) -> [f32; 3] {
        let size = entity.slimeSize() as f32;
        let squish = Self::interpolatedSquish(entity, partialTicks) / (size * 0.5 + 1.0);
        let inv = 1.0 / (squish + 1.0);
        [inv * size, (1.0 / inv) * size, inv * size]
    }
}
