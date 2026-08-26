use std::sync::mpsc::{self, Receiver};
use std::thread;

use crate::net::minecraft::client::account::Account::{current_time_millis, Account};
use crate::net::minecraft::client::account::AccountConfig::AccountConfig;
use crate::net::minecraft::client::account::MicrosoftAuth::{token_login, MicrosoftLogin};
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::gui::GuiTextField::{
    GuiTextField, GuiTextFieldKey, GuiTextFieldModifiers,
};
use crate::net::minecraft::util::Session::Session;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiSessionLoginAction {
    Cancel,
    Authenticated(Session),
    None,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GuiSessionLoginInteraction {
    pub action: GuiSessionLoginAction,
    pub sound: Option<GuiSoundCommand>,
}

#[derive(Debug)]
enum TokenEvent {
    Status(String),
    Success(MicrosoftLogin),
    Failure(&'static str),
}

#[derive(Debug)]
pub struct GuiSessionLogin {
    pub GuiScreen: GuiScreen,
    sessionField: GuiTextField,
    status: Option<String>,
    receiver: Option<Receiver<TokenEvent>>,
    finishedSession: Option<Session>,
}

impl GuiSessionLogin {
    pub fn new() -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            sessionField: GuiTextField::new(1, 0, 0, 200, 20),
            status: None,
            receiver: None,
            finishedSession: None,
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.sessionField = GuiTextField::new(1, width / 2 - 100, height / 2, 200, 20);
        self.sessionField.setMaxStringLength(32_767);
        self.sessionField.setFocused(true);
        self.GuiScreen.buttonList.push(GuiButton::new(
            1,
            width / 2 - 100,
            height / 2 + 35,
            "Login",
        ));
        self.GuiScreen.buttonList.push(GuiButton::new(
            0,
            width / 2 - 100,
            height / 2 + 65,
            "Cancel",
        ));
    }

    pub fn updateScreen(&mut self, config: &mut AccountConfig) -> Option<GuiSessionLoginAction> {
        self.sessionField.updateCursorCounter();
        let mut clear = false;
        if let Some(receiver) = &self.receiver {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    TokenEvent::Status(status) => self.status = Some(status),
                    TokenEvent::Success(login) => {
                        let account = Account::new(
                            login.refreshToken,
                            login.accessToken,
                            login.session.getUsername(),
                            current_time_millis(),
                            login.session.getPlayerID(),
                        );
                        if let Err(error) = config.add(account) {
                            self.status = Some(format!("§4Failed saving account: {error}"));
                        } else {
                            self.status =
                                Some(format!("§2Logged in as {}", login.session.getUsername()));
                            self.finishedSession = Some(login.session);
                        }
                        clear = true;
                    }
                    TokenEvent::Failure(tokenType) => {
                        self.status = Some(format!("§4Invalid token ({tokenType})"));
                        clear = true;
                    }
                }
            }
        }
        if clear {
            self.receiver = None;
            self.updateButton();
        }
        self.finishedSession
            .take()
            .map(GuiSessionLoginAction::Authenticated)
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
        self.sessionField.drawTextBox(drawList, font);
        if let Some(status) = &self.status {
            font.draw_string_with_shadow(
                drawList,
                status,
                (self.GuiScreen.width / 2 - 100) as f32,
                (self.GuiScreen.height / 2 - 20) as f32,
                -1,
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
    ) -> Option<GuiSessionLoginInteraction> {
        self.sessionField
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
            0 => GuiSessionLoginAction::Cancel,
            1 => {
                self.startLogin();
                GuiSessionLoginAction::None
            }
            _ => GuiSessionLoginAction::None,
        };
        Some(GuiSessionLoginInteraction {
            action,
            sound: Some(button.playPressSound()),
        })
    }

    pub fn typedText(&mut self, text: &str, font: &FontRenderer) -> bool {
        let changed = self.sessionField.writeText(text, Some(font));
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
        let changed = self.sessionField.keyPressed(key, modifiers, font);
        if changed {
            self.updateButton();
        }
        changed
    }

    pub fn selectAll(&mut self, font: &FontRenderer) {
        self.sessionField.selectAll(font);
    }
    pub fn enterPressed(&mut self) {
        self.startLogin();
    }

    fn startLogin(&mut self) {
        if self.receiver.is_some() {
            return;
        }
        let token = self.sessionField.getText().to_owned();
        let tokenType = if token.starts_with("M.C") {
            "Refresh Token"
        } else {
            "Access Token"
        };
        self.status = Some("§7Logging in...".to_owned());
        let (sender, receiver) = mpsc::channel();
        let statusSender = sender.clone();
        let _ = thread::Builder::new()
            .name("Exhibition Token Login".to_owned())
            .spawn(move || {
                let (uiSender, uiReceiver) = mpsc::channel::<String>();
                let relay = sender.clone();
                let relayThread = thread::spawn(move || {
                    while let Ok(status) = uiReceiver.recv() {
                        if relay.send(TokenEvent::Status(status)).is_err() {
                            break;
                        }
                    }
                });
                let result = token_login(&token, Some(&uiSender));
                drop(uiSender);
                let _ = relayThread.join();
                match result {
                    Ok(login) => {
                        let _ = statusSender.send(TokenEvent::Success(login));
                    }
                    Err(_) => {
                        let _ = statusSender.send(TokenEvent::Failure(tokenType));
                    }
                }
            });
        self.receiver = Some(receiver);
        self.updateButton();
    }

    fn updateButton(&mut self) {
        let enabled = self.receiver.is_none();
        if let Some(button) = self
            .GuiScreen
            .buttonList
            .iter_mut()
            .find(|button| button.id == 1)
        {
            button.enabled = enabled;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_login_buttons_match_exhibition_layout() {
        let mut screen = GuiSessionLogin::new();
        screen.initGui(854, 480);
        let login = screen
            .GuiScreen
            .buttonList
            .iter()
            .find(|button| button.id == 1)
            .unwrap();
        let cancel = screen
            .GuiScreen
            .buttonList
            .iter()
            .find(|button| button.id == 0)
            .unwrap();
        assert_eq!(
            (login.x, login.y, login.displayString.as_str()),
            (327, 275, "Login")
        );
        assert_eq!(
            (cancel.x, cancel.y, cancel.displayString.as_str()),
            (327, 305, "Cancel")
        );
        assert!(login.enabled);
    }
}
