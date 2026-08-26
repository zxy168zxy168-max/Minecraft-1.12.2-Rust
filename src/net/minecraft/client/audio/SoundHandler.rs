use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use thiserror::Error;

use crate::compat::Java::JavaRandom;
use crate::net::minecraft::client::audio::PositionedSoundRecord::PositionedSoundRecord;
use crate::net::minecraft::client::audio::Sound::{Sound, Type};
use crate::net::minecraft::client::audio::SoundEventAccessor::SoundEventAccessor;
use crate::net::minecraft::client::audio::SoundList::SoundList;
use crate::net::minecraft::client::audio::SoundManager::SoundManager;
use crate::net::minecraft::client::audio::SoundRegistry::SoundRegistry;
use crate::net::minecraft::client::resources::SimpleReloadableResourceManager::{
    ResourceManager, ResourceManagerError,
};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::util::SoundCategory::SoundCategory;

#[derive(Debug, Error)]
pub enum SoundHandlerError {
    #[error("invalid sounds.json in {pack} ({location}): {message}")]
    InvalidSoundsJson {
        pack: String,
        location: ResourceLocation,
        message: String,
    },
}

/// Resource registration half of MCP 1.12.2 `SoundHandler`.
///
/// Audio-device ownership remains in `SoundManager`; this value owns the
/// resource-pack registry and performs the exact `sounds.json` merge order.
#[derive(Debug, Clone, PartialEq)]
pub struct SoundPlayEvent {
    pub subtitle: String,
    pub position: [f32; 3],
    pub startedAtMillis: u128,
}

#[derive(Debug)]
pub struct SoundHandler {
    soundRegistry: SoundRegistry,
    mcResourceManager: ResourceManager,
    random: JavaRandom,
    sndManager: SoundManager,
    soundLevels: [f32; 10],
    pendingSoundEvents: Vec<SoundPlayEvent>,
}

