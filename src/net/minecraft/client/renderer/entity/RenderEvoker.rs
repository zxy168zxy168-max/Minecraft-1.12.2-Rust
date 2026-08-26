use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::client::model::ModelIllager::IllagerArmPose;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
pub struct RenderEvoker;
impl RenderEvoker {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "evocation_illager"
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/illager/evoker.png")
    }
    pub const fn preScale() -> f32 {
        0.9375
    }
    pub fn armPose(entity: &EntityOtherClient) -> IllagerArmPose {
        if entity.illagerSpellcasting() {
            IllagerArmPose::Spellcasting
        } else {
            IllagerArmPose::Crossed
        }
    }
    pub fn shouldRenderHeldItem(entity: &EntityOtherClient) -> bool {
        entity.illagerSpellcasting()
    }
}
