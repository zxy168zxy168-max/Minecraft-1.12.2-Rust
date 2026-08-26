use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::client::model::ModelIllager::IllagerArmPose;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
pub struct RenderVindicator;
impl RenderVindicator {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "vindication_illager"
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/illager/vindicator.png")
    }
    pub const fn preScale() -> f32 {
        0.9375
    }
    pub fn armPose(entity: &EntityOtherClient) -> IllagerArmPose {
        if entity.illagerAttacking() {
            IllagerArmPose::Attacking
        } else {
            IllagerArmPose::Crossed
        }
    }
    pub fn shouldRenderHeldItem(entity: &EntityOtherClient) -> bool {
        entity.illagerAttacking()
    }
}
