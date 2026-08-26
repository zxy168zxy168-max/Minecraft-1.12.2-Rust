use std::collections::HashMap;

use crate::net::minecraft::client::audio::AudioBackend::{
    createPlatformBackend, BackendPlayRequest, ListenerTransform, SoundBackend,
};
use crate::net::minecraft::client::audio::PositionedSoundRecord::PositionedSoundRecord;
use crate::net::minecraft::client::audio::Sound::Sound;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::util::SoundCategory::SoundCategory;

#[derive(Debug, Clone)]
struct PlayingSound {
    record: PositionedSoundRecord,
    selected: Sound,
    stopTime: i32,
}

/// Rust ownership equivalent of MCP 1.12.2 `SoundManager`.
///
/// Java's manager calls back into its owning `SoundHandler` to resolve events.
/// Rust cannot safely store that self-reference, so `SoundHandler` resolves the
/// weighted event and passes the concrete OGG resource into this device/channel
/// manager. Channel lifecycle, category volume, repeat delay, pause/resume and
/// listener transforms remain this type's responsibility.
pub struct SoundManager {
    backend: Box<dyn SoundBackend>,
    loaded: bool,
    playTime: i32,
    nextChannel: u64,
    playingSounds: HashMap<u64, PlayingSound>,
    delayedSounds: Vec<(PositionedSoundRecord, i32)>,
    pausedChannels: Vec<u64>,
}

impl std::fmt::Debug for SoundManager {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SoundManager")
            .field("loaded", &self.loaded)
            .field("playTime", &self.playTime)
            .field("playingSounds", &self.playingSounds.len())
            .field("delayedSounds", &self.delayedSounds.len())
            .finish_non_exhaustive()
    }
}

impl SoundManager {
    pub fn new() -> Self {
        Self::withBackend(createPlatformBackend())
    }

    fn withBackend(mut backend: Box<dyn SoundBackend>) -> Self {
        let loaded = backend.isLoaded();
        backend.setMasterVolume(1.0);
        Self {
            backend,
            loaded,
            playTime: 0,
            nextChannel: 1,
            playingSounds: HashMap::new(),
            delayedSounds: Vec::new(),
            pausedChannels: Vec::new(),
        }
    }

    pub const fn isLoaded(&self) -> bool {
        self.loaded
    }
    pub const fn getPlayTime(&self) -> i32 {
        self.playTime
    }

    pub fn reloadSoundSystem(&mut self, soundLevels: &[f32; 10]) {
        self.stopAllSounds();
        self.loaded = self.backend.isLoaded();
        self.backend
            .setMasterVolume(soundLevels[SoundCategory::Master.index()]);
    }

    pub fn playResolvedSound(
        &mut self,
        record: PositionedSoundRecord,
        selected: Sound,
        oggBytes: Vec<u8>,
        soundLevels: &[f32; 10],
    ) -> Option<u64> {
        if !self.loaded || soundLevels[SoundCategory::Master.index()] <= 0.0 {
            return None;
        }

        let resolvedVolume = record.volume * selected.getVolume();
        let attenuationDistance = if resolvedVolume > 1.0 {
            16.0 * resolvedVolume
        } else {
            16.0
        };
        let channelVolume = clamped_category_volume(record.category, resolvedVolume, soundLevels);
        if channelVolume <= 0.0 {
            return None;
        }

        let pitch = (record.pitch * selected.getPitch()).clamp(0.5, 2.0);
        let channel = self.allocateChannel();
        let request = BackendPlayRequest {
            channel,
            oggBytes,
            looping: record.repeat && record.repeatDelay == 0,
            volume: channelVolume,
            pitch,
            position: record.position,
            attenuation: record.attenuationType,
            attenuationDistance,
        };
        if let Err(error) = self.backend.play(request) {
            log::warn!(
                "Unable to start sound {}: {error}",
                selected.getSoundLocation()
            );
            return None;
        }

        self.playingSounds.insert(
            channel,
            PlayingSound {
                record,
                selected,
                // Paulscode may briefly report a new source stopped while its
                // decoder starts. MCP retains it for twenty ticks before cleanup.
                stopTime: self.playTime.saturating_add(20),
            },
        );
        Some(channel)
    }

