use std::collections::HashMap;
use std::sync::Arc;

use crate::net::minecraft::client::renderer::block::model::ModelBlock::{
    BlockPart, BlockPartFace, ModelBlock,
};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::vulkan::TextureSource::TextureSource;

pub struct ItemModelGenerator;

impl ItemModelGenerator {
    pub const LAYERS: [&'static str; 5] = ["layer0", "layer1", "layer2", "layer3", "layer4"];

    /// MCP 1.12.2 `ItemModelGenerator.makeItemModel`. The caller supplies the
    /// already-loaded sprites in layer order so resource-pack selection remains
    /// the responsibility of TextureMap/ResourceManager rather than this class.
    pub fn makeItemModel(
        source: &ModelBlock,
        layers: &[(String, ResourceLocation, Arc<TextureSource>)],
    ) -> Option<ModelBlock> {
        let mut elements = Vec::new();
        let mut textures = HashMap::new();
        for (tintIndex, (layerName, spriteName, texture)) in layers.iter().enumerate() {
            // ModelBlock stores logical atlas-sprite names (for example
            // `minecraft:items/apple`), not the backing PNG resource path.
            textures.insert(layerName.clone(), spriteName.to_string());
            elements.extend(Self::getBlockParts(tintIndex as i32, layerName, texture));
        }
        if elements.is_empty() {
            return None;
        }
        if let Some((_, firstName, _)) = layers.first() {
            let particle = source
                .resolveTextureName("#particle")
                .filter(|location| location.getPath() != "missingno")
                .unwrap_or_else(|| firstName.clone());
            textures.insert("particle".to_owned(), particle.to_string());
        }
        Some(ModelBlock {
            textures,
            elements,
            ambientOcclusion: false,
            gui3d: false,
            transforms: source.transforms.clone(),
            generated: false,
            builtInRenderer: false,
            namespace: source.namespace.clone(),
        })
    }

    fn getBlockParts(tintIndex: i32, layerName: &str, texture: &TextureSource) -> Vec<BlockPart> {
        let mut frontBack = HashMap::new();
        frontBack.insert(
            "south".to_owned(),
            BlockPartFace {
                texture: format!("#{layerName}"),
                tintindex: Some(tintIndex),
                cullface: None,
                uv: Some([0.0, 0.0, 16.0, 16.0]),
                rotation: 0,
            },
        );
        frontBack.insert(
            "north".to_owned(),
            BlockPartFace {
                texture: format!("#{layerName}"),
                tintindex: Some(tintIndex),
                cullface: None,
                uv: Some([16.0, 0.0, 0.0, 16.0]),
                rotation: 0,
            },
        );
        let mut parts = vec![BlockPart {
            from: [0.0, 0.0, 7.5],
            to: [16.0, 16.0, 8.5],
            rotation: None,
            shade: true,
            faces: frontBack,
        }];
        parts.extend(Self::edgeParts(texture, layerName, tintIndex));
        parts
    }

    fn edgeParts(texture: &TextureSource, layerName: &str, tintIndex: i32) -> Vec<BlockPart> {
        let image = &texture.image;
        // TextureAtlasSprite's icon dimensions are the dimensions of one
        // animation frame. Vanilla item animations are vertical strips, so
        // the PNG width is the frame width and the per-frame height is width.
        let frameWidth = image.width().max(1);
        let frameHeight = image.width().min(image.height().max(1));
        let spans = collect_spans(texture, frameWidth, frameHeight);
        let mut parts = Vec::with_capacity(spans.len());

        for span in spans {
            // This is the same f2..f16 calculation performed by MCP
            // ItemModelGenerator#getBlockParts(TextureAtlasSprite,...).
            let minimum = span.min as f32;
            let maximum = span.max as f32;
            let anchor = span.anchor as f32;
            let (mut x0, mut y0, mut x1, mut y1, mut u0, mut v0, mut u1, mut v1, uScale, vScale) =
                match span.facing {
                    SpanFacing::Up => (
                        minimum,
                        anchor,
                        maximum + 1.0,
                        anchor,
                        minimum,
                        anchor,
                        maximum + 1.0,
                        anchor,
                        16.0 / frameWidth as f32,
                        16.0 / (frameHeight.saturating_sub(1).max(1)) as f32,
                    ),
                    SpanFacing::Down => (
                        minimum,
                        anchor + 1.0,
                        maximum + 1.0,
                        anchor + 1.0,
                        minimum,
                        anchor,
                        maximum + 1.0,
                        anchor,
                        16.0 / frameWidth as f32,
                        16.0 / (frameHeight.saturating_sub(1).max(1)) as f32,
                    ),
                    SpanFacing::Left => (
                        anchor,
                        minimum,
                        anchor,
                        maximum + 1.0,
                        anchor,
                        maximum + 1.0,
                        anchor,
                        minimum,
                        16.0 / (frameWidth.saturating_sub(1).max(1)) as f32,
                        16.0 / frameHeight as f32,
                    ),
                    SpanFacing::Right => (
                        anchor + 1.0,
                        minimum,
                        anchor + 1.0,
                        maximum + 1.0,
                        anchor,
                        maximum + 1.0,
                        anchor,
                        minimum,
                        16.0 / (frameWidth.saturating_sub(1).max(1)) as f32,
                        16.0 / frameHeight as f32,
                    ),
                };

            let geometryU = 16.0 / frameWidth as f32;
            let geometryV = 16.0 / frameHeight as f32;
            x0 *= geometryU;
            x1 *= geometryU;
            y0 *= geometryV;
            y1 *= geometryV;
            y0 = 16.0 - y0;
            y1 = 16.0 - y1;
            u0 *= uScale;
            u1 *= uScale;
            v0 *= vScale;
            v1 *= vScale;

            let (from, to) = match span.facing {
                SpanFacing::Up => ([x0, y0, 7.5], [x1, y0, 8.5]),
                SpanFacing::Down => ([x0, y1, 7.5], [x1, y1, 8.5]),
                SpanFacing::Left => ([x0, y0, 7.5], [x0, y1, 8.5]),
                SpanFacing::Right => ([x1, y0, 7.5], [x1, y1, 8.5]),
            };
            let mut faces = HashMap::new();
            faces.insert(
                span.facing.face_name().to_owned(),
                BlockPartFace {
                    texture: format!("#{layerName}"),
                    tintindex: Some(tintIndex),
                    cullface: None,
                    uv: Some([u0, v0, u1, v1]),
                    rotation: 0,
                },
            );
            parts.push(BlockPart {
                from,
                to,
                rotation: None,
                shade: true,
                faces,
            });
        }
        parts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpanFacing {
    Up,
    Down,
    Left,
    Right,
}

impl SpanFacing {
    const VALUES: [Self; 4] = [Self::Up, Self::Down, Self::Left, Self::Right];

    fn offset(self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }

    fn horizontal(self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }

    fn face_name(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            // MCP SpanFacing.LEFT maps to EnumFacing.EAST and RIGHT to WEST.
            Self::Left => "east",
            Self::Right => "west",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Span {
    facing: SpanFacing,
    min: i32,
    max: i32,
    anchor: i32,
}

fn collect_spans(texture: &TextureSource, width: u32, height: u32) -> Vec<Span> {
    let image = &texture.image;
    let frames = (image.height() / height.max(1)).max(1);
    let mut spans = Vec::new();
    for frame in 0..frames {
        let frameY = frame * height;
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let opaque = !transparent_in_frame(image, x, y, width, height, frameY);
                for facing in SpanFacing::VALUES {
                    let (dx, dy) = facing.offset();
                    if opaque && transparent_in_frame(image, x + dx, y + dy, width, height, frameY)
                    {
                        create_or_expand_span(&mut spans, facing, x, y);
                    }
                }
            }
        }
    }
    spans
}

fn transparent_in_frame(
    image: &crate::vulkan::NativeImage::NativeImage,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    frameY: u32,
) -> bool {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return true;
    }
    image.alpha(x as u32, frameY + y as u32) == 0
}

fn create_or_expand_span(spans: &mut Vec<Span>, facing: SpanFacing, x: i32, y: i32) {
    let anchor = if facing.horizontal() { y } else { x };
    let value = if facing.horizontal() { x } else { y };
    if let Some(span) = spans
        .iter_mut()
        .find(|span| span.facing == facing && span.anchor == anchor)
    {
        span.min = span.min.min(value);
        span.max = span.max.max(value);
    } else {
        spans.push(Span {
            facing,
            min: value,
            max: value,
            anchor,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vulkan::NativeImage::NativeImage;
    use crate::vulkan::TextureSource::{TextureSampling, TextureSource};

    #[test]
    fn opaque_single_pixel_creates_four_edge_spans() {
        let mut rgba = vec![0_u8; 16 * 16 * 4];
        rgba[((8 * 16 + 8) * 4 + 3) as usize] = 255;
        let source = TextureSource {
            requested_location: ResourceLocation::new("minecraft", "textures/items/test.png"),
            source_pack: "test".to_owned(),
            image: NativeImage::from_rgba(16, 16, rgba).unwrap(),
            sampling: TextureSampling::default(),
            animation: None,
            missing: false,
        };
        let spans = collect_spans(&source, 16, 16);
        assert_eq!(spans.len(), 4);
    }
}
