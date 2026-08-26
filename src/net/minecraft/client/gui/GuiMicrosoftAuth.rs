use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread;

use crate::net::minecraft::client::account::Account::{current_time_millis, Account};
use crate::net::minecraft::client::account::AccountConfig::AccountConfig;
use crate::net::minecraft::client::account::MicrosoftAuth::{interactive_login, MicrosoftLogin};
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::util::Session::Session;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiMicrosoftAuthAction {
    Cancel,
    Authenticated(Session),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiMicrosoftAuthInteraction {
    pub action: GuiMicrosoftAuthAction,
    pub sound: Option<GuiSoundCommand>,
}

#[derive(Debug)]
enum AuthEvent {
    Status(String),
    Success(MicrosoftLogin),
    Failure(String),
}

#[derive(Debug)]
pub struct GuiMicrosoftAuth {
    pub GuiScreen: GuiScreen,
    status: String,
    cause: Option<String>,
    receiver: Option<Receiver<AuthEvent>>,
    finishedSession: Option<Session>,
    cancelled: Arc<AtomicBool>,
}

impl GuiMicrosoftAuth {
    pub fn new() -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            status: "§fCheck your browser to continue...§r".to_owned(),
            cause: None,
            receiver: None,
            finishedSession: None,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.GuiScreen.buttonList.push(GuiButton::new(
            0,
            width / 2 - 100,
            height / 2 + 13,
            "Cancel",
        ));
        if self.receiver.is_none() && self.finishedSession.is_none() {
            self.start();
        }
    }

    fn start(&mut self) {
        self.cancelled.store(false, Ordering::Release);
        let cancelled = Arc::clone(&self.cancelled);
        let (sender, receiver) = mpsc::channel();
        let statusSender = sender.clone();
        let _ = thread::Builder::new()
            .name("Exhibition Microsoft Auth".to_owned())
            .spawn(move || {
                let (uiSender, uiReceiver) = mpsc::channel::<String>();
                let relay = sender.clone();
                let relayThread = thread::spawn(move || {
                    while let Ok(status) = uiReceiver.recv() {
                        if relay.send(AuthEvent::Status(status)).is_err() {
                            break;
                        }
                    }
                });
                let result = interactive_login(Some(&uiSender), Some(cancelled.as_ref()));
                drop(uiSender);
                let _ = relayThread.join();
                match result {
                    Ok(login) => {
                        let _ = statusSender.send(AuthEvent::Success(login));
                    }
                    Err(error) => {
                        let _ = statusSender.send(AuthEvent::Failure(error.to_string()));
                    }
                }
            });
        self.receiver = Some(receiver);
    }

    pub fn updateScreen(&mut self, config: &mut AccountConfig) -> Option<GuiMicrosoftAuthAction> {
        let mut clear = false;
        if let Some(receiver) = &self.receiver {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    AuthEvent::Status(status) => self.status = status,
                    AuthEvent::Success(login) => {
                        let accountUuid = uuid::Uuid::parse_str(login.session.getPlayerID())
                            .map(|value| value.to_string())
                            .unwrap_or_else(|_| login.session.getPlayerID().to_owned());
                        let account = Account::new(
                            login.refreshToken,
                            login.accessToken,
                            login.session.getUsername(),
                            current_time_millis(),
                            accountUuid,
                        );
                        if let Err(error) = config.add(account) {
                            self.status = "§cFailed saving Microsoft account§r".to_owned();
                            self.cause = Some(format!("§c{error}§r"));
                        } else {
                            self.status =
                                format!("§aSuccessful login! ({})§r", login.session.getUsername());
                            self.finishedSession = Some(login.session);
                        }
                        clear = true;
                    }
                    AuthEvent::Failure(error) => {
                        self.status = "§cMicrosoft authentication failed§r".to_owned();
                        self.cause = Some(format!("§c{error}§r"));
                        clear = true;
                    }
                }
            }
        }
        if clear {
            self.receiver = None;
        }
        self.finishedSession
            .take()
            .map(GuiMicrosoftAuthAction::Authenticated)
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
            "Microsoft Authentication",
            self.GuiScreen.width / 2,
            self.GuiScreen.height / 2 - 22,
            11_184_810,
        );
        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            &self.status,
            self.GuiScreen.width / 2,
            self.GuiScreen.height / 2 - 4,
            -1,
        );
        if let Some(cause) = &self.cause {
            let width = font.get_string_width(cause);
            drawList.draw_rect(
                0,
                self.GuiScreen.height - font.font_height - 5,
                width + 6,
                self.GuiScreen.height,
                0x6400_0000_u32 as i32,
            );
            font.draw_string_with_shadow(
                drawList,
                cause,
                3.0,
                (self.GuiScreen.height - font.font_height - 2) as f32,
                -1,
            );
        }
        self.GuiScreen
            .drawScreen(drawList, font, mouseX, mouseY, partialTicks);
    }

    pub fn cancel(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        self.receiver = None;
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
    ) -> Option<GuiMicrosoftAuthInteraction> {
        if mouseButton != 0 {
            return None;
        }
        let button = self
            .GuiScreen
            .buttonList
            .iter()
            .find(|button| button.mousePressed(mouseX, mouseY))?
            .clone();
        self.cancel();
        Some(GuiMicrosoftAuthInteraction {
            action: GuiMicrosoftAuthAction::Cancel,
            sound: Some(button.playPressSound()),
        })
    }
}

impl Drop for GuiMicrosoftAuth {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
    }
}
