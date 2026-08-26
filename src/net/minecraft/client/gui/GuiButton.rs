use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::vulkan::GuiDrawList::GuiDrawList;

pub const BUTTON_TEXTURES_PATH: &str = "textures/gui/widgets.png";
pub const BUTTON_CLICK_SOUND_PATH: &str = "ui.button.click";

/// Renderer/audio-independent equivalent of
/// `PositionedSoundRecord.getMasterRecord(SoundEvents.UI_BUTTON_CLICK, 1.0F)`.
#[derive(Debug, Clone, PartialEq)]
pub struct GuiSoundCommand {
    pub event: ResourceLocation,
    pub pitch: f32,
}

/// Semantic port of `net.minecraft.client.gui.GuiButton`.
///
/// Rendering is recorded as an ordered GUI draw list. The Vulkan backend must
/// preserve this order and the same alpha-blend semantics; it must not replace
/// the two 100-pixel texture slices with a redesigned nine-patch widget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiButton {
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub x: i32,
    pub y: i32,
    pub displayString: String,
    pub id: i32,
    pub enabled: bool,
    pub visible: bool,
    pub(crate) hovered: bool,
}

impl GuiButton {
    pub fn new(button_id: i32, x: i32, y: i32, button_text: impl Into<String>) -> Self {
        Self::newWithSize(button_id, x, y, 200, 20, button_text)
    }

    pub fn newWithSize(
        button_id: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        button_text: impl Into<String>,
    ) -> Self {
        Self {
            width,
            height,
            x,
            y,
            displayString: button_text.into(),
            id: button_id,
            enabled: true,
            visible: true,
            hovered: false,
        }
    }

    /// Returns the MCP hover-state row: disabled=0, normal=1, hovered=2.
    pub const fn getHoverState(&self, mouse_over: bool) -> i32 {
        if !self.enabled {
            0
        } else if mouse_over {
            2
        } else {
            1
        }
    }

    pub fn drawButton(
        &mut self,
        draw_list: &mut GuiDrawList,
        font_renderer: &mut FontRenderer,
        mouse_x: i32,
        mouse_y: i32,
        _partial_ticks: f32,
    ) {
        if !self.visible {
            return;
        }

        self.hovered = self.contains(mouse_x, mouse_y);
        let getHoverState = self.getHoverState(self.hovered);
        let texture = ResourceLocation::parse(BUTTON_TEXTURES_PATH);

        // MCP draws the button as two halves from widgets.png so widths up to
        // 200 pixels preserve the original edge caps and center stretch.
        draw_list.draw_textured_modal_rect(
            texture.clone(),
            self.x,
            self.y,
            0,
            46 + getHoverState * 20,
            self.width / 2,
            self.height,
        );
        draw_list.draw_textured_modal_rect(
            texture,
            self.x + self.width / 2,
            self.y,
            200 - self.width / 2,
            46 + getHoverState * 20,
            self.width / 2,
            self.height,
        );

        self.mouseDragged(mouse_x, mouse_y);
        let text_color = if !self.enabled {
            10_526_880
        } else if self.hovered {
            16_777_120
        } else {
            14_737_632
        };
        font_renderer.draw_centered_string_with_shadow(
            draw_list,
            &self.displayString,
            self.x + self.width / 2,
            self.y + (self.height - 8) / 2,
            text_color,
        );
    }

    pub fn mousePressed(&self, mouse_x: i32, mouse_y: i32) -> bool {
        self.enabled && self.visible && self.contains(mouse_x, mouse_y)
    }

    pub fn mouseReleased(&mut self, _mouse_x: i32, _mouse_y: i32) {}

    pub fn playPressSound(&self) -> GuiSoundCommand {
        GuiSoundCommand {
            event: ResourceLocation::parse(BUTTON_CLICK_SOUND_PATH),
            pitch: 1.0,
        }
    }

    pub fn mouseDragged(&mut self, _mouse_x: i32, _mouse_y: i32) {}

    pub const fn isMouseOver(&self) -> bool {
        self.hovered
    }
    pub const fn getButtonWidth(&self) -> i32 {
        self.width
    }
    pub const fn getButtonHeight(&self) -> i32 {
        self.height
    }
    pub fn setWidth(&mut self, width: i32) {
        self.width = width;
    }

    pub(crate) fn contains(&self, mouse_x: i32, mouse_y: i32) -> bool {
        mouse_x >= self.x
            && mouse_y >= self.y
            && mouse_x < self.x + self.width
            && mouse_y < self.y + self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_button_hit_test_excludes_right_and_bottom_edges() {
        let button = GuiButton::newWithSize(1, 10, 20, 100, 20, "Test");
        assert!(button.mousePressed(10, 20));
        assert!(button.mousePressed(109, 39));
        assert!(!button.mousePressed(110, 39));
        assert!(!button.mousePressed(109, 40));
    }

    #[test]
    fn vanilla_getHoverState_values_are_preserved() {
        let mut button = GuiButton::new(1, 0, 0, "Test");
        assert_eq!(button.getHoverState(false), 1);
        assert_eq!(button.getHoverState(true), 2);
        button.enabled = false;
        assert_eq!(button.getHoverState(true), 0);
    }

    #[test]
    fn playPressSound_matches_sound_events_registration() {
        let button = GuiButton::new(1, 0, 0, "Test");
        let sound = button.playPressSound();
        assert_eq!(sound.event.to_string(), "minecraft:ui.button.click");
        assert_eq!(sound.pitch, 1.0);
    }
}