impl SoundHandler {
    pub fn new(manager: ResourceManager) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        let mut handler = Self {
            soundRegistry: SoundRegistry::default(),
            mcResourceManager: manager,
            random: JavaRandom::new(seed),
            sndManager: SoundManager::new(),
            soundLevels: [1.0; 10],
            pendingSoundEvents: Vec::new(),
        };
        handler.onResourceManagerReload();
        handler
    }

    pub fn setResourceManager(&mut self, manager: ResourceManager) {
        self.mcResourceManager = manager;
        self.onResourceManagerReload();
    }

    pub fn onResourceManagerReload(&mut self) {
        self.soundRegistry.clearMap();
        let mut domains = self
            .mcResourceManager
            .get_resource_domains()
            .into_iter()
            .collect::<Vec<_>>();
        // The Java Set has no semantic ordering. Sorting makes tests and logs
        // deterministic without changing per-domain resource-pack precedence.
        domains.sort();

        for namespace in domains {
            let location = ResourceLocation::new(&namespace, "sounds.json");
            let resources = match self.mcResourceManager.get_all_resources(&location) {
                Ok(resources) => resources,
                Err(ResourceManagerError::NotFound(_)) => continue,
                Err(error) => {
                    log::warn!("Could not enumerate {}: {}", location, error);
                    continue;
                }
            };
            for resource in resources {
                match parse_sound_map(&resource.bytes) {
                    Ok(map) => {
                        for (name, sound_list) in map {
                            self.loadSoundResource(
                                ResourceLocation::new(&namespace, name),
                                sound_list,
                            );
                        }
                    }
                    Err(message) => log::warn!(
                        "Invalid sounds.json in {} ({}): {}",
                        resource.pack_name,
                        resource.location,
                        message,
                    ),
                }
            }
        }
        self.sndManager.reloadSoundSystem(&self.soundLevels);
    }

    fn loadSoundResource(&mut self, location: ResourceLocation, sounds: SoundList) {
        let replace = sounds.canReplaceExisting();
        if self.soundRegistry.getObject(&location).is_none() || replace {
            if replace && self.soundRegistry.getObject(&location).is_some() {
                log::debug!("Replaced sound event location {}", location);
            }
            self.soundRegistry.add(SoundEventAccessor::new(
                location.clone(),
                sounds.getSubtitle().map(str::to_owned),
            ));
        }

        for sound in sounds.getSounds() {
            if sound.getType() == Type::File
                && !self
                    .mcResourceManager
                    .resource_exists(&sound.getSoundAsOggLocation())
            {
                log::warn!(
                    "File {} does not exist, cannot add it to event {}",
                    sound.getSoundAsOggLocation(),
                    location,
                );
                continue;
            }
            if let Some(accessor) = self.soundRegistry.getObjectMut(&location) {
                accessor.addSound(sound.clone());
            }
        }
    }

    pub fn playSound(&mut self, sound: PositionedSoundRecord) -> Option<u64> {
        let Some(accessor) = self.soundRegistry.getObject(&sound.positionedSoundLocation) else {
            log::warn!(
                "Unable to play unknown soundEvent: {}",
                sound.positionedSoundLocation,
            );
            return None;
        };
        if let Some(subtitle) = accessor.getSubtitle() {
            self.pendingSoundEvents.push(SoundPlayEvent {
                subtitle: subtitle.to_owned(),
                position: sound.position,
                startedAtMillis: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis(),
            });
        }

        let resolved = self.resolveSound(&sound.positionedSoundLocation);
        if resolved == Sound::missing() {
            log::warn!(
                "Unable to play empty soundEvent: {}",
                sound.positionedSoundLocation,
            );
            return None;
        }
        let ogg = resolved.getSoundAsOggLocation();
        let resource = match self.mcResourceManager.get_resource(&ogg) {
            Ok(resource) => resource,
            Err(error) => {
                log::warn!("Unable to read sound resource {}: {}", ogg, error);
                return None;
            }
        };
        self.sndManager
            .playResolvedSound(sound, resolved, resource.bytes, &self.soundLevels)
    }

    pub fn updateSoundChannel(
        &mut self,
        channel: u64,
        position: [f32; 3],
        volume: f32,
        pitch: f32,
    ) {
        self.sndManager.setSoundPosition(channel, position);
        self.sndManager
            .setSoundVolumePitch(channel, volume, pitch, &self.soundLevels);
    }

    pub fn stopChannel(&mut self, channel: u64) {
        self.sndManager.stopChannel(channel);
    }

    pub fn isChannelPlaying(&self, channel: u64) -> bool {
        self.sndManager.isChannelPlaying(channel)
    }

    pub fn playDelayedSound(&mut self, sound: PositionedSoundRecord, delay: i32) {
        self.sndManager.playDelayedSound(sound, delay);
    }

    pub fn update(&mut self) {
        let due = self.sndManager.updateAllSounds(&self.soundLevels);
        for sound in due {
            self.playSound(sound);
        }
    }

    pub fn setListener(&mut self, position: [f32; 3], rotationYaw: f32, rotationPitch: f32) {
        self.sndManager
            .setListener(position, rotationYaw, rotationPitch);
    }

    pub fn setSoundLevel(&mut self, category: SoundCategory, volume: f32) {
        self.soundLevels[category.index()] = volume.clamp(0.0, 1.0);
        self.sndManager.setVolume(category, &self.soundLevels);
    }

    pub fn setSoundLevels(&mut self, levels: [f32; 10]) {
        for category in SoundCategory::ALL {
            let volume = levels[category.index()].clamp(0.0, 1.0);
            if (self.soundLevels[category.index()] - volume).abs() > f32::EPSILON {
                self.soundLevels[category.index()] = volume;
                self.sndManager.setVolume(category, &self.soundLevels);
            }
        }
    }

    pub fn stopSound(&mut self, event: &ResourceLocation) {
        self.sndManager.stopSound(event);
    }

    pub fn stopRecordAt(&mut self, position: [f32; 3]) {
        self.sndManager.stopRecordAt(position);
    }

    pub fn stop(&mut self, event: Option<&ResourceLocation>, category: Option<SoundCategory>) {
        self.sndManager.stop(event, category);
    }

    pub fn stopSounds(&mut self) {
        self.sndManager.stopAllSounds();
    }

    pub fn pauseSounds(&mut self) {
        self.sndManager.pauseAllSounds();
    }

    pub fn resumeSounds(&mut self) {
        self.sndManager.resumeAllSounds();
    }

    pub fn isSoundPlaying(&self, event: &ResourceLocation) -> bool {
        self.sndManager.isSoundPlaying(event)
    }

    pub fn takeSoundPlayEvents(&mut self) -> Vec<SoundPlayEvent> {
        std::mem::take(&mut self.pendingSoundEvents)
    }

    pub fn soundManager(&self) -> &SoundManager {
        &self.sndManager
    }

    pub fn getAccessor(&self, location: &ResourceLocation) -> Option<&SoundEventAccessor> {
        self.soundRegistry.getObject(location)
    }

    pub fn resolveSound(&mut self, location: &ResourceLocation) -> Sound {
        let registry = &self.soundRegistry;
        let random = &mut self.random;
        registry
            .getObject(location)
            .map(|accessor| accessor.cloneEntry(registry, random))
            .unwrap_or_else(Sound::missing)
    }

    pub fn registry(&self) -> &SoundRegistry {
        &self.soundRegistry
    }
    pub fn resourceManager(&self) -> &ResourceManager {
        &self.mcResourceManager
    }
}

