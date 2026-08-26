use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::net::minecraft::client::account::Account::{current_time_millis, Account};
use crate::net::minecraft::client::account::AccountConfig::AccountConfig;
use crate::net::minecraft::client::account::MicrosoftAuth::{
    login_saved_account_cancelable, MicrosoftAuthError, MicrosoftLogin,
};
use crate::net::minecraft::client::account::SkinUpload::{upload_skin, SkinVariant};
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::util::Session::Session;
use crate::vulkan::GuiDrawList::GuiDrawList;
use crate::vulkan::NativeImage::NativeImage;

const LIST_TOP: i32 = 32;
const LIST_BOTTOM_MARGIN: i32 = 64;
const SLOT_HEIGHT: i32 = 27;
const LIST_WIDTH: i32 = 220;
const DEFAULT_SKIN: &str = "textures/entity/steve.png";
const DEFAULT_AVATAR_UUID: &str = "8667ba71-b85a-4004-af54-457a9734eed7";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiAccountManagerAction {
    Back,
    OpenMicrosoft,
    OpenOffline,
    OpenToken,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiAccountManagerInteraction {
    pub action: GuiAccountManagerAction,
    pub sound: Option<GuiSoundCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountManagerKey {
    Up,
    Down,
    Enter,
    Delete,
    Copy,
}

#[derive(Debug)]
enum AccountTaskEvent {
    Status(String),
    LoginSuccess {
        accountIndex: usize,
        account: Account,
        session: Session,
    },
    SkinSuccess,
    Failure(String),
    Cancelled,
}

#[derive(Debug)]
struct AccountTask {
    receiver: Receiver<AccountTaskEvent>,
    cancelled: Arc<AtomicBool>,
}

#[derive(Debug)]
enum AvatarEvent {
    Ready {
        key: String,
        location: ResourceLocation,
        image: NativeImage,
    },
    Failed {
        key: String,
    },
}

#[derive(Debug, Clone)]
struct Notification {
    message: String,
    expiresAt: Option<Instant>,
}

impl Notification {
    fn new(message: impl Into<String>, durationMillis: i64) -> Self {
        Self {
            message: message.into(),
            expiresAt: (durationMillis >= 0)
                .then(|| Instant::now() + Duration::from_millis(durationMillis as u64)),
        }
    }

    fn expired(&self) -> bool {
        self.expiresAt
            .is_some_and(|deadline| Instant::now() >= deadline)
    }
}

/// Port of Exhibition's `GuiAccountManager`.
///
/// The GUI keeps the original list dimensions, selection/reorder keys, button
/// layout, status text, token refresh behavior, Crafatar avatar cache and
/// native skin-file workflow. Rendering and session mutation are adapted to
/// this client's renderer-independent GUI and explicit `Minecraft::setSession`.
#[derive(Debug)]
pub struct GuiAccountManager {
    pub GuiScreen: GuiScreen,
    selectedAccount: Option<usize>,
    scrollOffset: i32,
    lastClickTime: u64,
    notification: Option<Notification>,
    task: Option<AccountTask>,
    avatarSender: Sender<AvatarEvent>,
    avatarReceiver: Receiver<AvatarEvent>,
    avatarRequested: HashSet<String>,
    avatarLocations: HashMap<String, ResourceLocation>,
    pendingAvatarTextures: Vec<(ResourceLocation, NativeImage)>,
    draggingList: bool,
    draggingScrollBar: bool,
    lastDragY: i32,
}

impl GuiAccountManager {
    pub fn new() -> Self {
        let (avatarSender, avatarReceiver) = mpsc::channel();
        Self {
            GuiScreen: GuiScreen::default(),
            selectedAccount: None,
            scrollOffset: 0,
            lastClickTime: 0,
            notification: None,
            task: None,
            avatarSender,
            avatarReceiver,
            avatarRequested: HashSet::new(),
            avatarLocations: HashMap::new(),
            pendingAvatarTextures: Vec::new(),
            draggingList: false,
            draggingScrollBar: false,
            lastDragY: 0,
        }
    }

    pub fn withNotification(message: impl Into<String>) -> Self {
        let mut screen = Self::new();
        screen.notification = Some(Notification::new(message, 5_000));
        screen
    }

    pub fn initGui(&mut self, width: i32, height: i32, config: &AccountConfig) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            0,
            width / 2 - 160,
            height - 48,
            78,
            20,
            "Login",
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            5,
            width / 2 + 3,
            height - 48,
            78,
            20,
            "Token",
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            1,
            width / 2 - 160,
            height - 24,
            78,
            20,
            "Microsoft",
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            4,
            width / 2 + 3,
            height - 24,
            78,
            20,
            "Offline",
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            7,
            width / 2 - 78,
            height - 24,
            78,
            20,
            "Change Skin",
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            2,
            width / 2 + 84,
            height - 48,
            78,
            20,
            "Delete",
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            3,
            width / 2 + 84,
            height - 24,
            78,
            20,
            "Back",
        ));
        if self
            .selectedAccount
            .is_some_and(|index| index >= config.len())
        {
            self.selectedAccount = None;
        }
        self.requestMissingAvatars(config);
        self.clampScroll(config.len());
        self.updateButtons(config.len());
    }

    pub fn updateScreen(&mut self, config: &mut AccountConfig) -> Option<Session> {
        if self
            .notification
            .as_ref()
            .is_some_and(Notification::expired)
        {
            self.notification = None;
        }

        let mut completed = None;
        let mut clearTask = false;
        if let Some(task) = &self.task {
            while let Ok(event) = task.receiver.try_recv() {
                match event {
                    AccountTaskEvent::Status(message) => {
                        self.notification = Some(Notification::new(message, -1));
                    }
                    AccountTaskEvent::LoginSuccess {
                        accountIndex,
                        account,
                        session,
                    } => {
                        if let Err(error) = config.replace(accountIndex, account) {
                            self.notification = Some(Notification::new(
                                format!("§cFailed saving account: {error}§r"),
                                5_000,
                            ));
                        } else {
                            self.notification = Some(Notification::new(
                                format!("§aSuccessful login! ({})§r", session.getUsername()),
                                5_000,
                            ));
                            completed = Some(session);
                        }
                        clearTask = true;
                    }
                    AccountTaskEvent::SkinSuccess => {
                        self.notification = Some(Notification::new("Skin changed!", 2_000));
                        clearTask = true;
                    }
                    AccountTaskEvent::Failure(message) => {
                        self.notification =
                            Some(Notification::new(format!("§c{message}§r"), 5_000));
                        clearTask = true;
                    }
                    AccountTaskEvent::Cancelled => {
                        self.notification = None;
                        clearTask = true;
                    }
                }
            }
        }
        if clearTask {
            self.task = None;
        }

        while let Ok(event) = self.avatarReceiver.try_recv() {
            match event {
                AvatarEvent::Ready {
                    key,
                    location,
                    image,
                } => {
                    self.avatarLocations.insert(key, location.clone());
                    self.pendingAvatarTextures.push((location, image));
                }
                AvatarEvent::Failed { key } => {
                    // Keep the failed key marked as requested, matching
                    // ThreadDownloadImageData's non-spamming failure behavior.
                    self.avatarRequested.insert(key);
                }
            }
        }
        self.requestMissingAvatars(config);
        self.updateButtons(config.len());
        completed
    }

    pub fn takePendingAvatarTextures(&mut self) -> Vec<(ResourceLocation, NativeImage)> {
        std::mem::take(&mut self.pendingAvatarTextures)
    }

    pub fn drawScreen(
        &mut self,
        drawList: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
        config: &AccountConfig,
        currentUsername: &str,
    ) {
        self.GuiScreen.drawDefaultBackground(drawList);
        self.drawAccountList(drawList, font, config, currentUsername);
        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            &format!("§rAccount Manager §8(§7{}§8)§r", config.len()),
            self.GuiScreen.width / 2,
            10,
            -1,
        );
        let status = self
            .notification
            .as_ref()
            .map(|value| value.message.clone())
            .unwrap_or_else(|| format!("Username: §7{currentUsername}"));
        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            &status,
            self.GuiScreen.width / 2,
            22,
            -1,
        );
        self.GuiScreen
            .drawScreen(drawList, font, mouseX, mouseY, partialTicks);
    }

    fn drawAccountList(
        &self,
        drawList: &mut GuiDrawList,
        font: &mut FontRenderer,
        config: &AccountConfig,
        currentUsername: &str,
    ) {
        let selectionLeft = self.GuiScreen.width / 2 - LIST_WIDTH / 2;
        let selectionRight = selectionLeft + LIST_WIDTH;
        let insideLeft = selectionLeft + 2;
        let bottom = self.GuiScreen.height - LIST_BOTTOM_MARGIN;

        // GuiSlot#drawContainerBackground covers the full slot viewport while
        // the actual entries remain centered inside the 220-pixel list width.
        drawList.draw_rect(
            0,
            LIST_TOP,
            self.GuiScreen.width,
            bottom,
            0xFF20_2020_u32 as i32,
        );
        for (index, account) in config.iter().enumerate() {
            let y = LIST_TOP + 4 + index as i32 * SLOT_HEIGHT - self.scrollOffset;
            let slotBodyHeight = SLOT_HEIGHT - 4;
            if y + slotBodyHeight <= LIST_TOP || y >= bottom {
                continue;
            }
            if self.selectedAccount == Some(index) {
                drawList.draw_rect(
                    selectionLeft,
                    y - 2,
                    selectionRight,
                    y + slotBodyHeight + 2,
                    0xFF80_8080_u32 as i32,
                );
                drawList.draw_rect(
                    selectionLeft + 1,
                    y - 1,
                    selectionRight - 1,
                    y + slotBodyHeight + 1,
                    0xFF00_0000_u32 as i32,
                );
            }
            self.drawHead(drawList, insideLeft + 3, y + 1, 21, account);
            let username = if account.username.trim().is_empty() {
                "???"
            } else {
                account.username.as_str()
            };
            let (accountType, typeColor) = if !account.refreshToken.trim().is_empty() {
                (" (Microsoft)", "§9")
            } else if !account.accessToken.trim().is_empty() {
                (" (Token)", "§6")
            } else {
                ("", "§7")
            };
            let prefix = if currentUsername == username {
                "§a§l"
            } else {
                typeColor
            };
            font.draw_string_with_shadow(
                drawList,
                &format!("{prefix}{username}{accountType}§r"),
                (insideLeft + 30) as f32,
                (y + 3) as f32,
                -1,
            );
            font.draw_string_with_shadow(
                drawList,
                &format!("§8§o{}§r", formatTimestamp(account.timestamp)),
                (insideLeft + 30) as f32,
                (y + 14) as f32,
                -1,
            );
        }

        // GuiSlot#overlayBackground hides partially scrolled entries above
        // and below the viewport before the title and buttons are drawn.
        drawList.draw_rect(0, 0, self.GuiScreen.width, LIST_TOP, 0xFF20_2020_u32 as i32);
        drawList.draw_rect(
            0,
            bottom,
            self.GuiScreen.width,
            self.GuiScreen.height,
            0xFF20_2020_u32 as i32,
        );

        // GuiSlot's four-pixel fade masks and six-pixel scrollbar.
        drawList.draw_gradient_rect(
            0,
            LIST_TOP,
            self.GuiScreen.width,
            LIST_TOP + 4,
            0xFF00_0000_u32 as i32,
            0x0100_0000,
        );
        drawList.draw_gradient_rect(
            0,
            bottom - 4,
            self.GuiScreen.width,
            bottom,
            0x0100_0000,
            0xFF00_0000_u32 as i32,
        );
        self.drawScrollBar(drawList, config.len());
    }

    fn drawHead(&self, drawList: &mut GuiDrawList, x: i32, y: i32, size: i32, account: &Account) {
        let key = avatarKey(account);
        if let Some(location) = self.avatarLocations.get(&key) {
            drawList.draw_modal_rect_with_custom_sized_texture(
                location.clone(),
                x as f32,
                y as f32,
                0.0,
                0.0,
                size as f32,
                size as f32,
                size as f32,
                size as f32,
            );
            return;
        }
        drawDefaultHead(drawList, x, y, size);
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
        config: &mut AccountConfig,
        currentAccessToken: &str,
    ) -> Option<GuiAccountManagerInteraction> {
        if mouseButton != 0 {
            return None;
        }
        let left = self.GuiScreen.width / 2 - LIST_WIDTH / 2;
        let bottom = self.GuiScreen.height - LIST_BOTTOM_MARGIN;
        let scrollBarLeft = self.GuiScreen.width / 2 + 124;
        if mouseY >= LIST_TOP && mouseY <= bottom {
            self.lastDragY = mouseY;
            if mouseX >= scrollBarLeft && mouseX <= scrollBarLeft + 6 {
                self.draggingScrollBar = self.maxScroll(config.len()) > 0;
                self.draggingList = false;
                return Some(GuiAccountManagerInteraction {
                    action: GuiAccountManagerAction::None,
                    sound: None,
                });
            }

            self.draggingList = true;
            self.draggingScrollBar = false;
            if mouseX >= left && mouseX <= left + LIST_WIDTH {
                let relativeY = mouseY - LIST_TOP + self.scrollOffset - 4;
                if relativeY >= 0 {
                    let index = (relativeY / SLOT_HEIGHT) as usize;
                    if index < config.len() {
                        let now = current_time_millis();
                        let doubleClick = self.selectedAccount == Some(index)
                            && now.saturating_sub(self.lastClickTime) < 250;
                        self.selectedAccount = Some(index);
                        self.lastClickTime = now;
                        self.updateButtons(config.len());
                        if doubleClick && self.task.is_none() {
                            self.startLogin(index, config);
                        }
                    }
                }
            }
            return Some(GuiAccountManagerInteraction {
                action: GuiAccountManagerAction::None,
                sound: None,
            });
        }
        let buttonIndex = self
            .GuiScreen
            .buttonList
            .iter()
            .position(|button| button.mousePressed(mouseX, mouseY))?;
        let button = self.GuiScreen.buttonList[buttonIndex].clone();
        let action = match button.id {
            0 => {
                if let Some(index) = self.selectedAccount {
                    self.startLogin(index, config);
                }
                GuiAccountManagerAction::None
            }
            1 => GuiAccountManagerAction::OpenMicrosoft,
            2 => {
                if let Some(index) = self.selectedAccount {
                    if let Err(error) = config.remove(index) {
                        self.notification = Some(Notification::new(
                            format!("§cFailed deleting account: {error}§r"),
                            5_000,
                        ));
                    }
                    self.selectedAccount = if config.is_empty() {
                        None
                    } else {
                        Some(index.min(config.len() - 1))
                    };
                    self.clampScroll(config.len());
                    self.updateButtons(config.len());
                }
                GuiAccountManagerAction::None
            }
            3 => GuiAccountManagerAction::Back,
            4 => GuiAccountManagerAction::OpenOffline,
            5 => GuiAccountManagerAction::OpenToken,
            7 => {
                self.startSkinChange(currentAccessToken);
                GuiAccountManagerAction::None
            }
            _ => GuiAccountManagerAction::None,
        };
        Some(GuiAccountManagerInteraction {
            action,
            sound: Some(button.playPressSound()),
        })
    }

    pub fn mouseDragged(&mut self, mouseY: i32, accountCount: usize) -> bool {
        if !self.draggingList && !self.draggingScrollBar {
            return false;
        }
        let delta = mouseY - self.lastDragY;
        self.lastDragY = mouseY;
        if delta == 0 {
            return false;
        }

        let old = self.scrollOffset;
        if self.draggingScrollBar {
            let top = LIST_TOP;
            let bottom = self.GuiScreen.height - LIST_BOTTOM_MARGIN;
            let viewport = bottom - top;
            let content = (accountCount as i32 * SLOT_HEIGHT).max(1);
            let thumbHeight = (viewport * viewport / content).clamp(32, viewport - 8);
            let track = (viewport - thumbHeight).max(1);
            let maxScroll = self.maxScroll(accountCount);
            self.scrollOffset += ((delta as f32 * maxScroll as f32) / track as f32) as i32;
        } else {
            // GuiSlot subtracts the pointer delta so dragging the contents down
            // reveals earlier entries and dragging them up reveals later ones.
            self.scrollOffset -= delta;
        }
        self.clampScroll(accountCount);
        old != self.scrollOffset
    }

    pub fn mouseReleased(&mut self) {
        self.draggingList = false;
        self.draggingScrollBar = false;
    }

    pub fn keyPressed(
        &mut self,
        key: AccountManagerKey,
        control: bool,
        config: &mut AccountConfig,
    ) -> Option<GuiAccountManagerAction> {
        match key {
            AccountManagerKey::Up => self.moveSelection(-1, control, config),
            AccountManagerKey::Down => self.moveSelection(1, control, config),
            AccountManagerKey::Enter => {
                if let Some(index) = self.selectedAccount {
                    self.startLogin(index, config);
                }
            }
            AccountManagerKey::Delete => {
                if let Some(index) = self.selectedAccount {
                    if let Err(error) = config.remove(index) {
                        self.notification = Some(Notification::new(
                            format!("§cFailed deleting account: {error}§r"),
                            5_000,
                        ));
                    }
                    self.selectedAccount = if config.is_empty() {
                        None
                    } else {
                        Some(index.min(config.len() - 1))
                    };
                    self.clampScroll(config.len());
                }
            }
            AccountManagerKey::Copy => self.copySelected(config),
        }
        self.updateButtons(config.len());
        Some(GuiAccountManagerAction::None)
    }

    pub fn scroll(&mut self, lines: f32, accountCount: usize) -> bool {
        let old = self.scrollOffset;
        self.scrollOffset = (self.scrollOffset - (lines * SLOT_HEIGHT as f32 / 3.0) as i32).max(0);
        self.clampScroll(accountCount);
        old != self.scrollOffset
    }

    fn moveSelection(&mut self, delta: i32, control: bool, config: &mut AccountConfig) {
        if config.is_empty() {
            self.selectedAccount = None;
            return;
        }
        if self.selectedAccount.is_none() && delta < 0 {
            // Exhibition starts at -1; pressing Up before any selection does
            // nothing, while pressing Down selects the first account.
            return;
        }
        let current = self.selectedAccount.map(|value| value as i32).unwrap_or(-1);
        let next = (current + delta).clamp(0, config.len() as i32 - 1) as usize;
        if control {
            if let Some(current) = self.selectedAccount {
                match config.swap(current, next) {
                    Ok(true) => self.selectedAccount = Some(next),
                    Ok(false) => {}
                    Err(error) => {
                        self.notification = Some(Notification::new(
                            format!("§cFailed saving account order: {error}§r"),
                            5_000,
                        ));
                    }
                }
            } else {
                self.selectedAccount = Some(next);
            }
        } else {
            self.selectedAccount = Some(next);
        }
        self.ensureSelectedVisible(config.len());
    }

    fn startLogin(&mut self, index: usize, config: &AccountConfig) {
        if self.task.is_some() {
            return;
        }
        let Some(account) = config.get(index).cloned() else {
            return;
        };
        let originalUsername = if account.username.trim().is_empty() {
            "???".to_owned()
        } else {
            account.username.clone()
        };
        if account.refreshToken.trim().is_empty() && account.accessToken.trim().is_empty() {
            self.notification = Some(Notification::new(
                format!("§cCannot login: Account {originalUsername} has no token information.§r"),
                5_000,
            ));
            return;
        }
        self.notification = Some(Notification::new(
            format!("§7Logging in... ({originalUsername})§r"),
            -1,
        ));
        let (eventSender, receiver) = mpsc::channel::<AccountTaskEvent>();
        let (statusSender, statusReceiver) = mpsc::channel::<String>();
        let statusEventSender = eventSender.clone();
        let statusRelay = thread::Builder::new()
            .name("Exhibition Account Login Status".to_owned())
            .spawn(move || {
                while let Ok(status) = statusReceiver.recv() {
                    if statusEventSender
                        .send(AccountTaskEvent::Status(status))
                        .is_err()
                    {
                        break;
                    }
                }
            })
            .ok();
        let cancelled = Arc::new(AtomicBool::new(false));
        let threadCancelled = Arc::clone(&cancelled);
        let _ = thread::Builder::new()
            .name("Exhibition Account Login".to_owned())
            .spawn(move || {
                let result = login_saved_account_cancelable(
                    &account.accessToken,
                    &account.refreshToken,
                    Some(&statusSender),
                    Some(threadCancelled.as_ref()),
                );
                drop(statusSender);
                if let Some(statusRelay) = statusRelay {
                    let _ = statusRelay.join();
                }
                match result {
                    Ok(MicrosoftLogin {
                        session,
                        refreshToken,
                        accessToken,
                    }) => {
                        let updated = Account::new(
                            refreshToken,
                            accessToken,
                            session.getUsername(),
                            current_time_millis(),
                            account.uuid,
                        );
                        let _ = eventSender.send(AccountTaskEvent::LoginSuccess {
                            accountIndex: index,
                            account: updated,
                            session,
                        });
                    }
                    Err(MicrosoftAuthError::Cancelled) => {
                        let _ = eventSender.send(AccountTaskEvent::Cancelled);
                    }
                    Err(error) => {
                        let _ = eventSender.send(AccountTaskEvent::Failure(format!(
                            "Login failed for {originalUsername}: {error}"
                        )));
                    }
                }
            });
        self.task = Some(AccountTask {
            receiver,
            cancelled,
        });
        self.updateButtons(config.len());
    }

    fn startSkinChange(&mut self, accessToken: &str) {
        if self.task.is_some() {
            return;
        }
        if accessToken.trim().is_empty() {
            self.notification = Some(Notification::new(
                "Failed to change skin: current session has no Minecraft access token.",
                2_000,
            ));
            return;
        }
        let token = accessToken.to_owned();
        let (sender, receiver) = mpsc::channel();
        let cancelled = Arc::new(AtomicBool::new(false));
        let threadCancelled = Arc::clone(&cancelled);
        let _ = thread::Builder::new()
            .name("Exhibition Skin Upload".to_owned())
            .spawn(move || match chooseSkinFileAndVariant() {
                Ok(Some((path, variant))) => {
                    if threadCancelled.load(Ordering::Acquire) {
                        let _ = sender.send(AccountTaskEvent::Cancelled);
                        return;
                    }
                    let _ =
                        sender.send(AccountTaskEvent::Status("§7Uploading skin...§r".to_owned()));
                    match upload_skin(&path, variant, &token) {
                        Ok(()) => {
                            let _ = sender.send(AccountTaskEvent::SkinSuccess);
                        }
                        Err(error) => {
                            let _ = sender.send(AccountTaskEvent::Failure(format!(
                                "Failed to change skin: {error}"
                            )));
                        }
                    }
                }
                Ok(None) => {
                    let _ = sender.send(AccountTaskEvent::Cancelled);
                }
                Err(error) => {
                    let _ = sender.send(AccountTaskEvent::Failure(error));
                }
            });
        self.task = Some(AccountTask {
            receiver,
            cancelled,
        });
        self.updateButtons(usize::MAX);
    }

    fn copySelected(&mut self, config: &AccountConfig) {
        let Some(account) = self.selectedAccount.and_then(|index| config.get(index)) else {
            return;
        };
        let value = if !account.username.trim().is_empty() && account.username != "???" {
            Some((account.username.as_str(), "§aCopied username to clipboard!"))
        } else if !account.accessToken.trim().is_empty() {
            Some((
                account.accessToken.as_str(),
                "§aCopied access token to clipboard!",
            ))
        } else {
            None
        };
        let Some((value, success)) = value else {
            return;
        };
        match setClipboardString(value) {
            Ok(()) => self.notification = Some(Notification::new(success, 2_000)),
            Err(error) => {
                self.notification = Some(Notification::new(
                    format!("§cFailed copying to clipboard: {error}§r"),
                    2_000,
                ));
            }
        }
    }

    fn requestMissingAvatars(&mut self, config: &AccountConfig) {
        for account in config.iter() {
            let key = avatarKey(account);
            if self.avatarRequested.contains(&key) || self.avatarLocations.contains_key(&key) {
                continue;
            }
            self.avatarRequested.insert(key.clone());
            let sender = self.avatarSender.clone();
            let _ = thread::Builder::new()
                .name(format!("Exhibition Avatar {key}"))
                .spawn(move || match downloadAvatar(&key) {
                    Ok((location, image)) => {
                        let _ = sender.send(AvatarEvent::Ready {
                            key,
                            location,
                            image,
                        });
                    }
                    Err(_) => {
                        let _ = sender.send(AvatarEvent::Failed { key });
                    }
                });
        }
    }

    fn updateButtons(&mut self, accountCount: usize) {
        let enabled = self.task.is_none()
            && self
                .selectedAccount
                .is_some_and(|index| index < accountCount);
        for button in &mut self.GuiScreen.buttonList {
            if matches!(button.id, 0 | 2) {
                button.enabled = enabled;
            }
        }
    }

    fn maxScroll(&self, accountCount: usize) -> i32 {
        let viewport = (self.GuiScreen.height - LIST_BOTTOM_MARGIN - LIST_TOP - 4).max(0);
        let content = accountCount as i32 * SLOT_HEIGHT;
        (content - viewport).max(0)
    }

    fn drawScrollBar(&self, drawList: &mut GuiDrawList, accountCount: usize) {
        let maxScroll = self.maxScroll(accountCount);
        if maxScroll <= 0 {
            return;
        }
        let top = LIST_TOP;
        let bottom = self.GuiScreen.height - LIST_BOTTOM_MARGIN;
        let viewport = bottom - top;
        let content = (accountCount as i32 * SLOT_HEIGHT).max(1);
        let thumbHeight = (viewport * viewport / content).clamp(32, viewport - 8);
        let thumbY = top + self.scrollOffset * (viewport - thumbHeight) / maxScroll;
        let left = self.GuiScreen.width / 2 + 124;
        let right = left + 6;
        drawList.draw_rect(left, top, right, bottom, 0xFF00_0000_u32 as i32);
        drawList.draw_rect(
            left,
            thumbY,
            right,
            thumbY + thumbHeight,
            0xFF80_8080_u32 as i32,
        );
        drawList.draw_rect(
            left,
            thumbY,
            right - 1,
            thumbY + thumbHeight - 1,
            0xFFC0_C0C0_u32 as i32,
        );
    }

    fn clampScroll(&mut self, accountCount: usize) {
        self.scrollOffset = self.scrollOffset.clamp(0, self.maxScroll(accountCount));
    }

    fn ensureSelectedVisible(&mut self, accountCount: usize) {
        if let Some(index) = self.selectedAccount {
            let top = 4 + index as i32 * SLOT_HEIGHT;
            let bottom = top + SLOT_HEIGHT;
            let visible = (self.GuiScreen.height - LIST_BOTTOM_MARGIN - LIST_TOP).max(0);
            if top < self.scrollOffset {
                self.scrollOffset = top;
            } else if bottom > self.scrollOffset + visible {
                self.scrollOffset = bottom - visible;
            }
        }
        self.clampScroll(accountCount);
    }
}

