use crate::net::minecraft::client::resources::SimpleReloadableResourceManager::ResourceManager;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::vulkan::TextureSource::TextureSource;

/// MCP 1.12.2 `ColorizerGrass` backed by the active resource pack's
/// `textures/colormap/grass.png`.
#[derive(Debug, Clone)]
pub struct ColorizerGrass {
    grassBuffer: Vec<i32>,
}

impl ColorizerGrass {
    pub fn load(manager: &ResourceManager) -> Self {
        let location = ResourceLocation::new("minecraft", "textures/colormap/grass.png");
        let buffer = TextureSource::load(manager, &location)
            .map(|source| {
                source
                    .image
                    .rgba()
                    .chunks_exact(4)
                    .map(|pixel| {
                        ((pixel[0] as i32) << 16) | ((pixel[1] as i32) << 8) | pixel[2] as i32
                    })
                    .collect()
            })
            .unwrap_or_else(|error| {
                log::warn!("failed loading vanilla grass colorizer: {error}");
                vec![0x91BD59; 65_536]
            });
        Self {
            grassBuffer: buffer,
        }
    }

    pub fn getGrassColor(&self, temperature: f64, humidity: f64) -> i32 {
        let humidity = humidity * temperature;
        let i = ((1.0 - temperature) * 255.0) as i32;
        let j = ((1.0 - humidity) * 255.0) as i32;
        let index = (j << 8 | i).clamp(0, self.grassBuffer.len().saturating_sub(1) as i32) as usize;
        self.grassBuffer.get(index).copied().unwrap_or(-65_281)
    }
}
