use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

use crate::net::minecraft::client::gui::GuiButton::{
    GuiButton, GuiSoundCommand, BUTTON_TEXTURES_PATH,
};
use crate::vulkan::GuiDrawList::GuiDrawList;

/// Semantic port of `GuiButtonLanguage`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiButtonLanguage {
    button: GuiButton,
}

impl GuiButtonLanguage {
    pub fn new(button_id: i32, x: i32, y: i32) -> Self {
        Self {
            button: GuiButton::newWithSize(button_id, x, y, 20, 20, ""),
        }
    }

    pub fn drawButton(&mut self, draw_list: &mut GuiDrawList, mouse_x: i32, mouse_y: i32) {
        if !self.button.visible {
            return;
        }
        let hovered = self.button.contains(mouse_x, mouse_y);
        self.button.hovered = hovered;
        let texture_y = if hovered { 126 } else { 106 };
        draw_list.draw_textured_modal_rect(
            ResourceLocation::parse(BUTTON_TEXTURES_PATH),
            self.button.x,
            self.button.y,
            0,
            texture_y,
            self.button.width,
            self.button.height,
        );
    }

    pub fn mousePressed(&self, mouse_x: i32, mouse_y: i32) -> bool {
        self.button.mousePressed(mouse_x, mouse_y)
    }

    pub fn playPressSound(&self) -> GuiSoundCommand {
        self.button.playPressSound()
    }

    pub const fn id(&self) -> i32 {
        self.button.id
    }
    pub const fn x(&self) -> i32 {
        self.button.x
    }
    pub const fn y(&self) -> i32 {
        self.button.y
    }
    pub const fn width(&self) -> i32 {
        self.button.width
    }
    pub const fn height(&self) -> i32 {
        self.button.height
    }
    pub const fn visible(&self) -> bool {
        self.button.visible
    }
    pub const fn enabled(&self) -> bool {
        self.button.enabled
    }
}