impl Drop for GuiAccountManager {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.cancelled.store(true, Ordering::Release);
        }
    }
}

fn avatarKey(account: &Account) -> String {
    if account.uuid.trim().is_empty() {
        DEFAULT_AVATAR_UUID.to_owned()
    } else {
        account.uuid.clone()
    }
}

fn downloadAvatar(key: &str) -> Result<(ResourceLocation, NativeImage), String> {
    let response = ureq::get(&format!("http://crafatar.com/avatars/{key}"))
        .timeout(Duration::from_secs(30))
        .call()
        .map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    let image = NativeImage::decode_png(&bytes).map_err(|error| error.to_string())?;
    let location = ResourceLocation::new("exhibition", format!("skins/{key}?overlay=true"));
    Ok((location, image))
}

fn drawDefaultHead(drawList: &mut GuiDrawList, x: i32, y: i32, size: i32) {
    let skin = ResourceLocation::parse(DEFAULT_SKIN);
    let u0 = 8.0 / 64.0;
    let v0 = 8.0 / 64.0;
    let u1 = 16.0 / 64.0;
    let v1 = 16.0 / 64.0;
    drawList.push_textured_quad(
        skin.clone(),
        [
            (x as f32, (y + size) as f32, u0, v1, 0xFFFF_FFFF),
            ((x + size) as f32, (y + size) as f32, u1, v1, 0xFFFF_FFFF),
            ((x + size) as f32, y as f32, u1, v0, 0xFFFF_FFFF),
            (x as f32, y as f32, u0, v0, 0xFFFF_FFFF),
        ],
    );
    let u0 = 40.0 / 64.0;
    let u1 = 48.0 / 64.0;
    drawList.push_textured_quad(
        skin,
        [
            (x as f32, (y + size) as f32, u0, v1, 0xFFFF_FFFF),
            ((x + size) as f32, (y + size) as f32, u1, v1, 0xFFFF_FFFF),
            ((x + size) as f32, y as f32, u1, v0, 0xFFFF_FFFF),
            (x as f32, y as f32, u0, v0, 0xFFFF_FFFF),
        ],
    );
}

