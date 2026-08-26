use crate::net::minecraft::client::resources::SimpleReloadableResourceManager::ResourceManager;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::vulkan::TextureSource::TextureSource;

/// MCP 1.12.2 `ColorizerFoliage` backed by the active resource pack.
#[derive(Debug, Clone)]
pub struct ColorizerFoliage {
    foliageBuffer: Vec<i32>,
}

impl ColorizerFoliage {
    pub fn load(manager: &ResourceManager) -> Self {
        let location = ResourceLocation::new("minecraft", "textures/colormap/foliage.png");
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
                log::warn!("failed loading vanilla foliage colorizer: {error}");
                vec![0x48B518; 65_536]
            });
        Self {
            foliageBuffer: buffer,
        }
    }

    pub fn getFoliageColor(&self, temperature: f64, humidity: f64) -> i32 {
        let humidity = humidity * temperature;
        let i = ((1.0 - temperature) * 255.0) as i32;
        let j = ((1.0 - humidity) * 255.0) as i32;
        let index =
            (j << 8 | i).clamp(0, self.foliageBuffer.len().saturating_sub(1) as i32) as usize;
        self.foliageBuffer
            .get(index)
            .copied()
            .unwrap_or(Self::getFoliageColorBasic())
    }

    pub const fn getFoliageColorPine() -> i32 {
        6_396_257
    }
    pub const fn getFoliageColorBirch() -> i32 {
        8_431_445
    }
    pub const fn getFoliageColorBasic() -> i32 {
        4_764_952
    }
}
