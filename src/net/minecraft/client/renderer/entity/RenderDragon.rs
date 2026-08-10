use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderDragon;
impl RenderDragon {
    pub fn supports(entityType: MobEntityType) -> bool { entityType.registryName == "ender_dragon" }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/enderdragon/dragon.png")
    }
    pub fn explodingTexture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/enderdragon/dragon_exploding.png")
    }
    pub fn eyesTexture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/enderdragon/dragon_eyes.png")
    }
    pub fn beamTexture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/endercrystal/endercrystal_beam.png")
    }
    pub const fn fullBright() -> u32 { (15 << 20) | (15 << 4) }
    pub fn deathAlpha(entity: &EntityOtherClient) -> Option<f32> {
        (entity.dragonDeathTicks > 0).then_some(entity.dragonDeathTicks as f32 / 200.0)
    }
}