fn chooseSkinFileAndVariant() -> Result<Option<(PathBuf, SkinVariant)>, String> {
    #[cfg(target_os = "windows")]
    {
        let script = r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$owner = New-Object System.Windows.Forms.Form
$owner.StartPosition = 'CenterScreen'
$owner.TopMost = $true
$owner.ShowInTaskbar = $false
$owner.FormBorderStyle = 'FixedToolWindow'
$owner.Size = New-Object System.Drawing.Size(1, 1)
$owner.Opacity = 0
$owner.Show()
$d = New-Object System.Windows.Forms.OpenFileDialog
$d.Filter = 'PNG skin (*.png)|*.png'
$d.Multiselect = $false
$d.ShowHelp = $false
if ($d.ShowDialog($owner) -ne 'OK') { $owner.Close(); exit 0 }
$r = [System.Windows.Forms.MessageBox]::Show($owner, 'Is this a slim skin?', 'alert', 'YesNoCancel')
$owner.Close()
if ($r -eq 'Cancel') { exit 0 }
$v = if ($r -eq 'Yes') { 'slim' } else { 'classic' }
[Console]::Write($v + [char]31 + $d.FileName)
"#;
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
            .map_err(|error| format!("Failed opening skin chooser: {error}"))?;
        if !output.status.success() {
            return Err("Failed opening skin chooser.".to_owned());
        }
        let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if value.is_empty() {
            return Ok(None);
        }
        let Some((variant, path)) = value.split_once('\u{1f}') else {
            return Err("Skin chooser returned an invalid result.".to_owned());
        };
        if !path.ends_with(".png") {
            return Err("Its seems that the file isn't a skin..".to_owned());
        }
        let variant = if variant == "slim" {
            SkinVariant::Slim
        } else {
            SkinVariant::Classic
        };
        Ok(Some((PathBuf::from(path), variant)))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("The Exhibition native skin chooser is currently available on Windows only.".to_owned())
    }
}