    pub fn playDelayedSound(&mut self, sound: PositionedSoundRecord, delay: i32) {
        self.delayedSounds
            .push((sound, self.playTime.saturating_add(delay.max(0))));
    }

    /// Advances the MCP sound clock and returns delayed/repeating event records
    /// whose weighted resource must be resolved again by `SoundHandler`.
    pub fn updateAllSounds(&mut self, _soundLevels: &[f32; 10]) -> Vec<PositionedSoundRecord> {
        self.playTime = self.playTime.wrapping_add(1);
        let ended = self
            .playingSounds
            .iter()
            .filter_map(|(&channel, sound)| {
                (!self.backend.isPlaying(channel) && sound.stopTime <= self.playTime)
                    .then_some(channel)
            })
            .collect::<Vec<_>>();

        for channel in ended {
            let Some(sound) = self.playingSounds.remove(&channel) else {
                continue;
            };
            self.backend.remove(channel);
            if sound.record.repeat && sound.record.repeatDelay > 0 {
                let repeatDelay = sound.record.repeatDelay;
                self.delayedSounds
                    .push((sound.record, self.playTime.saturating_add(repeatDelay)));
            }
            self.pausedChannels
                .retain(|candidate| *candidate != channel);
        }

        let mut due = Vec::new();
        self.delayedSounds.retain(|(sound, time)| {
            if self.playTime >= *time {
                due.push(sound.clone());
                false
            } else {
                true
            }
        });
        due
    }

    pub fn isSoundPlaying(&self, event: &ResourceLocation) -> bool {
        self.playingSounds.iter().any(|(&channel, sound)| {
            &sound.record.positionedSoundLocation == event
                && (self.backend.isPlaying(channel) || sound.stopTime > self.playTime)
        })
    }

    pub fn stopSound(&mut self, event: &ResourceLocation) {
        self.stop(Some(event), None);
    }

    pub fn stopRecordAt(&mut self, position: [f32; 3]) {
        let channels = self
            .playingSounds
            .iter()
            .filter_map(|(&channel, sound)| {
                (sound.record.category == SoundCategory::Records
                    && sound.record.position == position)
                    .then_some(channel)
            })
            .collect::<Vec<_>>();
        for channel in channels {
            self.backend.remove(channel);
            self.playingSounds.remove(&channel);
            self.pausedChannels
                .retain(|candidate| *candidate != channel);
        }
    }

    pub fn isChannelPlaying(&self, channel: u64) -> bool {
        self.playingSounds
            .get(&channel)
            .is_some_and(|sound| self.backend.isPlaying(channel) || sound.stopTime > self.playTime)
    }

    pub fn stopChannel(&mut self, channel: u64) {
        self.backend.remove(channel);
        self.playingSounds.remove(&channel);
        self.pausedChannels
            .retain(|candidate| *candidate != channel);
    }

    pub fn stopAllSounds(&mut self) {
        self.backend.stopAll();
        self.playingSounds.clear();
        self.delayedSounds.clear();
        self.pausedChannels.clear();
    }

    pub fn pauseAllSounds(&mut self) {
        self.pausedChannels.clear();
        for &channel in self.playingSounds.keys() {
            if self.backend.isPlaying(channel) {
                self.backend.pause(channel);
                self.pausedChannels.push(channel);
            }
        }
    }

    pub fn resumeAllSounds(&mut self) {
        for channel in self.pausedChannels.drain(..) {
            self.backend.resume(channel);
        }
    }

