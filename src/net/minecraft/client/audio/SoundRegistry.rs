use std::collections::HashMap;

use crate::net::minecraft::client::audio::SoundEventAccessor::SoundEventAccessor;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

#[derive(Debug, Clone, Default)]
pub struct SoundRegistry {
    soundRegistry: HashMap<ResourceLocation, SoundEventAccessor>,
}

impl SoundRegistry {
    pub fn add(&mut self, accessor: SoundEventAccessor) {
        self.soundRegistry
            .insert(accessor.getLocation().clone(), accessor);
    }
    pub fn getObject(&self, location: &ResourceLocation) -> Option<&SoundEventAccessor> {
        self.soundRegistry.get(location)
    }
    pub fn getObjectMut(&mut self, location: &ResourceLocation) -> Option<&mut SoundEventAccessor> {
        self.soundRegistry.get_mut(location)
    }
    pub fn clearMap(&mut self) {
        self.soundRegistry.clear();
    }
    pub fn getKeys(&self) -> impl Iterator<Item = &ResourceLocation> {
        self.soundRegistry.keys()
    }
    pub fn len(&self) -> usize {
        self.soundRegistry.len()
    }
}
