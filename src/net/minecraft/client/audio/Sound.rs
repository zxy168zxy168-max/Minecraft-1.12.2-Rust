use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    File,
    SoundEvent,
}

impl Type {
    pub fn getByName(name: &str) -> Option<Self> {
        match name {
            "file" => Some(Self::File),
            "event" => Some(Self::SoundEvent),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sound {
    name: ResourceLocation,
    volume: f32,
    pitch: f32,
    weight: i32,
    soundType: Type,
    streaming: bool,
}

impl Sound {
    pub fn new(
        name: impl AsRef<str>,
        volume: f32,
        pitch: f32,
        weight: i32,
        soundType: Type,
        streaming: bool,
    ) -> Self {
        Self {
            name: ResourceLocation::parse(name),
            volume,
            pitch,
            weight,
            soundType,
            streaming,
        }
    }

    pub fn missing() -> Self {
        Self::new("meta:missing_sound", 1.0, 1.0, 1, Type::File, false)
    }

    pub fn getSoundLocation(&self) -> &ResourceLocation {
        &self.name
    }

    pub fn getSoundAsOggLocation(&self) -> ResourceLocation {
        ResourceLocation::new(
            self.name.getNamespace(),
            format!("sounds/{}.ogg", self.name.getPath()),
        )
    }

    pub const fn getVolume(&self) -> f32 {
        self.volume
    }
    pub const fn getPitch(&self) -> f32 {
        self.pitch
    }
    pub const fn getWeight(&self) -> i32 {
        self.weight
    }
    pub const fn getType(&self) -> Type {
        self.soundType
    }
    pub const fn isStreaming(&self) -> bool {
        self.streaming
    }

    pub fn withEventModifiers(&self, event: &Sound) -> Self {
        Self {
            name: self.name.clone(),
            volume: self.volume * event.volume,
            pitch: self.pitch * event.pitch,
            weight: event.weight,
            soundType: Type::File,
            streaming: self.streaming || event.streaming,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ogg_path_matches_java_resource_layout() {
        let sound = Sound::new("modid:mob/test", 1.0, 1.0, 1, Type::File, false);
        assert_eq!(
            sound.getSoundAsOggLocation().to_string(),
            "modid:sounds/mob/test.ogg"
        );
    }
}
