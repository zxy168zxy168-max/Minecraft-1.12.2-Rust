use uuid::Uuid;

use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// Direct Rust port of MCP 1.12.2 `DefaultPlayerSkin`.
pub struct DefaultPlayerSkin;

impl DefaultPlayerSkin {
    pub fn getDefaultSkinLegacy() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/steve.png")
    }

    pub fn getDefaultSkin(playerUuid: Uuid) -> ResourceLocation {
        if Self::isSlimSkin(playerUuid) {
            ResourceLocation::new("minecraft", "textures/entity/alex.png")
        } else {
            Self::getDefaultSkinLegacy()
        }
    }

    pub fn getSkinType(playerUuid: Uuid) -> &'static str {
        if Self::isSlimSkin(playerUuid) {
            "slim"
        } else {
            "default"
        }
    }

    pub fn isSlimSkin(playerUuid: Uuid) -> bool {
        // java.util.UUID.hashCode(): (int)(most>>32 ^ most ^ least>>32 ^ least)
        let value = playerUuid.as_u128();
        let most = (value >> 64) as u64;
        let least = value as u64;
        let hash = ((most >> 32) ^ most ^ (least >> 32) ^ least) as u32;
        hash & 1 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_skin_type_uses_java_uuid_hash_parity() {
        let uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        assert!(DefaultPlayerSkin::isSlimSkin(uuid));
        assert_eq!(DefaultPlayerSkin::getSkinType(uuid), "slim");
    }
}
