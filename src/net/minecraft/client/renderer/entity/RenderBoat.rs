use crate::net::minecraft::entity::item::EntityBoat::BoatType;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `RenderBoat` constants and pure transform inputs.
pub struct RenderBoat;

impl RenderBoat {
    pub const SHADOW_SIZE: f32 = 0.5;
    pub const Y_TRANSLATION: f32 = 0.375;

    pub fn texture(boatType: BoatType) -> ResourceLocation {
        let name = match boatType {
            BoatType::Oak => "boat_oak.png",
            BoatType::Spruce => "boat_spruce.png",
            BoatType::Birch => "boat_birch.png",
            BoatType::Jungle => "boat_jungle.png",
            BoatType::Acacia => "boat_acacia.png",
            BoatType::DarkOak => "boat_darkoak.png",
        };
        ResourceLocation::new("minecraft", format!("textures/entity/boat/{name}"))
    }

    pub fn allTextures() -> Vec<ResourceLocation> {
        BoatType::ALL.into_iter().map(Self::texture).collect()
    }

    pub fn damageRotation(
        timeSinceHit: i32,
        damageTaken: f32,
        forwardDirection: i32,
        partialTicks: f32,
    ) -> f32 {
        let f = timeSinceHit as f32 - partialTicks;
        let f1 = (damageTaken - partialTicks).max(0.0);
        if f > 0.0 {
            f.sin() * f * f1 / 10.0 * forwardDirection as f32
        } else {
            0.0
        }
    }
}
