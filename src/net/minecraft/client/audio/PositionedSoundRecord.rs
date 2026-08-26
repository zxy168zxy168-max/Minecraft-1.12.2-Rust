use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::util::SoundCategory::SoundCategory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttenuationType {
    None,
    Linear,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PositionedSoundRecord {
    pub positionedSoundLocation: ResourceLocation,
    pub category: SoundCategory,
    pub volume: f32,
    pub pitch: f32,
    pub position: [f32; 3],
    pub repeat: bool,
    pub repeatDelay: i32,
    pub attenuationType: AttenuationType,
}

impl PositionedSoundRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        soundId: ResourceLocation,
        category: SoundCategory,
        volume: f32,
        pitch: f32,
        repeat: bool,
        repeatDelay: i32,
        attenuationType: AttenuationType,
        position: [f32; 3],
    ) -> Self {
        Self {
            positionedSoundLocation: soundId,
            category,
            volume,
            pitch,
            position,
            repeat,
            repeatDelay,
            attenuationType,
        }
    }

    pub fn getMasterRecord(sound: ResourceLocation, pitch: f32) -> Self {
        Self::new(
            sound,
            SoundCategory::Master,
            0.25,
            pitch,
            false,
            0,
            AttenuationType::None,
            [0.0; 3],
        )
    }

    pub fn getMusicRecord(sound: ResourceLocation) -> Self {
        Self::new(
            sound,
            SoundCategory::Music,
            1.0,
            1.0,
            false,
            0,
            AttenuationType::None,
            [0.0; 3],
        )
    }

    pub fn getRecordSoundRecord(sound: ResourceLocation, position: [f32; 3]) -> Self {
        Self::new(
            sound,
            SoundCategory::Records,
            4.0,
            1.0,
            false,
            0,
            AttenuationType::Linear,
            position,
        )
    }
}
