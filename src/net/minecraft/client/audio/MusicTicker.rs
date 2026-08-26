use std::time::{SystemTime, UNIX_EPOCH};

use crate::compat::Java::JavaRandom;
use crate::net::minecraft::client::audio::PositionedSoundRecord::PositionedSoundRecord;
use crate::net::minecraft::client::audio::SoundHandler::SoundHandler;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicType {
    Menu,
    Game,
    Creative,
    Credits,
    Nether,
    EndBoss,
    End,
}

impl MusicType {
    pub fn musicLocation(self) -> ResourceLocation {
        ResourceLocation::parse(match self {
            Self::Menu => "minecraft:music.menu",
            Self::Game => "minecraft:music.game",
            Self::Creative => "minecraft:music.creative",
            Self::Credits => "minecraft:music.credits",
            Self::Nether => "minecraft:music.nether",
            Self::EndBoss => "minecraft:music.dragon",
            Self::End => "minecraft:music.end",
        })
    }

    pub const fn minDelay(self) -> i32 {
        match self {
            Self::Menu => 20,
            Self::Game => 12_000,
            Self::Creative => 1_200,
            Self::Credits => 0,
            Self::Nether => 1_200,
            Self::EndBoss => 0,
            Self::End => 6_000,
        }
    }

    pub const fn maxDelay(self) -> i32 {
        match self {
            Self::Menu => 600,
            Self::Game => 24_000,
            Self::Creative => 3_600,
            Self::Credits => 0,
            Self::Nether => 3_600,
            Self::EndBoss => 0,
            Self::End => 24_000,
        }
    }
}

/// Port of MCP 1.12.2 `MusicTicker`.
#[derive(Debug, Clone)]
pub struct MusicTicker {
    rand: JavaRandom,
    currentMusic: Option<ResourceLocation>,
    timeUntilNextMusic: i32,
}

impl Default for MusicTicker {
    fn default() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        Self {
            rand: JavaRandom::new(seed),
            currentMusic: None,
            timeUntilNextMusic: 100,
        }
    }
}

impl MusicTicker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, musicType: MusicType, soundHandler: &mut SoundHandler) {
        let requested = musicType.musicLocation();

        if let Some(current) = self.currentMusic.clone() {
            if current != requested {
                soundHandler.stopSound(&current);
                self.timeUntilNextMusic =
                    random_inclusive(&mut self.rand, 0, musicType.minDelay() / 2);
            }

            if !soundHandler.isSoundPlaying(&current) {
                self.currentMusic = None;
                self.timeUntilNextMusic = self.timeUntilNextMusic.min(random_inclusive(
                    &mut self.rand,
                    musicType.minDelay(),
                    musicType.maxDelay(),
                ));
            }
        }

        self.timeUntilNextMusic = self.timeUntilNextMusic.min(musicType.maxDelay());
        if self.currentMusic.is_none() {
            let due = self.timeUntilNextMusic <= 0;
            self.timeUntilNextMusic = self.timeUntilNextMusic.saturating_sub(1);
            if due {
                self.playMusic(musicType, soundHandler);
            }
        }
    }

    pub fn playMusic(&mut self, musicType: MusicType, soundHandler: &mut SoundHandler) {
        let location = musicType.musicLocation();
        soundHandler.playSound(PositionedSoundRecord::getMusicRecord(location.clone()));
        self.currentMusic = Some(location);
        self.timeUntilNextMusic = i32::MAX;
    }

    pub fn currentMusic(&self) -> Option<&ResourceLocation> {
        self.currentMusic.as_ref()
    }
    pub const fn timeUntilNextMusic(&self) -> i32 {
        self.timeUntilNextMusic
    }
}

fn random_inclusive(random: &mut JavaRandom, minimum: i32, maximum: i32) -> i32 {
    if maximum <= minimum {
        minimum
    } else {
        minimum + random.next_i32_bound(maximum - minimum + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn music_type_values_match_mcp_1122() {
        assert_eq!(
            MusicType::Menu.musicLocation(),
            ResourceLocation::parse("minecraft:music.menu")
        );
        assert_eq!(MusicType::Game.minDelay(), 12_000);
        assert_eq!(MusicType::Game.maxDelay(), 24_000);
        assert_eq!(MusicType::EndBoss.maxDelay(), 0);
    }
}
