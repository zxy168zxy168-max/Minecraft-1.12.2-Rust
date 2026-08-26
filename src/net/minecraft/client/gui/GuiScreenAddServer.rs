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
pub enum GuiScreenAddServerAction {
    Confirm(ServerData),
    Cancel,
    CycleResourceMode,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GuiScreenAddServerInteraction {
    pub action: GuiScreenAddServerAction,
    pub sound: Option<GuiSoundCommand>,
}

#[derive(Debug, Clone)]
pub struct GuiScreenAddServer {
    pub GuiScreen: GuiScreen,
    serverData: ServerData,
    serverIPField: GuiTextField,
    serverNameField: GuiTextField,
    title: String,
    enterName: String,
    enterIp: String,
    resourcePackLabel: String,
    addLabel: String,
    cancelLabel: String,
}

impl GuiScreenAddServer {
    pub fn new(serverData: ServerData) -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            serverData,
            serverIPField: GuiTextField::new(1, 0, 0, 200, 20),
            serverNameField: GuiTextField::new(0, 0, 0, 200, 20),
            title: String::new(),
            enterName: String::new(),
            enterIp: String::new(),
            resourcePackLabel: String::new(),
            addLabel: String::new(),
            cancelLabel: String::new(),
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32, locale: &Locale, font: &FontRenderer) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.title = locale.translate_key("addServer.title").to_owned();
        self.enterName = locale.translate_key("addServer.enterName").to_owned();
        self.enterIp = locale.translate_key("addServer.enterIp").to_owned();
        self.resourcePackLabel = locale.translate_key("addServer.resourcePack").to_owned();
        self.addLabel = locale.translate_key("addServer.add").to_owned();
        self.cancelLabel = locale.translate_key("gui.cancel").to_owned();
        self.GuiScreen.buttonList.push(GuiButton::new(
            0,
            width / 2 - 100,
            height / 4 + 114,
            self.addLabel.clone(),
        ));
        self.GuiScreen.buttonList.push(GuiButton::new(
            1,
            width / 2 - 100,
            height / 4 + 138,
            self.cancelLabel.clone(),
        ));
        self.GuiScreen.buttonList.push(GuiButton::new(
            2,
            width / 2 - 100,
            height / 4 + 72,
            self.resourceButtonText(locale),
        ));
        self.serverNameField = GuiTextField::new(0, width / 2 - 100, 66, 200, 20);
        self.serverNameField.setFocused(true);
        self.serverNameField.setText(&self.serverData.serverName);
        self.serverIPField = GuiTextField::new(1, width / 2 - 100, 106, 200, 20);
        self.serverIPField.setMaxStringLength(128);
        self.serverIPField.setValidator(validServerAddressInput);
        self.serverIPField.setText(&self.serverData.serverIP);
        self.updateAddButton();
        self.serverNameField
            .setSelectionPos(self.serverNameField.getCursorPosition(), Some(font));
        self.serverIPField
            .setSelectionPos(self.serverIPField.getCursorPosition(), Some(font));
    }

    pub fn updateScreen(&mut self) {
        self.serverNameField.updateCursorCounter();
        self.serverIPField.updateCursorCounter();
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
            17,
            0x00FF_FFFF,
        );
        self.GuiScreen.Gui.drawString(
            font,
            drawList,
            &self.enterName,
            self.GuiScreen.width / 2 - 100,
            53,
            10_526_880,
        );
        self.GuiScreen.Gui.drawString(
            font,
            drawList,
            &self.enterIp,
            self.GuiScreen.width / 2 - 100,
            94,
            10_526_880,
        );
        self.serverNameField.drawTextBox(drawList, font);
        self.serverIPField.drawTextBox(drawList, font);
        self.GuiScreen
            .drawScreen(drawList, font, mouseX, mouseY, partialTicks);
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
        font: &FontRenderer,
        locale: &Locale,
    ) -> Option<GuiScreenAddServerInteraction> {
        self.serverIPField
            .mouseClicked(mouseX, mouseY, mouseButton, font);
        self.serverNameField
            .mouseClicked(mouseX, mouseY, mouseButton, font);
        if mouseButton != 0 {
            return None;
        }
        let buttonIndex = self
            .GuiScreen
            .buttonList
            .iter()
            .position(|button| button.mousePressed(mouseX, mouseY))?;
        let sound = self.GuiScreen.buttonList[buttonIndex].playPressSound();
        let action = match self.GuiScreen.buttonList[buttonIndex].id {
            2 => {
                self.serverData
                    .setResourceMode(self.serverData.getResourceMode().next());
                let displayString = self.resourceButtonText(locale);
                self.GuiScreen.buttonList[buttonIndex].displayString = displayString;
                GuiScreenAddServerAction::CycleResourceMode
            }
            1 => GuiScreenAddServerAction::Cancel,
            0 => {
                self.serverData.serverName = self.serverNameField.getText();
                self.serverData.serverIP = self.serverIPField.getText();
                GuiScreenAddServerAction::Confirm(self.serverData.clone())
            }
            _ => return None,
        };
        Some(GuiScreenAddServerInteraction {
            action,
            sound: Some(sound),
        })
    }

    pub fn typedText(&mut self, text: &str, font: &FontRenderer) -> bool {
        let changed = if self.serverNameField.isFocused() {
            self.serverNameField.writeText(text, Some(font))
        } else if self.serverIPField.isFocused() {
            self.serverIPField.writeText(text, Some(font))
        } else {
            false
        };
        if changed {
            self.updateAddButton();
        }
        changed
    }

    pub fn keyPressed(
        &mut self,
        key: GuiTextFieldKey,
        modifiers: GuiTextFieldModifiers,
        font: &FontRenderer,
    ) -> bool {
        let changed = if self.serverNameField.isFocused() {
            self.serverNameField.keyPressed(key, modifiers, font)
        } else if self.serverIPField.isFocused() {
            self.serverIPField.keyPressed(key, modifiers, font)
        } else {
            false
        };
        if changed {
            self.updateAddButton();
        }
        changed
    }

    pub fn selectAll(&mut self, font: &FontRenderer) -> bool {
        if self.serverNameField.isFocused() {
            self.serverNameField.selectAll(font);
            true
        } else if self.serverIPField.isFocused() {
            self.serverIPField.selectAll(font);
            true
        } else {
            false
        }
    }

    pub fn tabPressed(&mut self) {
        let nameFocused = self.serverNameField.isFocused();
        self.serverNameField.setFocused(!nameFocused);
        self.serverIPField.setFocused(nameFocused);
    }

    pub fn enterPressed(&mut self) -> Option<GuiScreenAddServerAction> {
        if !self.canConfirm() {
            return None;
        }
        self.serverData.serverName = self.serverNameField.getText();
        self.serverData.serverIP = self.serverIPField.getText();
        Some(GuiScreenAddServerAction::Confirm(self.serverData.clone()))
    }

    fn updateAddButton(&mut self) {
        let enabled = self.canConfirm();
        if let Some(button) = self.GuiScreen.buttonList.get_mut(0) {
            button.enabled = enabled;
        }
    }
    fn canConfirm(&self) -> bool {
        !self.serverIPField.getText().is_empty() && !self.serverNameField.getText().is_empty()
    }
    fn resourceButtonText(&self, locale: &Locale) -> String {
        format!(
            "{}: {}",
            self.resourcePackLabel,
            locale.translate_key(self.serverData.getResourceMode().translationKey())
        )
    }
}

fn validServerAddressInput(value: &str) -> bool {
    if value.is_empty() {
        return true;
    }
    let host = if let Some(rest) = value.strip_prefix('[') {
        rest.split(']').next().unwrap_or(rest)
    } else {
        value.split(':').next().unwrap_or(value)
    };
    idna::domain_to_ascii(host).is_ok()
}
