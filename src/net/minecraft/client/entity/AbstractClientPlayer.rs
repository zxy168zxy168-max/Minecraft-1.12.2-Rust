use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::net::minecraft::client::network::NetworkPlayerInfo::NetworkPlayerInfo;

/// Client-player base state owned by MCP 1.12.2 `AbstractClientPlayer`.
///
/// The Java class lazily resolves one `NetworkPlayerInfo` and retains that
/// object on the entity. Retention is important for NPC/Bot entities because
/// servers commonly remove their tab-list entry after spawning them while the
/// skin/cape download is still completing.
#[derive(Debug, Clone, PartialEq)]
pub struct AbstractClientPlayer {
    gameProfile: GameProfile,
    playerInfo: Option<NetworkPlayerInfo>,
}

impl AbstractClientPlayer {
    pub fn new(gameProfile: GameProfile) -> Self {
        Self {
            gameProfile,
            playerInfo: None,
        }
    }

    pub fn getGameProfile(&self) -> &GameProfile {
        &self.gameProfile
    }

    /// Rust equivalent of MCP `AbstractClientPlayer#getPlayerInfo` after the
    /// connection lookup has resolved. `NetworkPlayerInfo::clone` shares its
    /// asynchronous texture state through `Arc`, matching Java object identity
    /// for the skin/cape callback lifecycle.
    pub fn setPlayerInfo(&mut self, playerInfo: Option<NetworkPlayerInfo>) {
        self.playerInfo = playerInfo;
    }

    pub fn getPlayerInfo(&self) -> Option<&NetworkPlayerInfo> {
        self.playerInfo.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::com::mojang::authlib::minecraft::MinecraftProfileTexture::{
        MinecraftProfileTexture, TextureType,
    };
    use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
    use crate::net::minecraft::world::GameType::GameType;
    use std::collections::BTreeMap;
    use uuid::Uuid;

    #[test]
    fn retained_player_info_shares_late_texture_callback() {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000082").unwrap();
        let profile = GameProfile::new(Some(id), "RetainedBot");
        let info = NetworkPlayerInfo::new(profile.clone(), GameType::Survival, 0, None);
        let callback_owner = info.clone();
        let mut player = AbstractClientPlayer::new(profile);
        player.setPlayerInfo(Some(info));

        let texture = MinecraftProfileTexture::new(
            "https://textures.minecraft.net/texture/batch82",
            BTreeMap::from([("model".to_owned(), "slim".to_owned())]),
        );
        NetworkPlayerInfo::applyPlayerTexture(
            &callback_owner.textureState(),
            TextureType::Skin,
            ResourceLocation::new("minecraft", "skins/batch82"),
            &texture,
        );

        let retained = player.getPlayerInfo().unwrap();
        assert_eq!(retained.getLocationSkin().getPath(), "skins/batch82");
        assert_eq!(retained.getSkinType(), "slim");
    }
}
