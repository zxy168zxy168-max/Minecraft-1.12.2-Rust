use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use serde::Deserialize;
use thiserror::Error;

use crate::net::minecraft::client::resources::SimpleReloadableResourceManager::{
    ResourceManager, ResourceManagerError,
};
use crate::vulkan::NativeImage::{NativeImage, NativeImageError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextureSampling {
    /// `TextureMetadataSection.textureBlur` in MCP.
    pub blur: bool,
    /// `TextureMetadataSection.textureClamp` in MCP.
    pub clamp: bool,
}

/// One entry from MCP `AnimationMetadataSection`. `time` is measured in
/// client ticks and is always at least one after validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureAnimationFrame {
    pub index: u32,
    pub time: u32,
}

/// Runtime form of MCP `AnimationMetadataSection` used by TextureMap-backed
/// Vulkan sprites. The image keeps the vertically stacked source frames; this
/// descriptor only resolves the frame selected for a client tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureAnimation {
    pub frames: Vec<TextureAnimationFrame>,
    pub interpolate: bool,
}

impl TextureAnimation {
    pub fn frame_index_at_tick(&self, tick: i64) -> u32 {
        if self.frames.is_empty() {
            return 0;
        }
        let cycle = self
            .frames
            .iter()
            .fold(0_u64, |sum, frame| {
                sum.saturating_add(frame.time.max(1) as u64)
            })
            .max(1);
        let mut cursor = tick.rem_euclid(cycle as i64) as u64;
        for frame in &self.frames {
            let duration = frame.time.max(1) as u64;
            if cursor < duration {
                return frame.index;
            }
            cursor -= duration;
        }
        self.frames.last().map_or(0, |frame| frame.index)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureSource {
    pub requested_location: ResourceLocation,
    pub source_pack: String,
    pub image: NativeImage,
    pub sampling: TextureSampling,
    pub animation: Option<TextureAnimation>,
    pub missing: bool,
}

#[derive(Debug, Error)]
pub enum TextureSourceError {
    #[error(transparent)]
    Resource(#[from] ResourceManagerError),
    #[error(transparent)]
    Image(#[from] NativeImageError),
    #[error("invalid animation metadata for {location}: {message}")]
    AnimationMetadata {
        location: ResourceLocation,
        message: String,
    },
}

#[derive(Debug, Deserialize, Default)]
struct MetadataRoot {
    #[serde(default)]
    texture: TextureMetadata,
    animation: Option<RawAnimationMetadata>,
}

#[derive(Debug, Deserialize, Default)]
struct TextureMetadata {
    #[serde(default)]
    blur: bool,
    #[serde(default)]
    clamp: bool,
}

#[derive(Debug, Deserialize, Default)]
struct RawAnimationMetadata {
    #[serde(default = "default_frame_time")]
    frametime: u32,
    #[serde(default)]
    interpolate: bool,
    #[serde(default)]
    frames: Vec<RawAnimationFrame>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawAnimationFrame {
    Index(u32),
    Timed { index: u32, time: u32 },
}

const fn default_frame_time() -> u32 {
    1
}

fn build_animation(
    location: &ResourceLocation,
    image: &NativeImage,
    raw: RawAnimationMetadata,
) -> Result<TextureAnimation, TextureSourceError> {
    if raw.frametime == 0 {
        return Err(TextureSourceError::AnimationMetadata {
            location: location.clone(),
            message: "frametime must be at least 1".to_owned(),
        });
    }
    let frame_width = image.width().max(1);
    if image.height() % frame_width != 0 {
        return Err(TextureSourceError::AnimationMetadata {
            location: location.clone(),
            message: format!(
                "animated sprite height {} is not divisible by frame width {}",
                image.height(),
                frame_width,
            ),
        });
    }
    let available = (image.height() / frame_width).max(1);
    let frames = if raw.frames.is_empty() {
        (0..available)
            .map(|index| TextureAnimationFrame {
                index,
                time: raw.frametime,
            })
            .collect::<Vec<_>>()
    } else {
        let mut frames = Vec::with_capacity(raw.frames.len());
        for frame in raw.frames {
            let (index, time) = match frame {
                RawAnimationFrame::Index(index) => (index, raw.frametime),
                RawAnimationFrame::Timed { index, time } => (index, time),
            };
            if time == 0 {
                return Err(TextureSourceError::AnimationMetadata {
                    location: location.clone(),
                    message: "frame time must be at least 1".to_owned(),
                });
            }
            if index >= available {
                return Err(TextureSourceError::AnimationMetadata {
                    location: location.clone(),
                    message: format!(
                        "frame index {index} exceeds available frame count {available}"
                    ),
                });
            }
            frames.push(TextureAnimationFrame { index, time });
        }
        frames
    };
    Ok(TextureAnimation {
        frames,
        interpolate: raw.interpolate,
    })
}

impl TextureSource {
    /// Semantic port of `SimpleTexture.loadTexture`: load the selected resource,
    /// decode PNG pixels, and apply the optional `texture` metadata section.
    pub fn load(
        manager: &ResourceManager,
        location: &ResourceLocation,
    ) -> Result<Self, TextureSourceError> {
        let resource = manager.get_resource(location)?;
        let image = NativeImage::decode_png(&resource.bytes)?;
        let metadata = resource
            .metadata
            .as_deref()
            .map(serde_json::from_slice::<MetadataRoot>)
            .transpose()
            .map_err(|error| TextureSourceError::AnimationMetadata {
                location: location.clone(),
                message: error.to_string(),
            })?
            .unwrap_or_default();
        let sampling = TextureSampling {
            blur: metadata.texture.blur,
            clamp: metadata.texture.clamp,
        };
        let animation = metadata
            .animation
            .map(|raw| build_animation(location, &image, raw))
            .transpose()?;
        Ok(Self {
            requested_location: location.clone(),
            source_pack: resource.pack_name,
            image,
            sampling,
            animation,
            missing: false,
        })
    }

    /// Internal one-texel equivalent of rendering with texturing disabled. It
    /// allows the shared Vulkan atlas pipeline to preserve MapItemRenderer's
    /// exact per-vertex ARGB colors without borrowing an unrelated game sprite.
    pub fn solid_white(location: ResourceLocation) -> Self {
        Self {
            requested_location: location,
            source_pack: "builtin/solid-white".to_owned(),
            image: NativeImage::from_rgba(1, 1, vec![255, 255, 255, 255])
                .expect("fixed solid-white texture dimensions"),
            sampling: TextureSampling {
                blur: false,
                clamp: true,
            },
            animation: None,
            missing: false,
        }
    }

    /// Static transparent checker produced by `MapItemRenderer.Instance` for
    /// map color index zero. Non-air map pixels are overlaid separately.
    /// `ThreadDownloadImageData` result already decoded and processed by
    /// SkinManager. Dynamic player textures use native nearest sampling and
    /// clamp just like vanilla entity sheets.
    pub fn dynamic(
        location: ResourceLocation,
        image: NativeImage,
        source: impl Into<String>,
    ) -> Self {
        Self {
            requested_location: location,
            source_pack: source.into(),
            image,
            sampling: TextureSampling {
                blur: false,
                clamp: true,
            },
            animation: None,
            missing: false,
        }
    }

    pub fn map_checker(location: ResourceLocation) -> Self {
        let mut rgba = Vec::with_capacity(128 * 128 * 4);
        for index in 0..(128 * 128) {
            let alpha = (((index + index / 128) & 1) * 8 + 16) as u8;
            rgba.extend_from_slice(&[0, 0, 0, alpha]);
        }
        Self {
            requested_location: location,
            source_pack: "builtin/map-checker".to_owned(),
            image: NativeImage::from_rgba(128, 128, rgba)
                .expect("fixed map-checker texture dimensions"),
            sampling: TextureSampling {
                blur: false,
                clamp: true,
            },
            animation: None,
            missing: false,
        }
    }

    /// Equivalent fallback used by `TextureManager.loadTexture` after an I/O
    /// failure. The 16x16 magenta/black pattern and ARGB constants are copied
    /// from `TextureUtil`'s static initializer, then converted to RGBA bytes.
    pub fn missing(location: ResourceLocation) -> Self {
        Self {
            requested_location: location,
            source_pack: "builtin/missingno".to_owned(),
            image: missing_texture_image(),
            sampling: TextureSampling::default(),
            animation: None,
            missing: true,
        }
    }
}

pub fn missing_texture_image() -> NativeImage {
    const MAGENTA_ARGB: u32 = 0xFFF8_00F8;
    const BLACK_ARGB: u32 = 0xFF00_0000;
    let mut rgba = Vec::with_capacity(16 * 16 * 4);
    for y in 0..16 {
        for x in 0..16 {
            let color = if (x < 8) == (y < 8) {
                MAGENTA_ARGB
            } else {
                BLACK_ARGB
            };
            rgba.extend_from_slice(&[
                ((color >> 16) & 0xFF) as u8,
                ((color >> 8) & 0xFF) as u8,
                (color & 0xFF) as u8,
                ((color >> 24) & 0xFF) as u8,
            ]);
        }
    }
    NativeImage::from_rgba(16, 16, rgba).expect("fixed missing texture dimensions")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_texture_matches_texture_util_quadrants() {
        let image = missing_texture_image();
        let pixel = |x: u32, y: u32| {
            let offset = ((y * image.width() + x) * 4) as usize;
            &image.rgba()[offset..offset + 4]
        };
        assert_eq!(pixel(0, 0), [248, 0, 248, 255]);
        assert_eq!(pixel(8, 0), [0, 0, 0, 255]);
        assert_eq!(pixel(0, 8), [0, 0, 0, 255]);
        assert_eq!(pixel(8, 8), [248, 0, 248, 255]);
    }
}
