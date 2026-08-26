use crate::net::minecraft::client::entity::EntityOtherClient::ObjectSpawnType;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderArrow;

impl RenderArrow {
    pub fn texture(objectType: ObjectSpawnType) -> Option<ResourceLocation> {
        match objectType {
            ObjectSpawnType::TippedArrow => Some(ResourceLocation::new(
                "minecraft",
                "textures/entity/projectiles/arrow.png",
            )),
            ObjectSpawnType::SpectralArrow => Some(ResourceLocation::new(
                "minecraft",
                "textures/entity/projectiles/spectral_arrow.png",
            )),
            _ => None,
        }
    }

    pub fn shakeRotation(arrowShake: i32, partialTicks: f32) -> f32 {
        let shake = arrowShake as f32 - partialTicks;
        if shake > 0.0 {
            -(shake * 3.0).sin() * shake
        } else {
            0.0
        }
    }
}
