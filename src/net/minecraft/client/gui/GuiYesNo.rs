use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, PartialEq)]
pub struct GuiYesNoInteraction {
    pub result: bool,
    pub id: i32,
    pub sound: GuiSoundCommand,
}

/// MCP 1.12.2 `GuiYesNo` without the Java callback interface. The caller
/// receives `GuiYesNoInteraction` and performs the matching screen action.
#[derive(Debug, Clone)]
pub struct GuiYesNo {
    pub GuiScreen: GuiScreen,
    messageLine1: String,
    messageLine2: String,
    listLines: Vec<String>,
    confirmButtonText: String,
    cancelButtonText: String,
    parentButtonClickedId: i32,
    ticksUntilEnable: i32,
}

impl GuiYesNo {
    pub fn new(
        messageLine1: String,
        messageLine2: String,
        confirmButtonText: String,
        cancelButtonText: String,
        parentButtonClickedId: i32,
    ) -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            messageLine1,
            messageLine2,
            listLines: Vec::new(),
            confirmButtonText,
            cancelButtonText,
            parentButtonClickedId,
            ticksUntilEnable: 0,
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32, font: &FontRenderer) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            0,
            width / 2 - 155,
            height / 6 + 96,
            150,
            20,
            self.confirmButtonText.clone(),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            1,
            width / 2 + 5,
            height / 6 + 96,
            150,
            20,
            self.cancelButtonText.clone(),
        ));
        self.listLines.clear();
        self.listLines
            .extend(font.list_formatted_string_to_width(&self.messageLine2, width - 50));
        if self.ticksUntilEnable > 0 {
            for button in &mut self.GuiScreen.buttonList {
                button.enabled = false;
            }
        }
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
        self.drawContents(drawList, font, mouseX, mouseY, partialTicks);
    }

    /// Same screen while a world is loaded. MCP `GuiScreen#drawDefaultBackground`
    /// selects the translucent in-world gradient instead of the dirt texture.
    pub fn drawScreenInWorld(
        &mut self,
        drawList: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.GuiScreen.drawDefaultBackgroundInWorld(drawList);
        self.drawContents(drawList, font, mouseX, mouseY, partialTicks);
    }

    fn drawContents(
        &mut self,
        drawList: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            &self.messageLine1,
            self.GuiScreen.width / 2,
            70,
            0x00FF_FFFF,
        );
        let mut y = 90;
        for line in &self.listLines {
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

    pub fn setButtonDelay(&mut self, ticks: i32) {
        self.ticksUntilEnable = ticks;
        for button in &mut self.GuiScreen.buttonList {
            button.enabled = false;
        }
    }

    pub fn updateScreen(&mut self) {
        self.ticksUntilEnable -= 1;
        if self.ticksUntilEnable == 0 {
            for button in &mut self.GuiScreen.buttonList {
                button.enabled = true;
            }
        }
    }

    pub fn mouseClicked(
        &self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
    ) -> Option<GuiYesNoInteraction> {
        if mouseButton != 0 {
            return None;
        }
        let button = self
            .GuiScreen
            .buttonList
            .iter()
            .find(|button| button.mousePressed(mouseX, mouseY))?;
        Some(GuiYesNoInteraction {
            result: button.id == 0,
            id: self.parentButtonClickedId,
            sound: button.playPressSound(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_delay_matches_mcp_countdown() {
        let mut screen = GuiYesNo::new(
            "Question".to_owned(),
            String::new(),
            "Yes".to_owned(),
            "No".to_owned(),
            0,
        );
        screen
            .GuiScreen
            .buttonList
            .push(GuiButton::new(0, 0, 0, "Yes"));
        screen
            .GuiScreen
            .buttonList
            .push(GuiButton::new(1, 0, 24, "No"));
        screen.setButtonDelay(20);
        for _ in 0..19 {
            screen.updateScreen();
        }
        assert!(screen
            .GuiScreen
            .buttonList
            .iter()
            .all(|button| !button.enabled));
        screen.updateScreen();
        assert!(screen
            .GuiScreen
            .buttonList
            .iter()
            .all(|button| button.enabled));
    }
}
