use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::client::model::ModelIllager::IllagerArmPose;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
pub struct RenderIllusionIllager;
impl RenderIllusionIllager {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "illusion_illager"
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/illager/illusionist.png")
    }
    pub const fn preScale() -> f32 {
        0.9375
    }
    pub fn armPose(entity: &EntityOtherClient) -> IllagerArmPose {
        if entity.illagerSpellcasting() {
            IllagerArmPose::Spellcasting
        } else if entity.illagerAttacking() {
            IllagerArmPose::BowAndArrow
        } else {
            IllagerArmPose::Crossed
        }
    }
    pub fn shouldRenderHeldItem(entity: &EntityOtherClient) -> bool {
        entity.illagerSpellcasting() || entity.illagerAttacking()
    }
    pub const fn showHood() -> bool {
        true
    }
    /// `EntityIllusionIllager.getRenderBoundingBox`: keep displaced copies in
    /// the frustum even when the source body is near a camera edge.
    pub fn renderBoundingBox(entity: &EntityOtherClient) -> AxisAlignedBB {
        entity.entity.boundingBox.expand(3.0, 0.0, 3.0)
    }
}
