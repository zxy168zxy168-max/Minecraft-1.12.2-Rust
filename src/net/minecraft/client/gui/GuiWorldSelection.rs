use std::path::{Path, PathBuf};

use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::world::chunk::storage::AnvilSaveConverter::AnvilSaveConverter;
use crate::net::minecraft::world::storage::WorldSummary::WorldSummary;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiWorldSelectionAction { Select, Create, Edit, Delete, Recreate, Cancel }
#[derive(Debug, Clone, PartialEq)]
pub struct GuiWorldSelectionInteraction { pub action: GuiWorldSelectionAction, pub sound: GuiSoundCommand }

/// Initial MCP 1.12.2 `GuiWorldSelection` + `GuiListWorldSelection` port.
#[derive(Debug, Clone)]
pub struct GuiWorldSelection {
    pub GuiScreen: GuiScreen,
    pub title: String,
    savesDirectory: PathBuf,
    worlds: Vec<WorldSummary>,
    selected: Option<usize>,
    scrollOffset: i32,
    loadError: Option<String>,
}

impl GuiWorldSelection {
    pub fn new(savesDirectory: impl AsRef<Path>) -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            title: "Select world".to_owned(),
            savesDirectory: savesDirectory.as_ref().to_path_buf(),
            worlds: Vec::new(),
            selected: None,
            scrollOffset: 0,
            loadError: None,
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32, locale: &Locale) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.title = locale.translate_key("selectWorld.title").to_owned();
        self.refreshList();
        let selected = self.selected.is_some();
        let mut select = GuiButton::newWithSize(1, width / 2 - 154, height - 52, 150, 20, locale.translate_key("selectWorld.select"));
        select.enabled = selected;
        self.GuiScreen.buttonList.push(select);
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(3, width / 2 + 4, height - 52, 150, 20, locale.translate_key("selectWorld.create")));
        for (id, x, key) in [
            (4, width / 2 - 154, "selectWorld.edit"),
            (2, width / 2 - 76, "selectWorld.delete"),
            (5, width / 2 + 4, "selectWorld.recreate"),
            (0, width / 2 + 82, "gui.cancel"),
        ] {
            let mut button = GuiButton::newWithSize(id, x, height - 28, 72, 20, locale.translate_key(key));
            if id != 0 { button.enabled = selected; }
            self.GuiScreen.buttonList.push(button);
        }
    }

    pub fn refreshList(&mut self) {
        match AnvilSaveConverter::new(&self.savesDirectory).getSaveList() {
            Ok(worlds) => {
                self.worlds = worlds;
                self.loadError = None;
                if self.selected.is_some_and(|index| index >= self.worlds.len()) { self.selected = None; }
            }
            Err(error) => {
                self.worlds.clear();
                self.selected = None;
                self.loadError = Some(error.to_string());
            }
        }
        self.clampScroll();
        self.syncButtons();
    }

    fn listTop(&self) -> i32 { 32 }
    fn listBottom(&self) -> i32 { self.GuiScreen.height - 64 }
    fn maxScroll(&self) -> i32 { (self.worlds.len() as i32 * 36 - (self.listBottom() - self.listTop())).max(0) }
    fn clampScroll(&mut self) { self.scrollOffset = self.scrollOffset.clamp(0, self.maxScroll()); }
    fn syncButtons(&mut self) {
        let selected = self.selected.is_some();
        for button in &mut self.GuiScreen.buttonList {
            if matches!(button.id, 1 | 2 | 4 | 5) { button.enabled = selected; }
        }
    }

    pub fn selectedWorld(&self) -> Option<&WorldSummary> { self.selected.and_then(|index| self.worlds.get(index)) }

    pub fn drawScreen(&mut self, drawList: &mut GuiDrawList, font: &mut FontRenderer, locale: &Locale, mouseX: i32, mouseY: i32, partialTicks: f32) {
        self.GuiScreen.drawDefaultBackground(drawList);
        self.GuiScreen.Gui.drawCenteredString(font, drawList, &self.title, self.GuiScreen.width / 2, 20, 0x00FF_FFFF);
        self.clampScroll();
        let left = self.GuiScreen.width / 2 - 110;
        let top = self.listTop();
        for (index, world) in self.worlds.iter().enumerate() {
            let y = top - self.scrollOffset + index as i32 * 36;
            if y + 36 <= top || y >= self.listBottom() { continue; }
            if self.selected == Some(index) {
                drawList.draw_rect(left - 2, y - 1, left + 222, y + 34, 0xFF80_8080_u32 as i32);
                drawList.draw_rect(left - 1, y, left + 221, y + 33, 0xFF00_0000_u32 as i32);
            }
            crate::net::minecraft::client::gui::Gui::Gui::drawModalRectWithCustomSizedTexture(
                drawList,
                ResourceLocation::parse("textures/misc/unknown_server.png"),
                left as f32, y as f32, 0.0, 0.0, 32.0, 32.0, 32.0, 32.0,
            );
            let display = if world.getDisplayName().is_empty() { format!("{} {}", locale.translate_key("selectWorld.world"), index + 1) } else { world.getDisplayName().to_owned() };
            let details = format!("{} ({})", world.getFileName(), world.getLastTimePlayed());
            let mut mode = if world.isHardcoreModeEnabled() {
                locale.translate_key("gameMode.hardcore").to_owned()
            } else {
                locale.translate_key(&format!("gameMode.{}", world.getEnumGameType().getName())).to_owned()
            };
            if world.getCheatsEnabled() { mode.push_str(&format!(", {}", locale.translate_key("selectWorld.cheats"))); }
            mode.push_str(&format!(", {} {}", locale.translate_key("selectWorld.version"), world.getVersionName()));
            self.GuiScreen.Gui.drawString(font, drawList, &display, left + 35, y + 1, 0x00FF_FFFF);
            self.GuiScreen.Gui.drawString(font, drawList, &details, left + 35, y + font.font_height + 3, 0x0080_8080);
            self.GuiScreen.Gui.drawString(font, drawList, &mode, left + 35, y + font.font_height * 2 + 3, 0x0080_8080);
        }
        if let Some(error) = &self.loadError {
            self.GuiScreen.Gui.drawCenteredString(font, drawList, error, self.GuiScreen.width / 2, 48, 0x00FF_5555);
        }
        self.GuiScreen.drawScreen(drawList, font, mouseX, mouseY, partialTicks);
    }

    pub fn mouseClicked(&mut self, mouseX: i32, mouseY: i32, mouseButton: i32) -> Option<GuiWorldSelectionInteraction> {
        if mouseButton != 0 { return None; }
        let left = self.GuiScreen.width / 2 - 110;
        if mouseX >= left && mouseX < left + 220 && mouseY >= self.listTop() && mouseY < self.listBottom() {
            let index = ((mouseY - self.listTop() + self.scrollOffset) / 36) as usize;
            if index < self.worlds.len() {
                self.selected = Some(index);
                self.syncButtons();
                return None;
            }
        }
        self.GuiScreen.buttonList.iter().find_map(|button| {
            if !button.mousePressed(mouseX, mouseY) { return None; }
            let action = match button.id {
                1 => GuiWorldSelectionAction::Select,
                3 => GuiWorldSelectionAction::Create,
                4 => GuiWorldSelectionAction::Edit,
                2 => GuiWorldSelectionAction::Delete,
                5 => GuiWorldSelectionAction::Recreate,
                0 => GuiWorldSelectionAction::Cancel,
                _ => return None,
            };
            Some(GuiWorldSelectionInteraction { action, sound: button.playPressSound() })
        })
    }

    pub fn scroll(&mut self, lines: f32) -> bool {
        if self.maxScroll() <= 0 { return false; }
        self.scrollOffset -= (lines * 18.0) as i32;
        self.clampScroll();
        true
    }
}
