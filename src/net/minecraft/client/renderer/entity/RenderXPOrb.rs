use crate::net::minecraft::client::renderer::entity::Render::RenderProperties;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderXPOrb;

impl RenderXPOrb {
    pub const PROPERTIES: RenderProperties = RenderProperties::new(0.15, 0.75);

    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/experience_orb.png")
    }

    pub const fn getTextureByXP(xpValue: i16) -> i32 {
        if xpValue >= 2477 {
            10
        } else if xpValue >= 1237 {
            9
        } else if xpValue >= 617 {
            8
        } else if xpValue >= 307 {
            7
        } else if xpValue >= 149 {
            6
        } else if xpValue >= 73 {
            5
        } else if xpValue >= 37 {
            4
        } else if xpValue >= 17 {
            3
        } else if xpValue >= 7 {
            2
        } else if xpValue >= 3 {
            1
        } else {
            0
        }
    }

    pub fn textureCoordinates(xpValue: i16) -> [f32; 4] {
        let index = Self::getTextureByXP(xpValue);
        [
            (index % 4 * 16) as f32 / 64.0,
            (index / 4 * 16) as f32 / 64.0,
            (index % 4 * 16 + 16) as f32 / 64.0,
            (index / 4 * 16 + 16) as f32 / 64.0,
        ]
    }

    pub fn color(xpColor: i32, partialTicks: f32) -> [f32; 4] {
        let timer = (xpColor as f32 + partialTicks) / 2.0;
        let red = ((timer.sin() + 1.0) * 0.5 * 255.0) / 255.0;
        let blue = (((timer + 4.1887903).sin() + 1.0) * 0.1 * 255.0) / 255.0;
        [red, 1.0, blue, 128.0 / 255.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xp_sprite_thresholds_match_entity_xp_orb() {
        assert_eq!(RenderXPOrb::getTextureByXP(1), 0);
        assert_eq!(RenderXPOrb::getTextureByXP(3), 1);
        assert_eq!(RenderXPOrb::getTextureByXP(2477), 10);
    }
}
