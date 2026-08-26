use crate::net::minecraft::client::audio::PositionedSoundRecord::AttenuationType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ListenerTransform {
    pub position: [f32; 3],
    pub forward: [f32; 3],
    pub up: [f32; 3],
}

impl Default for ListenerTransform {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            forward: [0.0, 0.0, 1.0],
            up: [0.0, 1.0, 0.0],
        }
    }
}

#[derive(Debug)]
pub struct BackendPlayRequest {
    pub channel: u64,
    pub oggBytes: Vec<u8>,
    pub looping: bool,
    pub volume: f32,
    pub pitch: f32,
    pub position: [f32; 3],
    pub attenuation: AttenuationType,
    pub attenuationDistance: f32,
}

pub trait SoundBackend {
    fn isLoaded(&self) -> bool;
    fn setMasterVolume(&mut self, volume: f32);
    fn play(&mut self, request: BackendPlayRequest) -> Result<(), String>;
    fn isPlaying(&self, channel: u64) -> bool;
    fn stop(&mut self, channel: u64);
    fn remove(&mut self, channel: u64);
    fn stopAll(&mut self);
    fn pause(&mut self, channel: u64);
    fn resume(&mut self, channel: u64);
    fn setVolume(&mut self, channel: u64, volume: f32);
    fn setPitch(&mut self, channel: u64, pitch: f32);
    fn setPosition(&mut self, channel: u64, position: [f32; 3]);
    fn setListener(&mut self, listener: ListenerTransform);
}

pub fn createPlatformBackend() -> Box<dyn SoundBackend> {
    #[cfg(windows)]
    {
        match WindowsAudioBackend::new() {
            Ok(backend) => Box::new(backend),
            Err(error) => {
                log::error!("Unable to start Minecraft sound engine: {error}");
                Box::new(UnavailableAudioBackend)
            }
        }
    }
    #[cfg(not(windows))]
    {
        log::info!("Minecraft audio output is disabled on this non-Windows build");
        Box::new(UnavailableAudioBackend)
    }
}

struct UnavailableAudioBackend;

impl SoundBackend for UnavailableAudioBackend {
    fn isLoaded(&self) -> bool {
        false
    }
    fn setMasterVolume(&mut self, _volume: f32) {}
    fn play(&mut self, _request: BackendPlayRequest) -> Result<(), String> {
        Err("audio backend is unavailable".to_owned())
    }
    fn isPlaying(&self, _channel: u64) -> bool {
        false
    }
    fn stop(&mut self, _channel: u64) {}
    fn remove(&mut self, _channel: u64) {}
    fn stopAll(&mut self) {}
    fn pause(&mut self, _channel: u64) {}
    fn resume(&mut self, _channel: u64) {}
    fn setVolume(&mut self, _channel: u64, _volume: f32) {}
    fn setPitch(&mut self, _channel: u64, _pitch: f32) {}
    fn setPosition(&mut self, _channel: u64, _position: [f32; 3]) {}
    fn setListener(&mut self, _listener: ListenerTransform) {}
}

