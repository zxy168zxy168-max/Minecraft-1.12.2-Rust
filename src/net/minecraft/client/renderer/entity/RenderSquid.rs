use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderSquid;

impl RenderSquid {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "squid"
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/squid.png")
    }
    pub const fn shadowSize() -> f32 {
        0.7
    }

    /// MCP `RenderSquid#handleRotationFloat`.
    pub fn tentacleAngle(entity: &EntityOtherClient, partialTicks: f32) -> f32 {
        let partial = partialTicks.clamp(0.0, 1.0);
        entity.squidLastTentacleAngle
            + (entity.squidTentacleAngle - entity.squidLastTentacleAngle) * partial
    }

    /// MCP `RenderSquid#rotateCorpse` pitch/yaw interpolation.
    pub fn bodyAngles(entity: &EntityOtherClient, partialTicks: f32) -> [f32; 2] {
        let partial = partialTicks.clamp(0.0, 1.0);
        [
            entity.squidPrevPitch + (entity.squidPitch - entity.squidPrevPitch) * partial,
            entity.squidPrevYaw + (entity.squidYaw - entity.squidPrevYaw) * partial,
        ]
    }
}
