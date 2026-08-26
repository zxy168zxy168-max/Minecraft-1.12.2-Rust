use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::gui::GuiTextField::{
    GuiTextField, GuiTextFieldKey, GuiTextFieldModifiers,
};
use crate::net::minecraft::client::multiplayer::ServerData::ServerData;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiScreenServerListAction {
    Confirm(ServerData),
    Cancel,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GuiScreenServerListInteraction {
    pub action: GuiScreenServerListAction,
    pub sound: GuiSoundCommand,
}

#[derive(Debug, Clone)]
pub struct GuiScreenServerList {
    pub GuiScreen: GuiScreen,
    serverData: ServerData,
    ipEdit: GuiTextField,
    title: String,
    enterIp: String,
}

impl GuiScreenServerList {
    pub fn new(serverData: ServerData) -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            serverData,
            ipEdit: GuiTextField::new(2, 0, 0, 200, 20),
            title: String::new(),
            enterIp: String::new(),
        }
    }
    pub fn initGui(
        &mut self,
        width: i32,
        height: i32,
        locale: &Locale,
        font: &FontRenderer,
        lastServer: &str,
    ) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.title = locale.translate_key("selectServer.direct").to_owned();
        self.enterIp = locale.translate_key("addServer.enterIp").to_owned();
        self.GuiScreen.buttonList.push(GuiButton::new(
            0,
            width / 2 - 100,
            height / 4 + 108,
            locale.translate_key("selectServer.select"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::new(
            1,
            width / 2 - 100,
            height / 4 + 132,
            locale.translate_key("gui.cancel"),
        ));
        self.ipEdit = GuiTextField::new(2, width / 2 - 100, 116, 200, 20);
        self.ipEdit.setMaxStringLength(128);
        self.ipEdit.setFocused(true);
        self.ipEdit.setText(lastServer);
        self.ipEdit
            .setSelectionPos(self.ipEdit.getCursorPosition(), Some(font));
        self.updateSelectButton();
    }
    pub fn updateScreen(&mut self) {
        self.ipEdit.updateCursorCounter();
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
            &self.title,
            self.GuiScreen.width / 2,
            20,
            0x00FF_FFFF,
        );
        self.GuiScreen.Gui.drawString(
            font,
            drawList,
            &self.enterIp,
            self.GuiScreen.width / 2 - 100,
            100,
            10_526_880,
        );
        self.ipEdit.drawTextBox(drawList, font);
        self.GuiScreen
            .drawScreen(drawList, font, mouseX, mouseY, partialTicks);
    }
    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
        font: &FontRenderer,
    ) -> Option<GuiScreenServerListInteraction> {
        self.ipEdit.mouseClicked(mouseX, mouseY, mouseButton, font);
        if mouseButton != 0 {
            return None;
        }
        let button = self
            .GuiScreen
            .buttonList
            .iter()
            .find(|button| button.mousePressed(mouseX, mouseY))?;
        let action = match button.id {
            1 => GuiScreenServerListAction::Cancel,
            0 => {
                let mut server = self.serverData.clone();
                server.serverIP = self.ipEdit.getText();
                GuiScreenServerListAction::Confirm(server)
            }
            _ => return None,
        };
        Some(GuiScreenServerListInteraction {
            action,
            sound: button.playPressSound(),
        })
    }
    pub fn typedText(&mut self, text: &str, font: &FontRenderer) -> bool {
        let changed = self.ipEdit.writeText(text, Some(font));
        if changed {
            self.updateSelectButton();
        }
        changed
    }
    pub fn keyPressed(
        &mut self,
        key: GuiTextFieldKey,
        modifiers: GuiTextFieldModifiers,
        font: &FontRenderer,
    ) -> bool {
        let changed = self.ipEdit.keyPressed(key, modifiers, font);
        if changed {
            self.updateSelectButton();
        }
        changed
    }
    pub fn selectAll(&mut self, font: &FontRenderer) {
        self.ipEdit.selectAll(font);
    }
    pub fn enterPressed(&self) -> Option<GuiScreenServerListAction> {
        if self.canConfirm() {
            let mut server = self.serverData.clone();
            server.serverIP = self.ipEdit.getText();
            Some(GuiScreenServerListAction::Confirm(server))
        } else {
            None
        }
    }
    pub fn getAddress(&self) -> String {
        self.ipEdit.getText()
    }
    fn canConfirm(&self) -> bool {
        !self.ipEdit.getText().is_empty()
    }
    fn updateSelectButton(&mut self) {
        let enabled = self.canConfirm();
        if let Some(button) = self.GuiScreen.buttonList.get_mut(0) {
            button.enabled = enabled;
        }
    }
}
