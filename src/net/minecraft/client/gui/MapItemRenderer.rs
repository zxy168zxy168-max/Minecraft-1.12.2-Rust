use crate::net::minecraft::block::material::MapColor::MapColor;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::world::storage::MapData::MapData;
use crate::net::minecraft::world::storage::MapDecoration::MapDecoration;

/// Rendering constants and data transforms from MCP 1.12.2
/// `MapItemRenderer`. Vulkan submits equivalent map pixel/icon geometry rather
/// than creating one OpenGL DynamicTexture per map.
pub struct MapItemRenderer;

impl MapItemRenderer {
    pub const MAP_SIZE: f32 = 128.0;
    pub const MAP_SCALE: f32 = 1.0 / Self::MAP_SIZE;
    pub const MAP_PLANE_Z: f32 = -0.01;
    pub const ICON_SCALE: f32 = 4.0;
    pub const ICON_TRANSLATE_X: f32 = -0.125;
    pub const ICON_TRANSLATE_Y: f32 = 0.125;

    pub fn iconsTexture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/map/map_icons.png")
    }

    /// Exact CPU-side contents produced by `Instance#updateMapTexture` for one
    /// of the 16,384 map pixels.
    pub fn pixelColor(mapData: &MapData, index: usize) -> [f32; 4] {
        let color = mapData.colors.get(index).copied().unwrap_or(0);
        let palette = color / 4;
        if palette == 0 {
            let alpha = (((index + index / MapData::WIDTH) & 1) * 8 + 16) as f32 / 255.0;
            [0.0, 0.0, 0.0, alpha]
        } else {
            MapColor::argbToRgba(MapColor::getMapColor(palette as usize, color & 3))
        }
    }

    pub fn renderedDecorations<'a>(
        mapData: &'a MapData,
        noOverlayRendering: bool,
    ) -> impl Iterator<Item = &'a MapDecoration> + 'a {
        mapData
            .mapDecorations
            .iter()
            .filter(move |decoration| !noOverlayRendering || decoration.isRenderedOnFrame())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::world::storage::MapDecoration::{MapDecoration, MapDecorationType};

    #[test]
    fn empty_pixels_keep_the_vanilla_low_alpha_checker() {
        let map = MapData::new(0);
        assert_eq!(MapItemRenderer::pixelColor(&map, 0)[3], 16.0 / 255.0);
        assert_eq!(MapItemRenderer::pixelColor(&map, 1)[3], 24.0 / 255.0);
    }

    #[test]
    fn item_frame_filter_keeps_only_persistent_markers() {
        let mut map = MapData::new(0);
        map.mapDecorations = vec![
            MapDecoration::new(MapDecorationType::Player, 0, 0, 0),
            MapDecoration::new(MapDecorationType::Frame, 0, 0, 0),
        ];
        assert_eq!(MapItemRenderer::renderedDecorations(&map, true).count(), 1);
        assert_eq!(MapItemRenderer::renderedDecorations(&map, false).count(), 2);
    }
}
