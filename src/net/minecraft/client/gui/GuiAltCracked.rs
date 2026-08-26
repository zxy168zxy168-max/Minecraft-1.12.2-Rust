use std::sync::mpsc::{self, Receiver};
use std::thread;

use crate::net::minecraft::client::account::MicrosoftAuth::login_with_credentials;
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::gui::GuiTextField::{
    GuiTextField, GuiTextFieldKey, GuiTextFieldModifiers,
};
use crate::net::minecraft::util::Session::Session;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiAltCrackedAction {
    Cancel,
    Authenticated(Session),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiAltCrackedInteraction {
    pub action: GuiAltCrackedAction,
    pub sound: Option<GuiSoundCommand>,
}

#[derive(Debug)]
enum AltLoginEvent {
    Status(String),
    Success(Session),
    Failure,
}

/// Rust-equivalent port of Exhibition's `GuiAltCracked` and
/// `AltLoginThread`. A blank password creates an offline session. A non-empty
/// password runs the same external-openauth credential flow locally and never
/// writes the password to disk.
#[derive(Debug)]
pub struct GuiAltCracked {
    pub GuiScreen: GuiScreen,
    username: GuiTextField,
    password: GuiTextField,
    status: String,
    receiver: Option<Receiver<AltLoginEvent>>,
    completedSession: Option<Session>,
}

impl GuiAltCracked {
    pub fn new() -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            username: GuiTextField::new(0, 0, 0, 200, 20),
            password: GuiTextField::new(1, 0, 0, 200, 20),
            status: "§7Idle...".to_owned(),
            receiver: None,
            completedSession: None,
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        let y = height / 4 + 24;
        self.GuiScreen
            .buttonList
            .push(GuiButton::new(0, width / 2 - 100, y + 84, "Login"));
        self.GuiScreen
            .buttonList
            .push(GuiButton::new(1, width / 2 - 100, y + 108, "Back"));
        self.username = GuiTextField::new(0, width / 2 - 100, 60, 200, 20);
        self.password = GuiTextField::new(1, width / 2 - 100, 100, 200, 20);
        self.username.setFocused(true);
        self.password.setMaxStringLength(256);
        self.password.setMaskCharacter(Some('*'));
        self.updateButton();
    }

    pub fn updateScreen(&mut self) -> Option<Session> {
        self.username.updateCursorCounter();
        self.password.updateCursorCounter();
        let mut clear = false;
        if let Some(receiver) = &self.receiver {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    AltLoginEvent::Status(status) => self.status = status,
                    AltLoginEvent::Success(session) => {
                        self.status = format!("§aLogged in as {}", session.getUsername());
                        self.completedSession = Some(session);
                        clear = true;
                    }
                    AltLoginEvent::Failure => {
                        self.status = "§cLogin failed!".to_owned();
                        clear = true;
                    }
                }
            }
        }
        if clear {
            self.receiver = None;
            self.updateButton();
        }
        self.completedSession.take()
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
        self.username.drawTextBox(drawList, font);
        self.password.drawTextBox(drawList, font);
        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            "Alt Login",
            self.GuiScreen.width / 2,
            20,
            -1,
        );
        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            &self.status,
            self.GuiScreen.width / 2,
            29,
            -1,
        );
        if self.username.getText().is_empty() {
            self.GuiScreen.Gui.drawString(
                font,
                drawList,
                "Username / E-Mail",
                self.GuiScreen.width / 2 - 96,
                66,
                -7_829_368,
            );
        }
        if self.password.getText().is_empty() {
            self.GuiScreen.Gui.drawString(
                font,
                drawList,
                "Password",
                self.GuiScreen.width / 2 - 96,
                106,
                -7_829_368,
            );
        }
        self.GuiScreen
            .drawScreen(drawList, font, mouseX, mouseY, partialTicks);
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
        font: &FontRenderer,
    ) -> Option<GuiAltCrackedInteraction> {
        self.username
            .mouseClicked(mouseX, mouseY, mouseButton, font);
        self.password
            .mouseClicked(mouseX, mouseY, mouseButton, font);
        if mouseButton != 0 {
            return None;
        }
        let button = self
            .GuiScreen
            .buttonList
            .iter()
            .find(|button| button.mousePressed(mouseX, mouseY))?
            .clone();
        let action = match button.id {
            1 => GuiAltCrackedAction::Cancel,
            0 => self.login(),
            _ => GuiAltCrackedAction::None,
        };
        Some(GuiAltCrackedInteraction {
            action,
            sound: Some(button.playPressSound()),
        })
    }

    pub fn typedText(&mut self, text: &str, font: &FontRenderer) -> bool {
        let changed = if self.username.isFocused() {
            self.username.writeText(text, Some(font))
        } else if self.password.isFocused() {
            self.password.writeText(text, Some(font))
        } else {
            false
        };
        if changed {
            self.updateButton();
        }
        changed
    }

    pub fn keyPressed(
        &mut self,
        key: GuiTextFieldKey,
        modifiers: GuiTextFieldModifiers,
        font: &FontRenderer,
    ) -> bool {
        let changed = if self.username.isFocused() {
            self.username.keyPressed(key, modifiers, font)
        } else if self.password.isFocused() {
            self.password.keyPressed(key, modifiers, font)
        } else {
            false
        };
        if changed {
            self.updateButton();
        }
        changed
    }

    pub fn selectAll(&mut self, font: &FontRenderer) {
        if self.username.isFocused() {
            self.username.selectAll(font);
        } else if self.password.isFocused() {
            self.password.selectAll(font);
        }
    }

    pub fn tabPressed(&mut self) {
        let usernameFocused = self.username.isFocused();
        self.username.setFocused(!usernameFocused);
        self.password.setFocused(usernameFocused);
    }

    pub fn enterPressed(&mut self) -> GuiAltCrackedAction {
        self.login()
    }

    fn login(&mut self) -> GuiAltCrackedAction {
        if self.receiver.is_some() {
            return GuiAltCrackedAction::None;
        }
        let username = self.username.getText().to_owned();
        let password = self.password.getText();
        if password.is_empty() {
            let session = Session::new(username.clone(), "", "", "mojang");
            self.status = format!("§aLogged in. ({username} - offline name)");
            return GuiAltCrackedAction::Authenticated(session);
        }

        self.status = "§7Waiting...".to_owned();
        let (sender, receiver) = mpsc::channel();
        let statusSender = sender.clone();
        let _ = thread::Builder::new()
            .name("Alt Login Thread".to_owned())
            .spawn(move || {
                let (uiSender, uiReceiver) = mpsc::channel::<String>();
                let relay = sender.clone();
                let relayThread = thread::spawn(move || {
                    while let Ok(status) = uiReceiver.recv() {
                        if relay.send(AltLoginEvent::Status(status)).is_err() {
                            break;
                        }
                    }
                });
                let result = login_with_credentials(&username, &password, Some(&uiSender));
                drop(uiSender);
                let _ = relayThread.join();
                match result {
                    Ok(login) => {
                        let _ = statusSender.send(AltLoginEvent::Success(login.session));
                    }
                    Err(_) => {
                        let _ = statusSender.send(AltLoginEvent::Failure);
                    }
                }
            });
        self.receiver = Some(receiver);
        self.updateButton();
        GuiAltCrackedAction::None
    }

    fn updateButton(&mut self) {
        let enabled = self.receiver.is_none();
        if let Some(button) = self
            .GuiScreen
            .buttonList
            .iter_mut()
            .find(|button| button.id == 0)
        {
            button.enabled = enabled;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_username_remains_loginable_like_exhibition() {
        let mut screen = GuiAltCracked::new();
        screen.initGui(854, 480);
        let login = screen
            .GuiScreen
            .buttonList
            .iter()
            .find(|button| button.id == 0)
            .unwrap();
        assert!(login.enabled);
        assert_eq!(
            screen.enterPressed(),
            GuiAltCrackedAction::Authenticated(Session::new("", "", "", "mojang"))
        );
    }

    #[test]
    fn tab_toggles_between_username_and_password() {
        let mut screen = GuiAltCracked::new();
        screen.initGui(854, 480);
        assert!(screen.username.isFocused());
        screen.tabPressed();
        assert!(screen.password.isFocused());
        screen.tabPressed();
        assert!(screen.username.isFocused());
    }
}
