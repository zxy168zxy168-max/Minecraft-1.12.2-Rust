use crate::compat::Java::JavaRandom;
use crate::net::minecraft::client::audio::Sound::{Sound, Type};
use crate::net::minecraft::client::audio::SoundRegistry::SoundRegistry;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `SoundEventAccessor`.
///
/// Entries remain unresolved until playback so `type: "event"` registrations
/// observe later resource-pack additions exactly like the Java anonymous
/// `ISoundEventAccessor` implementation.
#[derive(Debug, Clone, PartialEq)]
pub struct SoundEventAccessor {
    accessorList: Vec<Sound>,
    location: ResourceLocation,
    subtitle: Option<String>,
}

impl SoundEventAccessor {
    pub fn new(location: ResourceLocation, subtitle: Option<String>) -> Self {
        Self {
            accessorList: Vec::new(),
            location,
            subtitle,
        }
    }

    pub fn addSound(&mut self, sound: Sound) {
        self.accessorList.push(sound);
    }
    pub fn getLocation(&self) -> &ResourceLocation {
        &self.location
    }
    pub fn getSubtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }
    pub fn sounds(&self) -> &[Sound] {
        &self.accessorList
    }

    pub fn getWeight(&self, registry: &SoundRegistry) -> i32 {
        self.accessorList
            .iter()
            .map(|sound| match sound.getType() {
                Type::File => sound.getWeight(),
                Type::SoundEvent => registry
                    .getObject(sound.getSoundLocation())
                    .map_or(0, |event| event.getWeight(registry)),
            })
            .sum()
    }

    pub fn cloneEntry(&self, registry: &SoundRegistry, random: &mut JavaRandom) -> Sound {
        let total = self.getWeight(registry);
        if self.accessorList.is_empty() || total == 0 {
            return Sound::missing();
        }

        let mut selected = random.next_i32_bound(total);
        for sound in &self.accessorList {
            let weight = match sound.getType() {
                Type::File => sound.getWeight(),
                Type::SoundEvent => registry
                    .getObject(sound.getSoundLocation())
                    .map_or(0, |event| event.getWeight(registry)),
            };
            selected -= weight;
            if selected >= 0 {
                continue;
            }

            return match sound.getType() {
                Type::File => sound.clone(),
                Type::SoundEvent => registry
                    .getObject(sound.getSoundLocation())
                    .map(|event| event.cloneEntry(registry, random).withEventModifiers(sound))
                    .unwrap_or_else(Sound::missing),
            };
        }
        Sound::missing()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_event_multiplies_file_modifiers() {
        let mut registry = SoundRegistry::default();
        let nested_location = ResourceLocation::parse("minecraft:nested");
        let mut nested = SoundEventAccessor::new(nested_location.clone(), None);
        nested.addSound(Sound::new("minecraft:file", 0.5, 0.8, 1, Type::File, false));
        registry.add(nested);

        let root_location = ResourceLocation::parse("minecraft:root");
        let mut root = SoundEventAccessor::new(root_location.clone(), None);
        root.addSound(Sound::new(
            "minecraft:nested",
            0.25,
            2.0,
            3,
            Type::SoundEvent,
            true,
        ));
        registry.add(root);

        let mut random = JavaRandom::new(0);
        let sound = registry
            .getObject(&root_location)
            .unwrap()
            .cloneEntry(&registry, &mut random);
        assert_eq!(sound.getSoundLocation().to_string(), "minecraft:file");
        assert_eq!(sound.getVolume(), 0.125);
        assert_eq!(sound.getPitch(), 1.6);
        assert_eq!(sound.getWeight(), 3);
        assert!(sound.isStreaming());
    }
}
