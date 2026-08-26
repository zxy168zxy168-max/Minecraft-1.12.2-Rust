use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiDisconnectedAction {
    ToMenu,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GuiDisconnectedInteraction {
    pub action: GuiDisconnectedAction,
    pub sound: GuiSoundCommand,
}
#[derive(Debug, Clone)]
pub struct GuiDisconnected {
    pub GuiScreen: GuiScreen,
    reason: String,
    message: String,
    multilineMessage: Vec<String>,
    textHeight: i32,
    toMenu: String,
}
impl GuiDisconnected {
    pub fn new(reason: String, message: String, toMenu: String) -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            reason,
            message,
            multilineMessage: Vec::new(),
            textHeight: 0,
            toMenu,
        }
    }
    pub fn initGui(&mut self, width: i32, height: i32, font: &FontRenderer) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.multilineMessage = font.list_formatted_string_to_width(&self.message, width - 50);
        self.textHeight = self.multilineMessage.len() as i32 * font.font_height;
        let y = (height / 2 + self.textHeight / 2 + font.font_height).min(height - 30);
        self.GuiScreen
            .buttonList
            .push(GuiButton::new(0, width / 2 - 100, y, self.toMenu.clone()));
    }
    pub fn drawScreen(
        &mut self,
        drawList: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.GuiScreen.drawDefaultBackground(drawList);
        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            &self.reason,
            self.GuiScreen.width / 2,
            self.GuiScreen.height / 2 - self.textHeight / 2 - font.font_height * 2,
            11_184_810,
        );
        let mut y = self.GuiScreen.height / 2 - self.textHeight / 2;
        for line in &self.multilineMessage {
            self.GuiScreen.Gui.drawCenteredString(
                font,
                drawList,
                line,
                self.GuiScreen.width / 2,
                y,
                0x00FF_FFFF,
            );
            y += font.font_height;
        }
        self.GuiScreen
            .drawScreen(drawList, font, mouseX, mouseY, partialTicks);
    }
    pub fn mouseClicked(
        &self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
    ) -> Option<GuiDisconnectedInteraction> {
        if mouseButton != 0 {
            return None;
        }
        let button = self
            .GuiScreen
            .buttonList
            .iter()
            .find(|button| button.mousePressed(mouseX, mouseY))?;
        Some(GuiDisconnectedInteraction {
            action: GuiDisconnectedAction::ToMenu,
            sound: button.playPressSound(),
        })
    }
}