fn setClipboardString(value: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let mut child = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Set-Clipboard -Value ([Console]::In.ReadToEnd())",
            ])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|error| error.to_string())?;
        child
            .stdin
            .as_mut()
            .ok_or_else(|| "clipboard process has no stdin".to_owned())?
            .write_all(value.as_bytes())
            .map_err(|error| error.to_string())?;
        let status = child.wait().map_err(|error| error.to_string())?;
        return status
            .success()
            .then_some(())
            .ok_or_else(|| "PowerShell Set-Clipboard failed".to_owned());
    }
    #[cfg(target_os = "macos")]
    {
        return writeClipboardProcess("pbcopy", &[], value);
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        writeClipboardProcess("wl-copy", &[], value)
            .or_else(|_| writeClipboardProcess("xclip", &["-selection", "clipboard"], value))
            .or_else(|_| writeClipboardProcess("xsel", &["--clipboard", "--input"], value))
    }
}

#[cfg(not(target_os = "windows"))]
fn writeClipboardProcess(command: &str, args: &[&str], value: &str) -> Result<(), String> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| "clipboard process has no stdin".to_owned())?
        .write_all(value.as_bytes())
        .map_err(|error| error.to_string())?;
    let status = child.wait().map_err(|error| error.to_string())?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("{command} failed"))
}

