use crate::net::minecraft::client::entity::EntityOtherClient::ObjectSpawnType;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `RenderFireball`. Both fire-charge fireballs use the
/// `Items.FIRE_CHARGE` particle sprite from the block/item texture atlas and
/// differ only in renderer scale.
pub struct RenderFireball;

impl RenderFireball {
    pub const LARGE_SCALE: f32 = 2.0;
    pub const SMALL_SCALE: f32 = 0.5;

    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/items/fireball.png")
    }

    pub const fn scale(objectType: ObjectSpawnType) -> Option<f32> {
        match objectType {
            ObjectSpawnType::LargeFireball => Some(Self::LARGE_SCALE),
            ObjectSpawnType::SmallFireball => Some(Self::SMALL_SCALE),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_scales_are_not_entity_dimensions() {
        assert_eq!(
            RenderFireball::scale(ObjectSpawnType::LargeFireball),
            Some(2.0)
        );
        assert_eq!(
            RenderFireball::scale(ObjectSpawnType::SmallFireball),
            Some(0.5)
        );
    }
}
