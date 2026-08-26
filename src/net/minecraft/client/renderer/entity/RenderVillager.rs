use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderVillager;

impl RenderVillager {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "villager"
    }

    pub fn texture(entity: &EntityOtherClient) -> ResourceLocation {
        let path = match entity.villagerProfession() {
            0 => "textures/entity/villager/farmer.png",
            1 => "textures/entity/villager/librarian.png",
            2 => "textures/entity/villager/priest.png",
            3 => "textures/entity/villager/smith.png",
            4 => "textures/entity/villager/butcher.png",
            _ => "textures/entity/villager/villager.png",
        };
        ResourceLocation::new("minecraft", path)
    }

    pub fn preScale(entity: &EntityOtherClient) -> f32 {
        if entity.isChild() {
            0.9375 * 0.5
        } else {
            0.9375
        }
    }

    pub fn allTextures() -> [ResourceLocation; 6] {
        [
            ResourceLocation::new("minecraft", "textures/entity/villager/villager.png"),
            ResourceLocation::new("minecraft", "textures/entity/villager/farmer.png"),
            ResourceLocation::new("minecraft", "textures/entity/villager/librarian.png"),
            ResourceLocation::new("minecraft", "textures/entity/villager/priest.png"),
            ResourceLocation::new("minecraft", "textures/entity/villager/smith.png"),
            ResourceLocation::new("minecraft", "textures/entity/villager/butcher.png"),
        ]
    }
}
