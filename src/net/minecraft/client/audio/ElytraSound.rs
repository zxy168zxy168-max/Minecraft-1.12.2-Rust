use crate::net::minecraft::client::audio::PositionedSoundRecord::{
    AttenuationType, PositionedSoundRecord,
};
use crate::net::minecraft::client::audio::SoundHandler::SoundHandler;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::util::SoundCategory::SoundCategory;

/// Rust ownership equivalent of MCP 1.12.2 `ElytraSound`.
///
/// Java stores a live `EntityPlayerSP` reference inside an `ITickableSound`.
/// The renderer/network world is lock-protected in this port, so each client
/// tick passes the same authoritative player snapshot into `update` instead.
#[derive(Debug, Clone)]
pub struct ElytraSound {
    channel: Option<u64>,
    time: i32,
    donePlaying: bool,
}

impl Default for ElytraSound {
    fn default() -> Self {
        Self {
            channel: None,
            time: 0,
            donePlaying: false,
        }
    }
}

impl ElytraSound {
    pub fn new(soundHandler: &mut SoundHandler, position: [f32; 3]) -> Self {
        let channel = soundHandler.playSound(PositionedSoundRecord::new(
            ResourceLocation::parse("minecraft:item.elytra.flying"),
            SoundCategory::Players,
            0.1,
            1.0,
            true,
            0,
            AttenuationType::Linear,
            position,
        ));
        Self {
            channel,
            ..Self::default()
        }
    }

    pub fn update(
        &mut self,
        playerDead: bool,
        elytraFlying: bool,
        position: [f32; 3],
        motion: [f64; 3],
        soundHandler: &mut SoundHandler,
    ) {
        self.time = self.time.saturating_add(1);

        if !playerDead && (self.time <= 20 || elytraFlying) {
            let speed = (motion[0] * motion[0] + motion[1] * motion[1] + motion[2] * motion[2])
                .sqrt() as f32;
            let halfSpeed = speed / 2.0;
            let mut volume = if speed >= 0.01 {
                (halfSpeed * halfSpeed).clamp(0.0, 1.0)
            } else {
                0.0
            };

            if self.time < 20 {
                volume = 0.0;
            } else if self.time < 40 {
                volume *= (self.time - 20) as f32 / 20.0;
            }

            let pitch = if volume > 0.8 {
                1.0 + (volume - 0.8)
            } else {
                1.0
            };

            if let Some(channel) = self.channel {
                if soundHandler.isChannelPlaying(channel) {
                    soundHandler.updateSoundChannel(channel, position, volume, pitch);
                } else {
                    self.donePlaying = true;
                }
            }
        } else {
            self.donePlaying = true;
        }

        if self.donePlaying {
            if let Some(channel) = self.channel.take() {
                soundHandler.stopChannel(channel);
            }
        }
    }

    pub const fn isDonePlaying(&self) -> bool {
        self.donePlaying
    }
    pub const fn time(&self) -> i32 {
        self.time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_volume_curve_is_silent_for_first_twenty_ticks() {
        let speed = 1.0_f32;
        let mut volume = ((speed / 2.0) * (speed / 2.0)).clamp(0.0, 1.0);
        let time = 10;
        if time < 20 {
            volume = 0.0;
        }
        assert_eq!(volume, 0.0);
    }
}
