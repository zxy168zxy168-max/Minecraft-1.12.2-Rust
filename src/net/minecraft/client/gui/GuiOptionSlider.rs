use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{
    GuiButton, GuiSoundCommand, BUTTON_TEXTURES_PATH,
};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::vulkan::GuiDrawList::GuiDrawList;

/// Renderer-independent port of MCP 1.12.2 `GuiOptionSlider`.
///
/// The option-specific normalization remains in `GuiVideoSettings`, matching
/// `GameSettings.Options.normalizeValue/denormalizeValue`. This class owns only
/// the original slider interaction and widgets.png geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct GuiOptionSlider {
    pub GuiButton: GuiButton,
    pub sliderValue: f32,
    pub dragging: bool,
}

impl GuiOptionSlider {
    pub fn new(
        buttonId: i32,
        x: i32,
        y: i32,
        sliderValue: f32,
        displayString: impl Into<String>,
    ) -> Self {
        Self {
            GuiButton: GuiButton::newWithSize(buttonId, x, y, 150, 20, displayString),
            sliderValue: sliderValue.clamp(0.0, 1.0),
            dragging: false,
        }
    }

    pub fn setDisplayString(&mut self, displayString: impl Into<String>) {
        self.GuiButton.displayString = displayString.into();
    }

    pub fn setSliderValue(&mut self, sliderValue: f32) {
        self.sliderValue = sliderValue.clamp(0.0, 1.0);
    }

    pub fn drawButton(
        &mut self,
        drawList: &mut GuiDrawList,
        fontRendererObj: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        _partialTicks: f32,
    ) {
        if !self.GuiButton.visible {
            return;
        }

        self.GuiButton.hovered = self.GuiButton.contains(mouseX, mouseY);
        let hoverState = 0;
        let texture = ResourceLocation::parse(BUTTON_TEXTURES_PATH);

        // `GuiOptionSlider.getHoverState` always returns 0, therefore the
        // slider background uses row 46 from widgets.png regardless of hover.
        drawList.draw_textured_modal_rect(
            texture.clone(),
            self.GuiButton.x,
            self.GuiButton.y,
            0,
            46 + hoverState * 20,
            self.GuiButton.width / 2,
            self.GuiButton.height,
        );
        drawList.draw_textured_modal_rect(
            texture.clone(),
            self.GuiButton.x + self.GuiButton.width / 2,
            self.GuiButton.y,
            200 - self.GuiButton.width / 2,
            46 + hoverState * 20,
            self.GuiButton.width / 2,
            self.GuiButton.height,
        );

        let handleX =
            self.GuiButton.x + (self.sliderValue * (self.GuiButton.width - 8) as f32) as i32;
        drawList.draw_textured_modal_rect(texture.clone(), handleX, self.GuiButton.y, 0, 66, 4, 20);
        drawList.draw_textured_modal_rect(texture, handleX + 4, self.GuiButton.y, 196, 66, 4, 20);

        let textColor = if self.GuiButton.enabled {
            14_737_632
        } else {
            10_526_880
        };
        fontRendererObj.draw_centered_string_with_shadow(
            drawList,
            &self.GuiButton.displayString,
            self.GuiButton.x + self.GuiButton.width / 2,
            self.GuiButton.y + (self.GuiButton.height - 8) / 2,
            textColor,
        );
    }

    pub fn mousePressed(&mut self, mouseX: i32, mouseY: i32) -> Option<f32> {
        if !self.GuiButton.mousePressed(mouseX, mouseY) {
            return None;
        }
        self.dragging = true;
        Some(self.updateFromMouse(mouseX))
    }

    pub fn mouseDragged(&mut self, mouseX: i32) -> Option<f32> {
        if self.dragging {
            Some(self.updateFromMouse(mouseX))
        } else {
            None
        }
    }

    pub fn mouseReleased(&mut self, _mouseX: i32, _mouseY: i32) {
        self.dragging = false;
    }

    pub fn playPressSound(&self) -> GuiSoundCommand {
        self.GuiButton.playPressSound()
    }

    fn updateFromMouse(&mut self, mouseX: i32) -> f32 {
        self.sliderValue =
            (mouseX - (self.GuiButton.x + 4)) as f32 / (self.GuiButton.width - 8) as f32;
        self.sliderValue = self.sliderValue.clamp(0.0, 1.0);
        self.sliderValue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_uses_vanilla_four_pixel_end_caps() {
        let mut slider = GuiOptionSlider::new(1, 10, 20, 0.0, "Test");
        assert_eq!(slider.mousePressed(14, 20), Some(0.0));
        assert_eq!(slider.mouseDragged(156), Some(1.0));
        slider.mouseReleased(156, 20);
        assert_eq!(slider.mouseDragged(80), None);
    }
}
