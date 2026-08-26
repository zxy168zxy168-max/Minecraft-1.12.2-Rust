use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderZombieVillager;
impl RenderZombieVillager {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "zombie_villager"
    }
    pub fn texture(entity: &EntityOtherClient) -> ResourceLocation {
        let path = match entity.zombieVillagerProfession() {
            0 => "textures/entity/zombie_villager/zombie_farmer.png",
            1 => "textures/entity/zombie_villager/zombie_librarian.png",
            2 => "textures/entity/zombie_villager/zombie_priest.png",
            3 => "textures/entity/zombie_villager/zombie_smith.png",
            4 => "textures/entity/zombie_villager/zombie_butcher.png",
            _ => "textures/entity/zombie_villager/zombie_villager.png",
        };
        ResourceLocation::new("minecraft", path)
    }
    pub fn allTextures() -> [ResourceLocation; 6] {
        [
            ResourceLocation::new(
                "minecraft",
                "textures/entity/zombie_villager/zombie_villager.png",
            ),
            ResourceLocation::new(
                "minecraft",
                "textures/entity/zombie_villager/zombie_farmer.png",
            ),
            ResourceLocation::new(
                "minecraft",
                "textures/entity/zombie_villager/zombie_librarian.png",
            ),
            ResourceLocation::new(
                "minecraft",
                "textures/entity/zombie_villager/zombie_priest.png",
            ),
            ResourceLocation::new(
                "minecraft",
                "textures/entity/zombie_villager/zombie_smith.png",
            ),
            ResourceLocation::new(
                "minecraft",
                "textures/entity/zombie_villager/zombie_butcher.png",
            ),
        ]
    }
}