#[cfg(windows)]
mod windows_backend {
    use std::collections::HashMap;
    use std::io::{BufReader, Cursor};
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };
    use std::time::Duration;

    use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

    use super::{AttenuationType, BackendPlayRequest, ListenerTransform, SoundBackend};

    #[derive(Debug)]
    struct StereoGains {
        left: AtomicU32,
        right: AtomicU32,
    }

    impl StereoGains {
        fn new() -> Self {
            Self {
                left: AtomicU32::new(1.0_f32.to_bits()),
                right: AtomicU32::new(1.0_f32.to_bits()),
            }
        }

        fn set(&self, left: f32, right: f32) {
            self.left.store(left.to_bits(), Ordering::Relaxed);
            self.right.store(right.to_bits(), Ordering::Relaxed);
        }

        fn get(&self) -> (f32, f32) {
            (
                f32::from_bits(self.left.load(Ordering::Relaxed)),
                f32::from_bits(self.right.load(Ordering::Relaxed)),
            )
        }
    }

    /// OpenAL in Minecraft 1.12.2 receives positional sources as mono and
    /// applies linear attenuation. This adapter performs the same two visible
    /// operations before the samples reach WASAPI: source frames are downmixed
    /// to mono, then emitted as stereo with continuously updateable gains.
    struct LinearSpatialSource {
        input: Box<dyn Source<Item = f32> + Send>,
        gains: Arc<StereoGains>,
        pendingRight: Option<f32>,
    }

    impl LinearSpatialSource {
        fn new(input: Box<dyn Source<Item = f32> + Send>, gains: Arc<StereoGains>) -> Self {
            Self {
                input,
                gains,
                pendingRight: None,
            }
        }
    }

    impl Iterator for LinearSpatialSource {
        type Item = f32;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(right) = self.pendingRight.take() {
                return Some(right);
            }
            let channels = self.input.channels().max(1) as usize;
            let mut mixed = 0.0_f32;
            for _ in 0..channels {
                mixed += self.input.next()?;
            }
            mixed /= channels as f32;
            let (leftGain, rightGain) = self.gains.get();
            self.pendingRight = Some(mixed * rightGain);
            Some(mixed * leftGain)
        }
    }

    impl Source for LinearSpatialSource {
        fn current_frame_len(&self) -> Option<usize> {
            self.input.current_frame_len().map(|samples| {
                let channels = self.input.channels().max(1) as usize;
                samples / channels * 2
            })
        }

        fn channels(&self) -> u16 {
            2
        }
        fn sample_rate(&self) -> u32 {
            self.input.sample_rate()
        }
        fn total_duration(&self) -> Option<Duration> {
            self.input.total_duration()
        }
    }

    struct Channel {
        sink: Sink,
        baseVolume: f32,
        position: [f32; 3],
        attenuation: AttenuationType,
        attenuationDistance: f32,
        gains: Option<Arc<StereoGains>>,
    }

    pub(super) struct WindowsAudioBackend {
        _stream: OutputStream,
        streamHandle: OutputStreamHandle,
        channels: HashMap<u64, Channel>,
        listener: ListenerTransform,
        masterVolume: f32,
    }

    impl WindowsAudioBackend {
        pub(super) fn new() -> Result<Self, String> {
            let (stream, streamHandle) =
                OutputStream::try_default().map_err(|error| error.to_string())?;
            Ok(Self {
                _stream: stream,
                streamHandle,
                channels: HashMap::new(),
                listener: ListenerTransform::default(),
                masterVolume: 1.0,
            })
        }

        fn refreshChannelSpatial(listener: ListenerTransform, channel: &Channel) {
            let Some(gains) = channel.gains.as_ref() else {
                return;
            };
            if channel.attenuation == AttenuationType::None {
                gains.set(1.0, 1.0);
                return;
            }

            let relative = [
                channel.position[0] - listener.position[0],
                channel.position[1] - listener.position[1],
                channel.position[2] - listener.position[2],
            ];
            let distance = length(relative);
            let attenuation = if channel.attenuationDistance <= 0.0 {
                0.0
            } else {
                (1.0 - distance / channel.attenuationDistance).clamp(0.0, 1.0)
            };
            let direction = if distance > f32::EPSILON {
                [
                    relative[0] / distance,
                    relative[1] / distance,
                    relative[2] / distance,
                ]
            } else {
                [0.0; 3]
            };
            let right = normalize(cross(listener.forward, listener.up));
            let pan = dot(direction, right).clamp(-1.0, 1.0);
            // Centered mono reaches both channels at full amplitude. Moving a
            // source to one side reduces only the opposite channel, matching
            // the channel-volume behavior of the 1.12.2 OpenAL path.
            let left = attenuation * (1.0 - pan).clamp(0.0, 1.0);
            let right = attenuation * (1.0 + pan).clamp(0.0, 1.0);
            gains.set(left, right);
        }

        fn applyChannelVolume(masterVolume: f32, channel: &Channel) {
            channel
                .sink
                .set_volume((masterVolume * channel.baseVolume).max(0.0));
        }
    }

    impl SoundBackend for WindowsAudioBackend {
        fn isLoaded(&self) -> bool {
            true
        }

        fn setMasterVolume(&mut self, volume: f32) {
            self.masterVolume = volume.clamp(0.0, 1.0);
            for channel in self.channels.values() {
                Self::applyChannelVolume(self.masterVolume, channel);
            }
        }

        fn play(&mut self, request: BackendPlayRequest) -> Result<(), String> {
            let sink = Sink::try_new(&self.streamHandle).map_err(|error| error.to_string())?;
            let cursor = BufReader::new(Cursor::new(request.oggBytes));
            let decoded: Box<dyn Source<Item = f32> + Send> = if request.looping {
                Box::new(
                    Decoder::new_looped(cursor)
                        .map_err(|error| error.to_string())?
                        .convert_samples::<f32>(),
                )
            } else {
                Box::new(
                    Decoder::new(cursor)
                        .map_err(|error| error.to_string())?
                        .convert_samples::<f32>(),
                )
            };

            let gains = if request.attenuation == AttenuationType::Linear {
                let gains = Arc::new(StereoGains::new());
                sink.append(LinearSpatialSource::new(decoded, gains.clone()));
                Some(gains)
            } else {
                sink.append(decoded);
                None
            };
            sink.set_speed(request.pitch.clamp(0.5, 2.0));
            let channel = Channel {
                sink,
                baseVolume: request.volume.clamp(0.0, 1.0),
                position: request.position,
                attenuation: request.attenuation,
                attenuationDistance: request.attenuationDistance,
                gains,
            };
            Self::refreshChannelSpatial(self.listener, &channel);
            Self::applyChannelVolume(self.masterVolume, &channel);
            channel.sink.play();
            if let Some(replaced) = self.channels.insert(request.channel, channel) {
                replaced.sink.stop();
            }
            Ok(())
        }

        fn isPlaying(&self, channel: u64) -> bool {
            self.channels
                .get(&channel)
                .is_some_and(|entry| !entry.sink.empty())
        }

        fn stop(&mut self, channel: u64) {
            if let Some(entry) = self.channels.get(&channel) {
                entry.sink.stop();
            }
        }

        fn remove(&mut self, channel: u64) {
            if let Some(entry) = self.channels.remove(&channel) {
                entry.sink.stop();
            }
        }

        fn stopAll(&mut self) {
            for (_, entry) in self.channels.drain() {
                entry.sink.stop();
            }
        }

        fn pause(&mut self, channel: u64) {
            if let Some(entry) = self.channels.get(&channel) {
                entry.sink.pause();
            }
        }

        fn resume(&mut self, channel: u64) {
            if let Some(entry) = self.channels.get(&channel) {
                entry.sink.play();
            }
        }

        fn setVolume(&mut self, channel: u64, volume: f32) {
            if let Some(entry) = self.channels.get_mut(&channel) {
                entry.baseVolume = volume.clamp(0.0, 1.0);
                Self::applyChannelVolume(self.masterVolume, entry);
            }
        }

        fn setPitch(&mut self, channel: u64, pitch: f32) {
            if let Some(entry) = self.channels.get(&channel) {
                entry.sink.set_speed(pitch.clamp(0.5, 2.0));
            }
        }

        fn setPosition(&mut self, channel: u64, position: [f32; 3]) {
            if let Some(entry) = self.channels.get_mut(&channel) {
                entry.position = position;
                Self::refreshChannelSpatial(self.listener, entry);
            }
        }

        fn setListener(&mut self, listener: ListenerTransform) {
            self.listener = listener;
            for channel in self.channels.values() {
                Self::refreshChannelSpatial(listener, channel);
            }
        }
    }

    fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }
    fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }
    fn length(value: [f32; 3]) -> f32 {
        dot(value, value).sqrt()
    }
    fn normalize(value: [f32; 3]) -> [f32; 3] {
        let length = length(value);
        if length <= f32::EPSILON {
            [1.0, 0.0, 0.0]
        } else {
            [value[0] / length, value[1] / length, value[2] / length]
        }
    }
}

#[cfg(windows)]
use windows_backend::WindowsAudioBackend;
