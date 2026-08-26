use once_cell::sync::Lazy;

use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::vulkan::GuiDrawList::GuiDrawList;

pub static OPTIONS_BACKGROUND: Lazy<ResourceLocation> =
    Lazy::new(|| ResourceLocation::parse("textures/gui/options_background.png"));
pub static STAT_ICONS: Lazy<ResourceLocation> =
    Lazy::new(|| ResourceLocation::parse("textures/gui/container/stats_icons.png"));
pub static ICONS: Lazy<ResourceLocation> =
    Lazy::new(|| ResourceLocation::parse("textures/gui/icons.png"));

/// MCP `Gui` drawing surface. Vulkan-specific command storage remains behind
/// `GuiDrawList`; the public method names, coordinate rules, and z-level state
/// mirror the 1.12.2 class.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gui {
    pub zLevel: f32,
}

impl Default for Gui {
    fn default() -> Self {
        Self { zLevel: 0.0 }
    }
}

impl Gui {
    pub fn drawHorizontalLine(
        &self,
        drawList: &mut GuiDrawList,
        startX: i32,
        endX: i32,
        y: i32,
        color: i32,
    ) {
        drawList.draw_horizontal_line(startX, endX, y, color);
    }

    pub fn drawVerticalLine(
        &self,
        drawList: &mut GuiDrawList,
        x: i32,
        startY: i32,
        endY: i32,
        color: i32,
    ) {
        drawList.draw_vertical_line(x, startY, endY, color);
    }

    pub fn drawRect(
        drawList: &mut GuiDrawList,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        color: i32,
    ) {
        drawList.draw_rect(left, top, right, bottom, color);
    }

    pub fn drawGradientRect(
        &self,
        drawList: &mut GuiDrawList,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        startColor: i32,
        endColor: i32,
    ) {
        drawList.set_z_level(self.zLevel);
        drawList.draw_gradient_rect(left, top, right, bottom, startColor, endColor);
    }

    pub fn drawCenteredString(
        &self,
        fontRendererIn: &mut FontRenderer,
        drawList: &mut GuiDrawList,
        text: &str,
        x: i32,
        y: i32,
        color: i32,
    ) {
        fontRendererIn.draw_string_with_shadow(
            drawList,
            text,
            (x - fontRendererIn.get_string_width(text) / 2) as f32,
            y as f32,
            color,
        );
    }

    pub fn drawString(
        &self,
        fontRendererIn: &mut FontRenderer,
        drawList: &mut GuiDrawList,
        text: &str,
        x: i32,
        y: i32,
        color: i32,
    ) {
        fontRendererIn.draw_string_with_shadow(drawList, text, x as f32, y as f32, color);
    }

    pub fn drawTexturedModalRect(
        &self,
        drawList: &mut GuiDrawList,
        texture: ResourceLocation,
        x: i32,
        y: i32,
        textureX: i32,
        textureY: i32,
        width: i32,
        height: i32,
    ) {
        drawList.set_z_level(self.zLevel);
        drawList.draw_textured_modal_rect(texture, x, y, textureX, textureY, width, height);
    }

    pub fn drawModalRectWithCustomSizedTexture(
        drawList: &mut GuiDrawList,
        texture: ResourceLocation,
        x: f32,
        y: f32,
        u: f32,
        v: f32,
        width: f32,
        height: f32,
        textureWidth: f32,
        textureHeight: f32,
    ) {
        drawList.draw_modal_rect_with_custom_sized_texture(
            texture,
            x,
            y,
            u,
            v,
            width,
            height,
            textureWidth,
            textureHeight,
        );
    }
}
