use std::collections::{BTreeMap, HashMap};

use base64::Engine;
use rsa::pkcs1v15::{Signature as RsaSignature, VerifyingKey};
use rsa::pkcs8::DecodePublicKey;
use rsa::signature::Verifier;
use rsa::RsaPublicKey;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::com::mojang::authlib::properties::Property::Property;
use sha1::Sha1;
use thiserror::Error;
use url::Url;

use crate::com::mojang::authlib::minecraft::MinecraftProfileTexture::{
    MinecraftProfileTexture, TextureType,
};
use crate::com::mojang::authlib::GameProfile::GameProfile;

const JOIN_URL: &str = "https://sessionserver.mojang.com/session/minecraft/join";
const PROFILE_REPOSITORY_URL: &str = "https://api.mojang.com/profiles/minecraft";
const PROFILE_PROPERTIES_URL: &str = "https://sessionserver.mojang.com/session/minecraft/profile";
const YGGDRASIL_PUBLIC_KEY_DER: &[u8] = include_bytes!("../yggdrasil_session_pubkey.der");

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum JoinServerError {
    #[error("authentication servers are unavailable: {0}")]
    AuthenticationUnavailable(String),
    #[error("invalid session: {0}")]
    InvalidCredentials(String),
    #[error("authentication failed: {0}")]
    Authentication(String),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProfileTextureError {
    #[error("signature is missing from textures payload")]
    MissingSignature,
    #[error("textures payload signature is invalid")]
    InvalidSignature,
    #[error("could not decode textures payload: {0}")]
    InvalidPayload(String),
    #[error("textures payload contains a non-whitelisted URL: {0}")]
    NonWhitelistedDomain(String),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProfileLookupError {
    #[error("profile has no player name")]
    MissingName,
    #[error("player profile was not found: {0}")]
    NotFound(String),
    #[error("profile service is unavailable: {0}")]
    Unavailable(String),
    #[error("profile service returned invalid data: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Deserialize)]
struct RawProfileLookup {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct RawProfileProperties {
    id: String,
    name: String,
    #[serde(default)]
    properties: Vec<RawProfileProperty>,
}

#[derive(Debug, Deserialize)]
struct RawProfileProperty {
    name: String,
    value: String,
    signature: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawTexturesPayload {
    #[serde(default)]
    textures: HashMap<String, RawProfileTexture>,
}

#[derive(Debug, Deserialize)]
struct RawProfileTexture {
    url: String,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct MinecraftSessionService;

impl MinecraftSessionService {
    pub const fn new() -> Self {
        Self
    }

    /// Authlib `MinecraftSessionService.joinServer` request used by
    /// `NetHandlerLoginClient.handleEncryptionRequest`.
    pub fn joinServer(
        &self,
        profile: &GameProfile,
        authenticationToken: &str,
        serverId: &str,
    ) -> Result<(), JoinServerError> {
        let profileId = profile.getId().ok_or_else(|| {
            JoinServerError::InvalidCredentials("session profile has no UUID".to_owned())
        })?;
        if authenticationToken.is_empty() {
            return Err(JoinServerError::InvalidCredentials(
                "session access token is empty".to_owned(),
            ));
        }
        let body = json!({
            "accessToken": authenticationToken,
            "selectedProfile": profileId.simple().to_string(),
            "serverId": serverId,
        });
        match ureq::post(JOIN_URL)
            .set("Content-Type", "application/json")
            .send_json(body)
        {
            Ok(response) if matches!(response.status(), 200 | 204) => Ok(()),
            Ok(response) => Err(classify_status(
                response.status(),
                response.into_string().unwrap_or_default(),
            )),
            Err(ureq::Error::Status(status, response)) => Err(classify_status(
                status,
                response.into_string().unwrap_or_default(),
            )),
            Err(ureq::Error::Transport(error)) => Err(JoinServerError::AuthenticationUnavailable(
                error.to_string(),
            )),
        }
    }

    /// `YggdrasilGameProfileRepository#findProfilesByNames` followed by
    /// `YggdrasilMinecraftSessionService#fillProfileProperties`. This is used
    /// by legacy name-only SkullOwner tags and runs only on SkinManager's
    /// worker pool, never on the render thread.
    pub fn completeProfile(
        &self,
        profile: &GameProfile,
        requireSecure: bool,
    ) -> Result<GameProfile, ProfileLookupError> {
        if profile.isComplete()
            && profile
                .getProperties()
                .iter()
                .any(|property| property.getName() == "textures")
        {
            return Ok(profile.clone());
        }
        let base = if let Some(id) = profile.getId() {
            GameProfile::new(Some(id), profile.getName())
        } else {
            self.findProfileByName(profile.getName())?
        };
        self.fillProfileProperties(&base, requireSecure)
    }

    pub fn findProfileByName(&self, name: &str) -> Result<GameProfile, ProfileLookupError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ProfileLookupError::MissingName);
        }
        let response = match ureq::post(PROFILE_REPOSITORY_URL)
            .set("Content-Type", "application/json")
            .send_json(json!([name]))
        {
            Ok(response) => response,
            Err(ureq::Error::Status(status, response)) => {
                return Err(ProfileLookupError::Unavailable(format!(
                    "HTTP {status}: {}",
                    response.into_string().unwrap_or_default(),
                )));
            }
            Err(ureq::Error::Transport(error)) => {
                return Err(ProfileLookupError::Unavailable(error.to_string()));
            }
        };
        let raw: Vec<RawProfileLookup> = response
            .into_json()
            .map_err(|error| ProfileLookupError::InvalidResponse(error.to_string()))?;
        let Some(raw) = raw.into_iter().next() else {
            return Err(ProfileLookupError::NotFound(name.to_owned()));
        };
        let id = Uuid::parse_str(&raw.id)
            .map_err(|error| ProfileLookupError::InvalidResponse(error.to_string()))?;
        Ok(GameProfile::new(Some(id), raw.name))
    }

    pub fn fillProfileProperties(
        &self,
        profile: &GameProfile,
        requireSecure: bool,
    ) -> Result<GameProfile, ProfileLookupError> {
        let id = profile
            .getId()
            .ok_or_else(|| ProfileLookupError::InvalidResponse("profile has no UUID".to_owned()))?;
        let unsigned = if requireSecure { "false" } else { "true" };
        let url = format!(
            "{PROFILE_PROPERTIES_URL}/{}?unsigned={unsigned}",
            id.simple(),
        );
        let response = match ureq::get(&url).call() {
            Ok(response) => response,
            Err(ureq::Error::Status(204 | 404, _)) => {
                return Err(ProfileLookupError::NotFound(profile.getName().to_owned()));
            }
            Err(ureq::Error::Status(status, response)) => {
                return Err(ProfileLookupError::Unavailable(format!(
                    "HTTP {status}: {}",
                    response.into_string().unwrap_or_default(),
                )));
            }
            Err(ureq::Error::Transport(error)) => {
                return Err(ProfileLookupError::Unavailable(error.to_string()));
            }
        };
        let raw: RawProfileProperties = response
            .into_json()
            .map_err(|error| ProfileLookupError::InvalidResponse(error.to_string()))?;
        let rawId = Uuid::parse_str(&raw.id)
            .map_err(|error| ProfileLookupError::InvalidResponse(error.to_string()))?;
        let mut completed = GameProfile::new(Some(rawId), raw.name);
        for property in raw.properties {
            completed.addProperty(Property::new(
                property.name,
                property.value,
                property.signature,
            ));
        }
        Ok(completed)
    }

    /// Authlib 1.5.25 `YggdrasilMinecraftSessionService#getTextures`.
    /// The first `textures` property is signature-checked when requested,
    /// decoded as UTF-8 JSON, and rejected unless every URL belongs to the
    /// `.minecraft.net` or `.mojang.com` suffix whitelist.
    pub fn getTextures(
        &self,
        profile: &GameProfile,
        requireSecure: bool,
    ) -> Result<HashMap<TextureType, MinecraftProfileTexture>, ProfileTextureError> {
        let Some(property) = profile
            .getProperties()
            .iter()
            .find(|property| property.getName() == "textures")
        else {
            return Ok(HashMap::new());
        };

        if requireSecure {
            let signature = property
                .getSignature()
                .ok_or(ProfileTextureError::MissingSignature)?;
            if !verify_texture_signature(property.getValue().as_bytes(), signature) {
                return Err(ProfileTextureError::InvalidSignature);
            }
        }

        let decoded = base64::engine::general_purpose::STANDARD
            .decode(property.getValue())
            .map_err(|error| ProfileTextureError::InvalidPayload(error.to_string()))?;
        let payload: RawTexturesPayload = serde_json::from_slice(&decoded)
            .map_err(|error| ProfileTextureError::InvalidPayload(error.to_string()))?;
        let mut textures = HashMap::new();
        for (raw_type, raw_texture) in payload.textures {
            if !is_whitelisted_domain(&raw_texture.url) {
                return Err(ProfileTextureError::NonWhitelistedDomain(raw_texture.url));
            }
            let texture_type = match raw_type.as_str() {
                "SKIN" => TextureType::Skin,
                "CAPE" => TextureType::Cape,
                "ELYTRA" => TextureType::Elytra,
                _ => continue,
            };
            textures.insert(
                texture_type,
                MinecraftProfileTexture::new(raw_texture.url, raw_texture.metadata),
            );
        }
        Ok(textures)
    }
}

fn verify_texture_signature(value: &[u8], signature_base64: &str) -> bool {
    let Ok(public_key) = RsaPublicKey::from_public_key_der(YGGDRASIL_PUBLIC_KEY_DER) else {
        return false;
    };
    let Ok(signature_bytes) = base64::engine::general_purpose::STANDARD.decode(signature_base64)
    else {
        return false;
    };
    let Ok(signature) = RsaSignature::try_from(signature_bytes.as_slice()) else {
        return false;
    };
    VerifyingKey::<Sha1>::new(public_key)
        .verify(value, &signature)
        .is_ok()
}

fn is_whitelisted_domain(raw_url: &str) -> bool {
    Url::parse(raw_url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .is_some_and(|host| host.ends_with(".minecraft.net") || host.ends_with(".mojang.com"))
}

fn classify_status(status: u16, response: String) -> JoinServerError {
    let message = serde_json::from_str::<serde_json::Value>(&response)
        .ok()
        .and_then(|value| {
            value
                .get("errorMessage")
                .or_else(|| value.get("error"))
                .and_then(|value| value.as_str())
                .map(str::to_owned)
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("HTTP {status}"));
    match status {
        401 | 403 => JoinServerError::InvalidCredentials(message),
        429 | 500..=599 => JoinServerError::AuthenticationUnavailable(message),
        _ => JoinServerError::Authentication(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::com::mojang::authlib::properties::Property::Property;

    #[test]
    fn insecure_profile_texture_payload_preserves_model_metadata() {
        let payload = r#"{"textures":{"SKIN":{"url":"https://textures.minecraft.net/texture/abc","metadata":{"model":"slim"}},"CAPE":{"url":"https://textures.minecraft.net/texture/def"}}}"#;
        let mut profile = GameProfile::new(None, "Test");
        profile.addProperty(Property::new(
            "textures",
            base64::engine::general_purpose::STANDARD.encode(payload),
            None,
        ));
        let textures = MinecraftSessionService::new()
            .getTextures(&profile, false)
            .unwrap();
        assert_eq!(
            textures[&TextureType::Skin].getMetadata("model"),
            Some("slim")
        );
        assert_eq!(textures[&TextureType::Cape].getHash(), "def");
    }

    #[test]
    fn secure_mode_rejects_unsigned_payload() {
        let mut profile = GameProfile::new(None, "Test");
        profile.addProperty(Property::new("textures", "e30=", None));
        assert_eq!(
            MinecraftSessionService::new().getTextures(&profile, true),
            Err(ProfileTextureError::MissingSignature),
        );
    }

    #[test]
    fn texture_domain_whitelist_matches_authlib_suffixes() {
        assert!(is_whitelisted_domain(
            "https://textures.minecraft.net/texture/a"
        ));
        assert!(is_whitelisted_domain("https://assets.mojang.com/a"));
        assert!(!is_whitelisted_domain(
            "https://minecraft.net.evil.invalid/a"
        ));
    }
}