    pub fn setVolume(&mut self, category: SoundCategory, soundLevels: &[f32; 10]) {
        if category == SoundCategory::Master {
            self.backend.setMasterVolume(soundLevels[category.index()]);
            if soundLevels[category.index()] <= 0.0 {
                self.stopAllSounds();
            }
            return;
        }

        let updates = self
            .playingSounds
            .iter()
            .map(|(&channel, playing)| {
                let resolved = playing.record.volume * playing.selected.getVolume();
                (
                    channel,
                    clamped_category_volume(playing.record.category, resolved, soundLevels),
                )
            })
            .collect::<Vec<_>>();

        let mut remove = Vec::new();
        for (channel, channelVolume) in updates {
            if channelVolume <= 0.0 {
                self.backend.stop(channel);
                remove.push(channel);
            } else {
                self.backend.setVolume(channel, channelVolume);
            }
        }
        for channel in remove {
            self.backend.remove(channel);
            self.playingSounds.remove(&channel);
            self.pausedChannels
                .retain(|candidate| *candidate != channel);
        }
    }

    pub fn stop(&mut self, event: Option<&ResourceLocation>, category: Option<SoundCategory>) {
        if event.is_none() && category.is_none() {
            self.stopAllSounds();
            return;
        }
        let channels = self
            .playingSounds
            .iter()
            .filter_map(|(&channel, sound)| {
                let categoryMatches = category.map_or(true, |value| value == sound.record.category);
                let eventMatches =
                    event.map_or(true, |value| value == &sound.record.positionedSoundLocation);
                (categoryMatches && eventMatches).then_some(channel)
            })
            .collect::<Vec<_>>();
        for channel in channels {
            self.backend.remove(channel);
            self.playingSounds.remove(&channel);
            self.pausedChannels
                .retain(|candidate| *candidate != channel);
        }
        self.delayedSounds.retain(|(sound, _)| {
            !category.map_or(true, |value| value == sound.category)
                || !event.map_or(true, |value| value == &sound.positionedSoundLocation)
        });
    }

    pub fn setListener(&mut self, position: [f32; 3], rotationYaw: f32, rotationPitch: f32) {
        let pitch = rotationPitch.to_radians();
        let yaw = (rotationYaw + 90.0).to_radians();
        let forward = [
            yaw.cos() * (-pitch).cos(),
            (-pitch).sin(),
            yaw.sin() * (-pitch).cos(),
        ];
        let upPitch = (-rotationPitch + 90.0).to_radians();
        let up = [
            yaw.cos() * upPitch.cos(),
            upPitch.sin(),
            yaw.sin() * upPitch.cos(),
        ];
        self.backend.setListener(ListenerTransform {
            position,
            forward,
            up,
        });
    }

    pub fn setSoundPosition(&mut self, channel: u64, position: [f32; 3]) {
        if let Some(sound) = self.playingSounds.get_mut(&channel) {
            sound.record.position = position;
            self.backend.setPosition(channel, position);
        }
    }

    pub fn setSoundVolumePitch(
        &mut self,
        channel: u64,
        volume: f32,
        pitch: f32,
        soundLevels: &[f32; 10],
    ) {
        let Some(sound) = self.playingSounds.get_mut(&channel) else {
            return;
        };
        sound.record.volume = volume;
        sound.record.pitch = pitch;
        let resolvedVolume = volume * sound.selected.getVolume();
        let resolvedPitch = (pitch * sound.selected.getPitch()).clamp(0.5, 2.0);
        let channelVolume =
            clamped_category_volume(sound.record.category, resolvedVolume, soundLevels);
        self.backend.setVolume(channel, channelVolume);
        self.backend.setPitch(channel, resolvedPitch);
    }

    fn allocateChannel(&mut self) -> u64 {
        let channel = self.nextChannel;
        self.nextChannel = self.nextChannel.wrapping_add(1).max(1);
        channel
    }
}

