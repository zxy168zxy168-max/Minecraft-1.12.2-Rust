use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderCreeper;

impl RenderCreeper {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "creeper"
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/creeper/creeper.png")
    }
    pub fn chargeTexture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/creeper/creeper_armor.png")
    }

    pub fn flashIntensity(entity: &EntityOtherClient, partialTicks: f32) -> f32 {
        (entity.lastActiveTime as f32
            + (entity.timeSinceIgnited - entity.lastActiveTime) as f32
                * partialTicks.clamp(0.0, 1.0))
            / 28.0
    }

    pub fn scale(entity: &EntityOtherClient, partialTicks: f32) -> [f32; 3] {
        let mut flash = Self::flashIntensity(entity, partialTicks);
        let pulse = 1.0 + (flash * 100.0).sin() * flash * 0.01;
        flash = flash.clamp(0.0, 1.0);
        flash *= flash;
        flash *= flash;
        [
            (1.0 + flash * 0.4) * pulse,
            (1.0 + flash * 0.1) / pulse,
            (1.0 + flash * 0.4) * pulse,
        ]
    }

    pub fn flashColor(entity: &EntityOtherClient, partialTicks: f32) -> Option<[f32; 4]> {
        let flash = Self::flashIntensity(entity, partialTicks);
        if (flash * 10.0) as i32 % 2 == 0 {
            return None;
        }
        let alpha = (flash * 0.2).clamp(0.0, 1.0);
        Some([1.0, 1.0, 1.0, alpha])
    }

    pub fn powered(entity: &EntityOtherClient) -> bool {
        entity.dataManager.boolean(13, false)
    }
}
