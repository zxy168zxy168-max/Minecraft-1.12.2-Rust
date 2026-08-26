use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
pub struct RenderOcelot;
impl RenderOcelot {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "ocelot"
    }
    pub fn texture(entity: &EntityOtherClient) -> ResourceLocation {
        let path = match entity.ocelotVariant() {
            1 => "textures/entity/cat/black.png",
            2 => "textures/entity/cat/red.png",
            3 => "textures/entity/cat/siamese.png",
            _ => "textures/entity/cat/ocelot.png",
        };
        ResourceLocation::new("minecraft", path)
    }
    pub fn scale(entity: &EntityOtherClient) -> f32 {
        if entity.tameableTamed() {
            0.8
        } else {
            1.0
        }
    }
    pub fn allTextures() -> [ResourceLocation; 4] {
        [
            ResourceLocation::new("minecraft", "textures/entity/cat/ocelot.png"),
            ResourceLocation::new("minecraft", "textures/entity/cat/black.png"),
            ResourceLocation::new("minecraft", "textures/entity/cat/red.png"),
            ResourceLocation::new("minecraft", "textures/entity/cat/siamese.png"),
        ]
    }
}
