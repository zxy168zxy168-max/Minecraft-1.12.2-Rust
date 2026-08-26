use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderWolf;
impl RenderWolf {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "wolf"
    }
    pub fn texture(entity: &EntityOtherClient) -> ResourceLocation {
        let path = if entity.tameableTamed() {
            "textures/entity/wolf/wolf_tame.png"
        } else if entity.wolfAngry() {
            "textures/entity/wolf/wolf_angry.png"
        } else {
            "textures/entity/wolf/wolf.png"
        };
        ResourceLocation::new("minecraft", path)
    }
    pub fn wetColor(entity: &EntityOtherClient, partialTicks: f32) -> [f32; 4] {
        if !entity.wolfIsWet() {
            return [1.0, 1.0, 1.0, 1.0];
        }
        let shade = entity.wolfShadingWhileWet(partialTicks);
        [shade, shade, shade, 1.0]
    }
    pub fn allTextures() -> [ResourceLocation; 3] {
        [
            ResourceLocation::new("minecraft", "textures/entity/wolf/wolf.png"),
            ResourceLocation::new("minecraft", "textures/entity/wolf/wolf_tame.png"),
            ResourceLocation::new("minecraft", "textures/entity/wolf/wolf_angry.png"),
        ]
    }
}