fn formatTimestamp(timestampMillis: u64) -> String {
    let seconds = (timestampMillis / 1_000) as i64;
    if let Some((year, month, day, hour, minute, second)) = localTimeParts(seconds) {
        return format!("{day:02}.{month:02}.{year:04} {hour:02}:{minute:02}:{second:02}");
    }

    // The game is primarily shipped on Windows. Keep a deterministic UTC
    // fallback for unsupported targets or a C-runtime conversion failure.
    let days = seconds.div_euclid(86_400);
    let daySeconds = seconds.rem_euclid(86_400) as u64;
    let hour = daySeconds / 3_600;
    let minute = (daySeconds % 3_600) / 60;
    let second = daySeconds % 60;
    let (year, month, day) = civilFromDays(days);
    format!("{day:02}.{month:02}.{year:04} {hour:02}:{minute:02}:{second:02}")
}

#[cfg(target_os = "windows")]
fn localTimeParts(seconds: i64) -> Option<(i64, i64, i64, u64, u64, u64)> {
    use std::mem::MaybeUninit;
    use std::os::raw::c_int;

    #[repr(C)]
    struct Tm {
        tm_sec: c_int,
        tm_min: c_int,
        tm_hour: c_int,
        tm_mday: c_int,
        tm_mon: c_int,
        tm_year: c_int,
        tm_wday: c_int,
        tm_yday: c_int,
        tm_isdst: c_int,
    }

    extern "C" {
        fn _localtime64_s(result: *mut Tm, time: *const i64) -> c_int;
    }

    let mut value = MaybeUninit::<Tm>::uninit();
    let result = unsafe { _localtime64_s(value.as_mut_ptr(), &seconds) };
    if result != 0 {
        return None;
    }
    let value = unsafe { value.assume_init() };
    Some((
        i64::from(value.tm_year + 1900),
        i64::from(value.tm_mon + 1),
        i64::from(value.tm_mday),
        value.tm_hour.max(0) as u64,
        value.tm_min.max(0) as u64,
        value.tm_sec.max(0) as u64,
    ))
}

