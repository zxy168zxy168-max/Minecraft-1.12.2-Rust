use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::com::mojang::authlib::minecraft::MinecraftProfileTexture::{
    MinecraftProfileTexture, TextureType,
};
use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::net::minecraft::client::resources::DefaultPlayerSkin::DefaultPlayerSkin;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::world::GameType::GameType;

#[derive(Debug, Default)]
pub struct PlayerTextureState {
    playerTextures: HashMap<TextureType, ResourceLocation>,
    playerTexturesLoaded: bool,
    skinType: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NetworkPlayerInfo {
    gameProfile: GameProfile,
    gameType: GameType,
    responseTime: i32,
    displayName: Option<ITextComponent>,
    textureState: Arc<RwLock<PlayerTextureState>>,
    lastHealth: i32,
    displayHealth: i32,
    lastHealthTime: i64,
    healthBlinkTime: i64,
    renderVisibilityId: i64,
}

impl PartialEq for NetworkPlayerInfo {
    fn eq(&self, other: &Self) -> bool {
        self.gameProfile == other.gameProfile
            && self.gameType == other.gameType
            && self.responseTime == other.responseTime
            && self.displayName == other.displayName
    }
}

impl Eq for NetworkPlayerInfo {}

impl NetworkPlayerInfo {
    pub fn new(
        gameProfile: GameProfile,
        gameType: GameType,
        responseTime: i32,
        displayName: Option<ITextComponent>,
    ) -> Self {
        Self {
            gameProfile,
            gameType,
            responseTime,
            displayName,
            textureState: Arc::new(RwLock::new(PlayerTextureState::default())),
            lastHealth: 0,
            displayHealth: 0,
            lastHealthTime: 0,
            healthBlinkTime: 0,
            renderVisibilityId: 0,
        }
    }

    pub fn getGameProfile(&self) -> &GameProfile {
        &self.gameProfile
    }
    pub const fn getGameType(&self) -> GameType {
        self.gameType
    }
    pub const fn getResponseTime(&self) -> i32 {
        self.responseTime
    }
    pub fn getDisplayName(&self) -> Option<&ITextComponent> {
        self.displayName.as_ref()
    }
    pub fn setGameType(&mut self, value: GameType) {
        self.gameType = value;
    }
    pub fn setResponseTime(&mut self, value: i32) {
        self.responseTime = value;
    }
    pub fn setDisplayName(&mut self, value: Option<ITextComponent>) {
        self.displayName = value;
    }

    pub fn hasLocationSkin(&self) -> bool {
        self.textureState
            .read()
            .unwrap_or_else(|poison| poison.into_inner())
            .playerTextures
            .contains_key(&TextureType::Skin)
    }

    pub fn getSkinType(&self) -> String {
        let state = self
            .textureState
            .read()
            .unwrap_or_else(|poison| poison.into_inner());
        state.skinType.clone().unwrap_or_else(|| {
            self.gameProfile
                .getId()
                .map(DefaultPlayerSkin::getSkinType)
                .unwrap_or("default")
                .to_owned()
        })
    }

    pub fn getLocationSkin(&self) -> ResourceLocation {
        let state = self
            .textureState
            .read()
            .unwrap_or_else(|poison| poison.into_inner());
        state
            .playerTextures
            .get(&TextureType::Skin)
            .cloned()
            .unwrap_or_else(|| {
                self.gameProfile
                    .getId()
                    .map(DefaultPlayerSkin::getDefaultSkin)
                    .unwrap_or_else(DefaultPlayerSkin::getDefaultSkinLegacy)
            })
    }

    pub fn getLocationCape(&self) -> Option<ResourceLocation> {
        self.textureState
            .read()
            .unwrap_or_else(|poison| poison.into_inner())
            .playerTextures
            .get(&TextureType::Cape)
            .cloned()
    }

    pub fn getLocationElytra(&self) -> Option<ResourceLocation> {
        self.textureState
            .read()
            .unwrap_or_else(|poison| poison.into_inner())
            .playerTextures
            .get(&TextureType::Elytra)
            .cloned()
    }

    /// Equivalent to the synchronized guard in
    /// `NetworkPlayerInfo#loadPlayerTextures`.
    pub(crate) fn beginPlayerTexturesLoad(&self) -> bool {
        let mut state = self
            .textureState
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        if state.playerTexturesLoaded {
            false
        } else {
            state.playerTexturesLoaded = true;
            true
        }
    }

    pub(crate) fn textureState(&self) -> Arc<RwLock<PlayerTextureState>> {
        Arc::clone(&self.textureState)
    }

    pub(crate) fn applyPlayerTexture(
        state: &Arc<RwLock<PlayerTextureState>>,
        textureType: TextureType,
        location: ResourceLocation,
        profileTexture: &MinecraftProfileTexture,
    ) {
        let mut state = state.write().unwrap_or_else(|poison| poison.into_inner());
        state.playerTextures.insert(textureType, location);
        if textureType == TextureType::Skin {
            state.skinType = Some(
                profileTexture
                    .getMetadata("model")
                    .unwrap_or("default")
                    .to_owned(),
            );
        }
    }

    pub const fn getLastHealth(&self) -> i32 {
        self.lastHealth
    }
    pub fn setLastHealth(&mut self, value: i32) {
        self.lastHealth = value;
    }
    pub const fn getDisplayHealth(&self) -> i32 {
        self.displayHealth
    }
    pub fn setDisplayHealth(&mut self, value: i32) {
        self.displayHealth = value;
    }
    pub const fn getLastHealthTime(&self) -> i64 {
        self.lastHealthTime
    }
    pub fn setLastHealthTime(&mut self, value: i64) {
        self.lastHealthTime = value;
    }
    pub const fn getHealthBlinkTime(&self) -> i64 {
        self.healthBlinkTime
    }
    pub fn setHealthBlinkTime(&mut self, value: i64) {
        self.healthBlinkTime = value;
    }
    pub const fn getRenderVisibilityId(&self) -> i64 {
        self.renderVisibilityId
    }
    pub fn setRenderVisibilityId(&mut self, value: i64) {
        self.renderVisibilityId = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use uuid::Uuid;

    #[test]
    fn cloned_player_info_shares_async_texture_callback_state() {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let info = NetworkPlayerInfo::new(
            GameProfile::new(Some(id), "Alex"),
            GameType::Survival,
            0,
            None,
        );
        let clone = info.clone();
        let texture = MinecraftProfileTexture::new(
            "https://textures.minecraft.net/texture/abc",
            BTreeMap::from([("model".to_owned(), "slim".to_owned())]),
        );
        NetworkPlayerInfo::applyPlayerTexture(
            &info.textureState(),
            TextureType::Skin,
            ResourceLocation::new("minecraft", "skins/abc"),
            &texture,
        );
        assert_eq!(clone.getLocationSkin().getPath(), "skins/abc");
        assert_eq!(clone.getSkinType(), "slim");
    }
}
