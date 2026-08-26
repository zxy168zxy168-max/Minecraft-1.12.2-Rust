use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderWitch;
impl RenderWitch {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "witch"
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/witch.png")
    }
    pub const fn preScale() -> f32 {
        0.9375
    }
    pub fn holdingItem(entity: &EntityOtherClient) -> bool {
        !entity.equipment.getItemStackFromSlot(crate::net::minecraft::inventory::EntityEquipmentSlot::EntityEquipmentSlot::Mainhand).isEmpty()
    }
}