#[cfg(not(target_os = "windows"))]
fn localTimeParts(_seconds: i64) -> Option<(i64, i64, i64, u64, u64, u64)> {
    None
}

fn civilFromDays(daysSinceEpoch: i64) -> (i64, i64, i64) {
    let z = daysSinceEpoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_matches_exhibition_pattern() {
        let value = formatTimestamp(0);
        assert_eq!(value.len(), 19);
        assert_eq!(&value[2..3], ".");
        assert_eq!(&value[5..6], ".");
        assert_eq!(&value[10..11], " ");
        assert_eq!(&value[13..14], ":");
        assert_eq!(&value[16..17], ":");
    }

    #[test]
    fn buttons_match_exhibition_layout() {
        let root = std::env::temp_dir().join("mc112-account-layout-test");
        let config = AccountConfig::load(&root);
        let mut screen = GuiAccountManager::new();
        screen.initGui(854, 480, &config);
        let expected = [
            (0, 267, 432, "Login"),
            (5, 430, 432, "Token"),
            (1, 267, 456, "Microsoft"),
            (4, 430, 456, "Offline"),
            (7, 349, 456, "Change Skin"),
            (2, 511, 432, "Delete"),
            (3, 511, 456, "Back"),
        ];
        for (id, x, y, text) in expected {
            let button = screen
                .GuiScreen
                .buttonList
                .iter()
                .find(|button| button.id == id)
                .unwrap();
            assert_eq!(
                (button.x, button.y, button.displayString.as_str()),
                (x, y, text)
            );
        }
    }

    #[test]
    fn list_geometry_matches_gui_slot_defaults() {
        let root = std::env::temp_dir().join("mc112-account-list-layout-test");
        let config = AccountConfig::load(&root);
        let mut screen = GuiAccountManager::new();
        screen.initGui(854, 480, &config);
        assert_eq!(LIST_WIDTH, 220);
        assert_eq!(854 / 2 - LIST_WIDTH / 2 + 2, 319);
        assert_eq!(854 / 2 + 124, 551);
    }

    #[test]
    fn empty_uuid_uses_exhibition_default_avatar() {
        let account = Account::new("", "", "Player", 0, "");
        assert_eq!(avatarKey(&account), DEFAULT_AVATAR_UUID);
    }

    #[test]
    fn up_does_not_select_last_account_from_minus_one_state() {
        let root =
            std::env::temp_dir().join(format!("mc112-account-key-test-{}", current_time_millis()));
        let mut config = AccountConfig::load(&root);
        config
            .add(Account::new("r", "a", "First", 1, "u1"))
            .unwrap();
        config
            .add(Account::new("r", "a", "Second", 2, "u2"))
            .unwrap();
        let mut screen = GuiAccountManager::new();
        screen.initGui(854, 480, &config);
        screen.keyPressed(AccountManagerKey::Up, false, &mut config);
        assert_eq!(screen.selectedAccount, None);
        screen.keyPressed(AccountManagerKey::Down, false, &mut config);
        assert_eq!(screen.selectedAccount, Some(0));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn scrollbar_drag_scales_to_gui_slot_max_scroll() {
        let root = std::env::temp_dir().join(format!(
            "mc112-account-scroll-test-{}",
            current_time_millis()
        ));
        let mut config = AccountConfig::load(&root);
        for index in 0..40 {
            config
                .add(Account::new(
                    "r",
                    "a",
                    format!("P{index}"),
                    index,
                    format!("u{index}"),
                ))
                .unwrap();
        }
        let mut screen = GuiAccountManager::new();
        screen.initGui(854, 480, &config);
        let bar_x = 854 / 2 + 124;
        let _ = screen.mouseClicked(bar_x, LIST_TOP + 10, 0, &mut config, "");
        assert!(screen.draggingScrollBar);
        assert!(screen.mouseDragged(300, config.len()));
        assert!(screen.scrollOffset > 0);
        screen.mouseReleased();
        assert!(!screen.draggingScrollBar);
        let _ = std::fs::remove_dir_all(root);
    }
}
