use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `RenderEnderman` state owner. Geometry is provided by
/// `ModelEnderman`; the Vulkan/OpenGL frame builder applies the same main
/// texture, eyes layer and held-block layer.
pub struct RenderEnderman;

impl RenderEnderman {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "enderman"
    }

    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/enderman/enderman.png")
    }

    pub const fn shadowSize() -> f32 { 0.5 }

    pub fn carrying(entity: &EntityOtherClient) -> bool {
        entity.endermanHeldBlockStateId().is_some()
    }

    pub fn attacking(entity: &EntityOtherClient) -> bool {
        entity.endermanScreaming()
    }
}
