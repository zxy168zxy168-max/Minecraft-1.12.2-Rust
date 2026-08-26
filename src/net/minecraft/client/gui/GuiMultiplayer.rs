use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::Gui::ICONS;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::multiplayer::ServerData::ServerData;
use crate::net::minecraft::client::multiplayer::ServerList::ServerList;
use crate::net::minecraft::client::network::ServerPinger::{ServerPingFailure, ServerPinger};
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::vulkan::GuiDrawList::GuiDrawList;

const SLOT_HEIGHT: i32 = 36;
const LIST_TOP: i32 = 32;
const LIST_BOTTOM_MARGIN: i32 = 64;
const LIST_WIDTH: i32 = 305;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiMultiplayerAction {
    Select(ServerData),
    DirectConnect,
    AddServer,
    Edit { index: usize, server: ServerData },
    Delete { index: usize, serverName: String },
    Refresh,
    Cancel,
    SelectionChanged,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiMultiplayerInteraction {
    pub action: GuiMultiplayerAction,
    pub sound: Option<GuiSoundCommand>,
}

pub struct GuiMultiplayer {
    pub GuiScreen: GuiScreen,
    pub screenTitle: String,
    pub savedServerList: ServerList,
    oldServerPinger: ServerPinger,
    selectedIndex: Option<usize>,
    scrollOffset: i32,
    lastClickTime: u64,
    hoveringText: Option<String>,
    pingingText: String,
    cannotConnectText: String,
    cannotResolveText: String,
    noConnectionText: String,
    clientOutOfDateText: String,
    serverOutOfDateText: String,
}

impl GuiMultiplayer {
    pub fn new(gameDir: PathBuf) -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            screenTitle: "Play Multiplayer".to_owned(),
            savedServerList: ServerList::new(gameDir),
            oldServerPinger: ServerPinger::new(),
            selectedIndex: None,
            scrollOffset: 0,
            lastClickTime: 0,
            hoveringText: None,
            pingingText: "Pinging...".to_owned(),
            cannotConnectText: "Can't connect to server.".to_owned(),
            cannotResolveText: "Can't resolve hostname.".to_owned(),
            noConnectionText: "No connection".to_owned(),
            clientOutOfDateText: "Client out of date!".to_owned(),
            serverOutOfDateText: "Server out of date!".to_owned(),
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32, locale: &Locale) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.screenTitle = locale.translate_key("multiplayer.title").to_owned();
        self.pingingText = locale
            .translate_key("multiplayer.status.pinging")
            .to_owned();
        self.cannotConnectText = locale
            .translate_key("multiplayer.status.cannot_connect")
            .to_owned();
        self.cannotResolveText = locale
            .translate_key("multiplayer.status.cannot_resolve")
            .to_owned();
        self.noConnectionText = locale
            .translate_key("multiplayer.status.no_connection")
            .to_owned();
        self.clientOutOfDateText = locale
            .translate_key("multiplayer.status.client_out_of_date")
            .to_owned();
        self.serverOutOfDateText = locale
            .translate_key("multiplayer.status.server_out_of_date")
            .to_owned();
        let y = height - 52;
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            7,
            width / 2 - 154,
            height - 28,
            70,
            20,
            locale.translate_key("selectServer.edit"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            2,
            width / 2 - 74,
            height - 28,
            70,
            20,
            locale.translate_key("selectServer.delete"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            1,
            width / 2 - 154,
            y,
            100,
            20,
            locale.translate_key("selectServer.select"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            4,
            width / 2 - 50,
            y,
            100,
            20,
            locale.translate_key("selectServer.direct"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            3,
            width / 2 + 54,
            y,
            100,
            20,
            locale.translate_key("selectServer.add"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            8,
            width / 2 + 4,
            height - 28,
            70,
            20,
            locale.translate_key("selectServer.refresh"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            0,
            width / 2 + 80,
            height - 28,
            75,
            20,
            locale.translate_key("gui.cancel"),
        ));
        self.updateSelectionButtons();
        self.startUnpingedServers();
        self.clampScroll();
    }

    pub fn updateScreen(&mut self) -> bool {
        let mut changed = false;
        for result in self.oldServerPinger.pingPendingNetworks() {
            let Some(server) = self.savedServerList.getServerDataMut(result.serverIndex) else {
                continue;
            };
            if server.serverIP != result.serverIP {
                continue;
            }
            server.pinged = true;
            match result.result {
                Ok(status) => {
                    server.serverMOTD = status.serverMOTD;
                    server.populationInfo = status.populationInfo;
                    server.pingToServer = status.pingToServer;
                    server.version = status.version;
                    server.gameVersion = status.gameVersion;
                    server.playerList = status.playerList;
                    server.setBase64EncodedIconData(status.serverIcon);
                }
                Err(ServerPingFailure::CannotResolve) => {
                    server.pingToServer = -1;
                    server.serverMOTD = format!("§4{}", self.cannotResolveText);
                    server.populationInfo.clear();
                }
                Err(ServerPingFailure::CannotConnect(_)) => {
                    server.pingToServer = -1;
                    server.serverMOTD = format!("§4{}", self.cannotConnectText);
                    server.populationInfo.clear();
                }
            }
            changed = true;
        }
        changed
    }

    pub fn isPinging(&self) -> bool {
        self.oldServerPinger.hasPendingNetworks()
    }

    pub fn refreshServerList(&mut self) {
        self.oldServerPinger.clearPendingNetworks();
        if let Err(error) = self.savedServerList.loadServerList() {
            log::error!("Couldn't reload server list: {error}");
        }
        self.selectedIndex = None;
        self.scrollOffset = 0;
        for server in self.savedServerList.serversMut() {
            server.resetPingState(&self.pingingText);
        }
        self.startUnpingedServers();
        self.updateSelectionButtons();
    }

    pub fn addServer(&mut self, server: ServerData) -> std::io::Result<()> {
        self.savedServerList.addServerData(server);
        self.savedServerList.saveServerList()?;
        self.selectedIndex = None;
        self.startUnpingedServers();
        self.updateSelectionButtons();
        Ok(())
    }
    pub fn editServer(&mut self, index: usize, server: ServerData) -> std::io::Result<()> {
        self.savedServerList.set(index, server);
        self.savedServerList.saveServerList()?;
        if let Some(value) = self.savedServerList.getServerDataMut(index) {
            value.resetPingState(&self.pingingText);
        }
        self.startUnpingedServers();
        self.updateSelectionButtons();
        Ok(())
    }
    pub fn deleteServer(&mut self, index: usize) -> std::io::Result<()> {
        self.savedServerList.removeServerData(index);
        self.savedServerList.saveServerList()?;
        self.selectedIndex = None;
        self.clampScroll();
        self.updateSelectionButtons();
        Ok(())
    }
    pub fn selectedServer(&self) -> Option<(usize, ServerData)> {
        self.selectedIndex.and_then(|index| {
            self.savedServerList
                .getServerData(index)
                .cloned()
                .map(|server| (index, server))
        })
    }

    pub fn drawScreen(
        &mut self,
        drawList: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.hoveringText = None;
        self.GuiScreen.drawDefaultBackground(drawList);
        let listLeft = self.GuiScreen.width / 2 - LIST_WIDTH / 2;
        let listBottom = self.GuiScreen.height - LIST_BOTTOM_MARGIN;
        drawList.draw_rect(
            listLeft - 1,
            LIST_TOP - 1,
            listLeft + LIST_WIDTH + 1,
            listBottom + 1,
            0xA000_0000_u32 as i32,
        );
        let first = (self.scrollOffset / SLOT_HEIGHT).max(0) as usize;
        let offsetWithin = self.scrollOffset.rem_euclid(SLOT_HEIGHT);
        for index in first..self.savedServerList.countServers() {
            let y = LIST_TOP - offsetWithin + (index - first) as i32 * SLOT_HEIGHT;
            if y >= listBottom {
                break;
            }
            if y + SLOT_HEIGHT <= LIST_TOP {
                continue;
            }
            self.drawServerEntry(
                drawList, font, index, listLeft, y, LIST_WIDTH, mouseX, mouseY,
            );
        }
        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            &self.screenTitle,
            self.GuiScreen.width / 2,
            20,
            0x00FF_FFFF,
        );
        self.GuiScreen
            .drawScreen(drawList, font, mouseX, mouseY, partialTicks);
        if let Some(text) = &self.hoveringText {
            let width = font.get_string_width(text) + 8;
            let x = (mouseX + 12).min(self.GuiScreen.width - width - 4).max(4);
            let y = (mouseY - 12).max(4);
            drawList.draw_rect(x - 3, y - 4, x + width, y + 10, 0xF010_0010_u32 as i32);
            font.draw_string_with_shadow(drawList, text, x as f32, y as f32, 0x00FF_FFFF);
        }
    }

    fn drawServerEntry(
        &mut self,
        drawList: &mut GuiDrawList,
        font: &mut FontRenderer,
        index: usize,
        x: i32,
        y: i32,
        width: i32,
        mouseX: i32,
        mouseY: i32,
    ) {
        let Some(server) = self.savedServerList.getServerData(index) else {
            return;
        };
        if self.selectedIndex == Some(index) {
            drawList.draw_rect(x, y, x + width, y + 32, 0x8060_6060_u32 as i32);
        }
        let icon = ResourceLocation::parse("textures/misc/unknown_server.png");
        drawList.draw_modal_rect_with_custom_sized_texture(
            icon, x as f32, y as f32, 0.0, 0.0, 32.0, 32.0, 32.0, 32.0,
        );
        font.draw_string_with_shadow(
            drawList,
            &server.serverName,
            (x + 35) as f32,
            (y + 1) as f32,
            0x00FF_FFFF,
        );
        for (lineIndex, line) in wrapText(font, &server.serverMOTD, width - 51)
            .into_iter()
            .take(2)
            .enumerate()
        {
            font.draw_string_with_shadow(
                drawList,
                &line,
                (x + 35) as f32,
                (y + 12 + font.font_height * lineIndex as i32) as f32,
                8_421_504,
            );
        }
        let incompatible = server.version != 340;
        let rightText = if incompatible {
            format!("§4{}", server.gameVersion)
        } else {
            server.populationInfo.clone()
        };
        let rightWidth = font.get_string_width(&rightText);
        font.draw_string_with_shadow(
            drawList,
            &rightText,
            (x + width - rightWidth - 17) as f32,
            (y + 1) as f32,
            8_421_504,
        );
        let (column, row, tooltip) = if incompatible {
            (
                0,
                5,
                if server.version > 340 {
                    self.clientOutOfDateText.clone()
                } else {
                    self.serverOutOfDateText.clone()
                },
            )
        } else if server.pinged && server.pingToServer != -2 {
            let row = if server.pingToServer < 0 {
                5
            } else if server.pingToServer < 150 {
                0
            } else if server.pingToServer < 300 {
                1
            } else if server.pingToServer < 600 {
                2
            } else if server.pingToServer < 1000 {
                3
            } else {
                4
            };
            let tooltip = if server.pingToServer < 0 {
                self.noConnectionText.clone()
            } else {
                format!("{}ms", server.pingToServer)
            };
            (0, row, tooltip)
        } else {
            let mut row = ((currentTimeMillis() / 100 + (index as u64 * 2)) & 7) as i32;
            if row > 4 {
                row = 8 - row;
            }
            (1, row, self.pingingText.clone())
        };
        let pingX = x + width - 15;
        drawList.draw_modal_rect_with_custom_sized_texture(
            (*ICONS).clone(),
            pingX as f32,
            y as f32,
            (column * 10) as f32,
            (176 + row * 8) as f32,
            10.0,
            8.0,
            256.0,
            256.0,
        );
        if mouseX >= pingX && mouseX <= pingX + 10 && mouseY >= y && mouseY <= y + 8 {
            self.hoveringText = Some(tooltip);
        }
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
    ) -> Option<GuiMultiplayerInteraction> {
        if mouseButton != 0 {
            return None;
        }
        let listLeft = self.GuiScreen.width / 2 - LIST_WIDTH / 2;
        let listBottom = self.GuiScreen.height - LIST_BOTTOM_MARGIN;
        if mouseX >= listLeft
            && mouseX < listLeft + LIST_WIDTH
            && mouseY >= LIST_TOP
            && mouseY < listBottom
        {
            let index = ((mouseY - LIST_TOP + self.scrollOffset) / SLOT_HEIGHT) as usize;
            if index < self.savedServerList.countServers() {
                let now = currentTimeMillis();
                let doubleClick = self.selectedIndex == Some(index)
                    && now.saturating_sub(self.lastClickTime) < 250;
                self.selectedIndex = Some(index);
                self.lastClickTime = now;
                self.updateSelectionButtons();
                if doubleClick {
                    return self
                        .savedServerList
                        .getServerData(index)
                        .cloned()
                        .map(|server| GuiMultiplayerInteraction {
                            action: GuiMultiplayerAction::Select(server),
                            sound: None,
                        });
                }
                return Some(GuiMultiplayerInteraction {
                    action: GuiMultiplayerAction::SelectionChanged,
                    sound: None,
                });
            }
        }
        let button = self
            .GuiScreen
            .buttonList
            .iter()
            .find(|button| button.mousePressed(mouseX, mouseY))?;
        let action = match button.id {
            1 => GuiMultiplayerAction::Select(self.selectedServer()?.1),
            4 => GuiMultiplayerAction::DirectConnect,
            3 => GuiMultiplayerAction::AddServer,
            7 => {
                let (index, server) = self.selectedServer()?;
                GuiMultiplayerAction::Edit { index, server }
            }
            2 => {
                let (index, server) = self.selectedServer()?;
                GuiMultiplayerAction::Delete {
                    index,
                    serverName: server.serverName,
                }
            }
            8 => GuiMultiplayerAction::Refresh,
            0 => GuiMultiplayerAction::Cancel,
            _ => return None,
        };
        Some(GuiMultiplayerInteraction {
            action,
            sound: Some(button.playPressSound()),
        })
    }

    pub fn scroll(&mut self, lines: f32) {
        self.scrollOffset = (self.scrollOffset - (lines * SLOT_HEIGHT as f32 / 3.0) as i32).max(0);
        self.clampScroll();
    }
    pub fn moveSelection(&mut self, delta: i32) {
        if self.savedServerList.countServers() == 0 {
            self.selectedIndex = None;
            return;
        }
        let current = self
            .selectedIndex
            .map(|value| value as i32)
            .unwrap_or(if delta > 0 {
                -1
            } else {
                self.savedServerList.countServers() as i32
            });
        let next =
            (current + delta).clamp(0, self.savedServerList.countServers() as i32 - 1) as usize;
        self.selectedIndex = Some(next);
        self.ensureSelectedVisible();
        self.updateSelectionButtons();
    }

    pub fn moveSelectedServer(&mut self, delta: i32) -> std::io::Result<bool> {
        let Some(current) = self.selectedIndex else {
            return Ok(false);
        };
        let count = self.savedServerList.countServers();
        if count == 0 {
            return Ok(false);
        }
        let target = (current as i32 + delta).clamp(0, count as i32 - 1) as usize;
        if target == current {
            return Ok(false);
        }
        self.savedServerList.swapServers(current, target)?;
        self.selectedIndex = Some(target);
        self.ensureSelectedVisible();
        self.updateSelectionButtons();
        Ok(true)
    }

    fn startUnpingedServers(&mut self) {
        let pingingText = self.pingingText.clone();
        for index in 0..self.savedServerList.countServers() {
            let serverIP = {
                let Some(server) = self.savedServerList.getServerDataMut(index) else {
                    continue;
                };
                if server.pinged {
                    continue;
                }
                server.pinged = true;
                server.pingToServer = -2;
                server.serverMOTD = pingingText.clone();
                server.populationInfo.clear();
                server.serverIP.clone()
            };
            self.oldServerPinger.ping(index, serverIP);
        }
    }
    fn updateSelectionButtons(&mut self) {
        let enabled = self
            .selectedIndex
            .is_some_and(|index| index < self.savedServerList.countServers());
        for button in &mut self.GuiScreen.buttonList {
            if matches!(button.id, 1 | 2 | 7) {
                button.enabled = enabled;
            }
        }
    }
    fn clampScroll(&mut self) {
        let visible = (self.GuiScreen.height - LIST_BOTTOM_MARGIN - LIST_TOP).max(0);
        let content = self.savedServerList.countServers() as i32 * SLOT_HEIGHT;
        self.scrollOffset = self.scrollOffset.clamp(0, (content - visible).max(0));
    }
    fn ensureSelectedVisible(&mut self) {
        if let Some(index) = self.selectedIndex {
            let top = index as i32 * SLOT_HEIGHT;
            let bottom = top + SLOT_HEIGHT;
            let visible = (self.GuiScreen.height - LIST_BOTTOM_MARGIN - LIST_TOP).max(0);
            if top < self.scrollOffset {
                self.scrollOffset = top;
            } else if bottom > self.scrollOffset + visible {
                self.scrollOffset = bottom - visible;
            }
            self.clampScroll();
        }
    }
}

fn wrapText(font: &FontRenderer, text: &str, width: i32) -> Vec<String> {
    let mut output = Vec::new();
    for sourceLine in text.split('\n') {
        let mut remaining = sourceLine.to_owned();
        while !remaining.is_empty() {
            let piece = font.trim_string_to_width(&remaining, width, false);
            if piece.is_empty() {
                break;
            }
            let units = piece.encode_utf16().count();
            output.push(piece);
            let all: Vec<u16> = remaining.encode_utf16().collect();
            remaining = String::from_utf16_lossy(&all[units.min(all.len())..]);
        }
        if sourceLine.is_empty() {
            output.push(String::new());
        }
    }
    output
}
fn currentTimeMillis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as u64)
        .unwrap_or(0)
}