fn clamped_category_volume(
    category: SoundCategory,
    resolvedVolume: f32,
    soundLevels: &[f32; 10],
) -> f32 {
    let categoryVolume = if category == SoundCategory::Master {
        1.0
    } else {
        soundLevels[category.index()]
    };
    (resolvedVolume * categoryVolume).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::client::audio::AudioBackend::{
        BackendPlayRequest, ListenerTransform,
    };
    use crate::net::minecraft::client::audio::PositionedSoundRecord::AttenuationType;
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct MockState {
        playing: HashSet<u64>,
        started: Vec<u64>,
        stopped: Vec<u64>,
        listener: ListenerTransform,
    }

    struct MockBackend(Arc<Mutex<MockState>>);
    impl SoundBackend for MockBackend {
        fn isLoaded(&self) -> bool {
            true
        }
        fn setMasterVolume(&mut self, _volume: f32) {}
        fn play(&mut self, request: BackendPlayRequest) -> Result<(), String> {
            let mut state = self.0.lock().unwrap();
            state.playing.insert(request.channel);
            state.started.push(request.channel);
            Ok(())
        }
        fn isPlaying(&self, channel: u64) -> bool {
            self.0.lock().unwrap().playing.contains(&channel)
        }
        fn stop(&mut self, channel: u64) {
            let mut state = self.0.lock().unwrap();
            state.playing.remove(&channel);
            state.stopped.push(channel);
        }
        fn remove(&mut self, channel: u64) {
            self.0.lock().unwrap().playing.remove(&channel);
        }
        fn stopAll(&mut self) {
            self.0.lock().unwrap().playing.clear();
        }
        fn pause(&mut self, _channel: u64) {}
        fn resume(&mut self, _channel: u64) {}
        fn setVolume(&mut self, _channel: u64, _volume: f32) {}
        fn setPitch(&mut self, _channel: u64, _pitch: f32) {}
        fn setPosition(&mut self, _channel: u64, _position: [f32; 3]) {}
        fn setListener(&mut self, listener: ListenerTransform) {
            self.0.lock().unwrap().listener = listener;
        }
    }

    fn record(repeat: bool, repeatDelay: i32) -> PositionedSoundRecord {
        PositionedSoundRecord::new(
            ResourceLocation::parse("test.event"),
            SoundCategory::Blocks,
            1.0,
            1.0,
            repeat,
            repeatDelay,
            AttenuationType::Linear,
            [1.0, 2.0, 3.0],
        )
    }

    fn selected() -> Sound {
        Sound::new(
            "test/file",
            1.0,
            1.0,
            1,
            crate::net::minecraft::client::audio::Sound::Type::File,
            false,
        )
    }

    #[test]
    fn delayed_repeat_returns_event_for_fresh_resolution() {
        let state = Arc::new(Mutex::new(MockState::default()));
        let mut manager = SoundManager::withBackend(Box::new(MockBackend(state.clone())));
        let channel = manager
            .playResolvedSound(record(true, 3), selected(), Vec::new(), &[1.0; 10])
            .unwrap();
        state.lock().unwrap().playing.remove(&channel);
        for _ in 0..20 {
            assert!(manager.updateAllSounds(&[1.0; 10]).is_empty());
        }
        for _ in 0..2 {
            assert!(manager.updateAllSounds(&[1.0; 10]).is_empty());
        }
        assert_eq!(manager.updateAllSounds(&[1.0; 10]).len(), 1);
    }

    #[test]
    fn listener_basis_matches_minecraft_angles() {
        let state = Arc::new(Mutex::new(MockState::default()));
        let mut manager = SoundManager::withBackend(Box::new(MockBackend(state.clone())));
        manager.setListener([4.0, 5.0, 6.0], 0.0, 0.0);
        let listener = state.lock().unwrap().listener;
        assert_eq!(listener.position, [4.0, 5.0, 6.0]);
        assert!((listener.forward[2] - 1.0).abs() < 0.0001);
        assert!((listener.up[1] - 1.0).abs() < 0.0001);
    }
}