fn parse_sound_map(bytes: &[u8]) -> Result<Vec<(String, SoundList)>, String> {
    let value: Value = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    let object = value
        .as_object()
        .ok_or_else(|| "sounds.json root must be an object".to_owned())?;
    object
        .iter()
        .map(|(name, value)| {
            SoundList::fromJson(value)
                .map(|list| (name.clone(), list))
                .map_err(|message| format!("event {name}: {message}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_assets() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "mc112-sounds-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ));
        fs::create_dir_all(path.join("minecraft/sounds/test")).unwrap();
        path
    }

    #[test]
    fn later_sounds_json_appends_or_replaces_like_fallback_resource_manager() {
        let base = temp_assets();
        let overlay = temp_assets();
        fs::write(base.join("minecraft/sounds/test/base.ogg"), b"ogg").unwrap();
        fs::write(
            base.join("minecraft/sounds.json"),
            br#"{
            "test.event":{"subtitle":"subtitles.test","sounds":["test/base"]}
        }"#,
        )
        .unwrap();
        fs::write(overlay.join("minecraft/sounds/test/overlay.ogg"), b"ogg").unwrap();
        fs::write(
            overlay.join("minecraft/sounds.json"),
            br#"{
            "test.event":{"sounds":["test/overlay"]}
        }"#,
        )
        .unwrap();

        let mut manager = ResourceManager::new();
        manager.add_directory_pack("base", &base);
        manager.add_directory_pack("overlay", &overlay);
        let handler = SoundHandler::new(manager);
        let accessor = handler
            .getAccessor(&ResourceLocation::parse("minecraft:test.event"))
            .unwrap();
        assert_eq!(accessor.sounds().len(), 2);
        assert_eq!(accessor.getSubtitle(), Some("subtitles.test"));

        let _ = fs::remove_dir_all(base);
        let _ = fs::remove_dir_all(overlay);
    }

    #[test]
    fn replace_discards_earlier_event_entries() {
        let base = temp_assets();
        let overlay = temp_assets();
        fs::write(base.join("minecraft/sounds/test/base.ogg"), b"ogg").unwrap();
        fs::write(
            base.join("minecraft/sounds.json"),
            br#"{
            "test.event":{"sounds":["test/base"]}
        }"#,
        )
        .unwrap();
        fs::write(overlay.join("minecraft/sounds/test/overlay.ogg"), b"ogg").unwrap();
        fs::write(
            overlay.join("minecraft/sounds.json"),
            br#"{
            "test.event":{"replace":true,"sounds":["test/overlay"]}
        }"#,
        )
        .unwrap();

        let mut manager = ResourceManager::new();
        manager.add_directory_pack("base", &base);
        manager.add_directory_pack("overlay", &overlay);
        let handler = SoundHandler::new(manager);
        let accessor = handler
            .getAccessor(&ResourceLocation::parse("minecraft:test.event"))
            .unwrap();
        assert_eq!(accessor.sounds().len(), 1);
        assert_eq!(
            accessor.sounds()[0].getSoundLocation().to_string(),
            "minecraft:test/overlay"
        );

        let _ = fs::remove_dir_all(base);
        let _ = fs::remove_dir_all(overlay);
    }
}
