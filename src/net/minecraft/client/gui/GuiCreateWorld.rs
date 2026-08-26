use std::path::{Path, PathBuf};

use rand::Rng;

use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::gui::GuiTextField::{
    GuiTextField, GuiTextFieldKey, GuiTextFieldModifiers,
};
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::world::chunk::storage::AnvilSaveConverter::AnvilSaveConverter;
use crate::net::minecraft::world::GameType::GameType;
use crate::net::minecraft::world::WorldSettings::WorldSettings;
use crate::net::minecraft::world::WorldType::WorldType;
use crate::vulkan::GuiDrawList::GuiDrawList;

const DISALLOWED_FILENAMES: [&str; 24] = [
    "CON", "COM", "PRN", "AUX", "CLOCK$", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6",
    "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];
const ILLEGAL_FILE_CHARACTERS: [char; 15] = [
    '/', '\n', '\r', '\t', '\0', '\x0c', '`', '?', '*', '\\', '<', '>', '|', '"', ':',
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldCreationRequest {
    pub saveDirName: String,
    pub worldName: String,
    pub settings: WorldSettings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GuiCreateWorldAction {
    None,
    Cancel,
    Create(WorldCreationRequest),
    CustomizeFlat,
    CustomizeWorld,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiCreateWorldInteraction {
    pub action: GuiCreateWorldAction,
    pub sound: Option<GuiSoundCommand>,
}

/// Initial source-shaped port of MCP 1.12.2 `GuiCreateWorld`.
#[derive(Debug, Clone)]
pub struct GuiCreateWorld {
    pub GuiScreen: GuiScreen,
    worldNameField: GuiTextField,
    worldSeedField: GuiTextField,
    saveDirName: String,
    gameMode: String,
    savedGameMode: Option<String>,
    generateStructuresEnabled: bool,
    allowCheats: bool,
    allowCheatsWasSetByUser: bool,
    bonusChestEnabled: bool,
    hardCoreMode: bool,
    alreadyGenerated: bool,
    inMoreWorldOptionsDisplay: bool,
    gameModeDesc1: String,
    gameModeDesc2: String,
    worldSeed: String,
    worldName: String,
    selectedWorldType: WorldType,
    pub chunkProviderSettingsJson: String,
    savesDirectory: PathBuf,
}

impl GuiCreateWorld {
    pub fn new(savesDirectory: impl AsRef<Path>, newWorldName: impl Into<String>) -> Self {
        let worldName = newWorldName.into();
        Self {
            GuiScreen: GuiScreen::default(),
            worldNameField: GuiTextField::new(9, 0, 0, 200, 20),
            worldSeedField: GuiTextField::new(10, 0, 0, 200, 20),
            saveDirName: String::new(),
            gameMode: "survival".to_owned(),
            savedGameMode: None,
            generateStructuresEnabled: true,
            allowCheats: false,
            allowCheatsWasSetByUser: false,
            bonusChestEnabled: false,
            hardCoreMode: false,
            alreadyGenerated: false,
            inMoreWorldOptionsDisplay: false,
            gameModeDesc1: String::new(),
            gameModeDesc2: String::new(),
            worldSeed: String::new(),
            worldName,
            selectedWorldType: WorldType::Default,
            chunkProviderSettingsJson: String::new(),
            savesDirectory: savesDirectory.as_ref().to_path_buf(),
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32, locale: &Locale, font: &FontRenderer) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            0,
            width / 2 - 155,
            height - 28,
            150,
            20,
            locale.translate_key("selectWorld.create"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            1,
            width / 2 + 5,
            height - 28,
            150,
            20,
            locale.translate_key("gui.cancel"),
        ));
        self.GuiScreen
            .buttonList
            .push(GuiButton::newWithSize(2, width / 2 - 75, 115, 150, 20, ""));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            3,
            width / 2 - 75,
            187,
            150,
            20,
            locale.translate_key("selectWorld.moreWorldOptions"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            4,
            width / 2 - 155,
            100,
            150,
            20,
            "",
        ));
        self.GuiScreen
            .buttonList
            .push(GuiButton::newWithSize(7, width / 2 + 5, 151, 150, 20, ""));
        self.GuiScreen
            .buttonList
            .push(GuiButton::newWithSize(5, width / 2 + 5, 100, 150, 20, ""));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            6,
            width / 2 - 155,
            151,
            150,
            20,
            "",
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            8,
            width / 2 + 5,
            120,
            150,
            20,
            locale.translate_key("selectWorld.customizeType"),
        ));

        self.worldNameField = GuiTextField::new(9, width / 2 - 100, 60, 200, 20);
        self.worldNameField.setFocused(true);
        self.worldNameField.setText(&self.worldName);
        self.worldNameField
            .setSelectionPos(self.worldNameField.getCursorPosition(), Some(font));
        self.worldSeedField = GuiTextField::new(10, width / 2 - 100, 60, 200, 20);
        self.worldSeedField.setText(&self.worldSeed);
        self.worldSeedField
            .setSelectionPos(self.worldSeedField.getCursorPosition(), Some(font));
        self.showMoreWorldOptions(self.inMoreWorldOptionsDisplay, locale);
        self.calcSaveDirName();
        self.updateDisplayState(locale);
        self.updateCreateButton();
    }

    pub fn updateScreen(&mut self) {
        self.worldNameField.updateCursorCounter();
        self.worldSeedField.updateCursorCounter();
    }

    fn buttonMut(&mut self, id: i32) -> Option<&mut GuiButton> {
        self.GuiScreen
            .buttonList
            .iter_mut()
            .find(|button| button.id == id)
    }

    fn buttonVisible(&self, id: i32) -> bool {
        self.GuiScreen
            .buttonList
            .iter()
            .find(|button| button.id == id)
            .is_some_and(|button| button.visible)
    }

    fn updateCreateButton(&mut self) {
        let enabled = !self.worldNameField.getText().is_empty() && !self.alreadyGenerated;
        if let Some(button) = self.buttonMut(0) {
            button.enabled = enabled;
        }
    }

    fn calcSaveDirName(&mut self) {
        let mut name = self.worldNameField.getText().trim().to_owned();
        for ch in ILLEGAL_FILE_CHARACTERS {
            name = name.replace(ch, "_");
        }
        if name.is_empty() {
            name = "World".to_owned();
        }
        name = getUncollidingSaveDirName(&self.savesDirectory, &name);
        self.saveDirName = name;
    }

    fn updateDisplayState(&mut self, locale: &Locale) {
        let gameMode = self.gameMode.clone();
        if let Some(button) = self.buttonMut(2) {
            button.displayString = format!(
                "{}: {}",
                locale.translate_key("selectWorld.gameMode"),
                locale.translate_key(&format!("selectWorld.gameMode.{gameMode}"))
            );
        }
        self.gameModeDesc1 = locale
            .translate_key(&format!("selectWorld.gameMode.{gameMode}.line1"))
            .to_owned();
        self.gameModeDesc2 = locale
            .translate_key(&format!("selectWorld.gameMode.{gameMode}.line2"))
            .to_owned();
        let structures = self.generateStructuresEnabled;
        if let Some(button) = self.buttonMut(4) {
            button.displayString = format!(
                "{} {}",
                locale.translate_key("selectWorld.mapFeatures"),
                locale.translate_key(if structures {
                    "options.on"
                } else {
                    "options.off"
                })
            );
        }
        let bonus = self.bonusChestEnabled && !self.hardCoreMode;
        if let Some(button) = self.buttonMut(7) {
            button.displayString = format!(
                "{} {}",
                locale.translate_key("selectWorld.bonusItems"),
                locale.translate_key(if bonus { "options.on" } else { "options.off" })
            );
        }
        let worldType = self.selectedWorldType;
        if let Some(button) = self.buttonMut(5) {
            button.displayString = format!(
                "{} {}",
                locale.translate_key("selectWorld.mapType"),
                locale.translate_key(worldType.getTranslateName())
            );
        }
        let cheats = self.allowCheats && !self.hardCoreMode;
        if let Some(button) = self.buttonMut(6) {
            button.displayString = format!(
                "{} {}",
                locale.translate_key("selectWorld.allowCommands"),
                locale.translate_key(if cheats { "options.on" } else { "options.off" })
            );
        }
    }

    fn showMoreWorldOptions(&mut self, toggle: bool, locale: &Locale) {
        self.inMoreWorldOptionsDisplay = toggle;
        let debug = self.selectedWorldType == WorldType::DebugWorld;
        if debug {
            if let Some(button) = self.buttonMut(2) {
                button.visible = !toggle;
                button.enabled = false;
            }
            if self.savedGameMode.is_none() {
                self.savedGameMode = Some(self.gameMode.clone());
            }
            self.gameMode = "spectator".to_owned();
            for id in [4, 7, 6, 8] {
                if let Some(button) = self.buttonMut(id) {
                    button.visible = false;
                }
            }
            if let Some(button) = self.buttonMut(5) {
                button.visible = toggle;
            }
        } else {
            if let Some(button) = self.buttonMut(2) {
                button.visible = !toggle;
                button.enabled = true;
            }
            if let Some(saved) = self.savedGameMode.take() {
                self.gameMode = saved;
            }
            // Snapshot the world type before borrowing a button mutably.
            // This preserves MCP GuiCreateWorld's visibility state machine
            // while avoiding overlapping `&mut self` / `&self` borrows.
            let selectedWorldType = self.selectedWorldType;
            let mapFeaturesVisible = toggle && selectedWorldType != WorldType::Customized;
            let customizeVisible =
                toggle && matches!(selectedWorldType, WorldType::Flat | WorldType::Customized);
            if let Some(button) = self.buttonMut(4) {
                button.visible = mapFeaturesVisible;
            }
            if let Some(button) = self.buttonMut(7) {
                button.visible = toggle;
            }
            if let Some(button) = self.buttonMut(5) {
                button.visible = toggle;
            }
            if let Some(button) = self.buttonMut(6) {
                button.visible = toggle;
            }
            if let Some(button) = self.buttonMut(8) {
                button.visible = customizeVisible;
            }
        }
        if let Some(button) = self.buttonMut(3) {
            button.displayString = locale
                .translate_key(if toggle {
                    "gui.done"
                } else {
                    "selectWorld.moreWorldOptions"
                })
                .to_owned();
        }
        self.updateDisplayState(locale);
    }

    fn cycleGameMode(&mut self, locale: &Locale) {
        match self.gameMode.as_str() {
            "survival" => {
                if !self.allowCheatsWasSetByUser {
                    self.allowCheats = false;
                }
                self.gameMode = "hardcore".to_owned();
                self.hardCoreMode = true;
                if let Some(button) = self.buttonMut(6) {
                    button.enabled = false;
                }
                if let Some(button) = self.buttonMut(7) {
                    button.enabled = false;
                }
            }
            "hardcore" => {
                if !self.allowCheatsWasSetByUser {
                    self.allowCheats = true;
                }
                self.hardCoreMode = false;
                self.gameMode = "creative".to_owned();
                if let Some(button) = self.buttonMut(6) {
                    button.enabled = true;
                }
                if let Some(button) = self.buttonMut(7) {
                    button.enabled = true;
                }
            }
            _ => {
                if !self.allowCheatsWasSetByUser {
                    self.allowCheats = false;
                }
                self.gameMode = "survival".to_owned();
                self.hardCoreMode = false;
                if let Some(button) = self.buttonMut(6) {
                    button.enabled = true;
                }
                if let Some(button) = self.buttonMut(7) {
                    button.enabled = true;
                }
            }
        }
        self.updateDisplayState(locale);
    }

    fn cycleWorldType(&mut self, shiftDown: bool, locale: &Locale) {
        let list = WorldType::CREATABLE;
        let mut index = list
            .iter()
            .position(|value| *value == self.selectedWorldType)
            .unwrap_or(0);
        loop {
            index = (index + 1) % list.len();
            let candidate = list[index];
            if candidate.getCanBeCreated() && (candidate != WorldType::DebugWorld || shiftDown) {
                self.selectedWorldType = candidate;
                break;
            }
        }
        self.chunkProviderSettingsJson.clear();
        self.updateDisplayState(locale);
        self.showMoreWorldOptions(self.inMoreWorldOptionsDisplay, locale);
    }

    fn buildCreationRequest(&mut self) -> Option<WorldCreationRequest> {
        if self.alreadyGenerated || self.worldNameField.getText().is_empty() {
            return None;
        }
        self.alreadyGenerated = true;
        self.updateCreateButton();
        let text = self.worldSeedField.getText();
        let seed = if text.is_empty() {
            rand::thread_rng().gen::<i64>()
        } else if let Ok(value) = text.parse::<i64>() {
            if value != 0 {
                value
            } else {
                rand::thread_rng().gen::<i64>()
            }
        } else {
            java_string_hash(&text) as i64
        };
        let mut settings = WorldSettings::new(
            seed,
            GameType::getByName(&self.gameMode),
            self.generateStructuresEnabled,
            self.hardCoreMode,
            self.selectedWorldType,
        )
        .setGeneratorOptions(self.chunkProviderSettingsJson.clone());
        if self.bonusChestEnabled && !self.hardCoreMode {
            settings = settings.enableBonusChest();
        }
        if self.allowCheats && !self.hardCoreMode {
            settings = settings.enableCommands();
        }
        Some(WorldCreationRequest {
            saveDirName: self.saveDirName.clone(),
            worldName: self.worldNameField.getText().trim().to_owned(),
            settings,
        })
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
        font: &FontRenderer,
        locale: &Locale,
        shiftDown: bool,
    ) -> Option<GuiCreateWorldInteraction> {
        if self.inMoreWorldOptionsDisplay {
            self.worldSeedField
                .mouseClicked(mouseX, mouseY, mouseButton, font);
        } else {
            self.worldNameField
                .mouseClicked(mouseX, mouseY, mouseButton, font);
        }
        if mouseButton != 0 {
            return None;
        }
        let index = self
            .GuiScreen
            .buttonList
            .iter()
            .position(|button| button.mousePressed(mouseX, mouseY))?;
        let id = self.GuiScreen.buttonList[index].id;
        let sound = Some(self.GuiScreen.buttonList[index].playPressSound());
        let action = match id {
            1 => GuiCreateWorldAction::Cancel,
            0 => self
                .buildCreationRequest()
                .map(GuiCreateWorldAction::Create)
                .unwrap_or(GuiCreateWorldAction::None),
            3 => {
                self.showMoreWorldOptions(!self.inMoreWorldOptionsDisplay, locale);
                GuiCreateWorldAction::None
            }
            2 => {
                self.cycleGameMode(locale);
                GuiCreateWorldAction::None
            }
            4 => {
                self.generateStructuresEnabled = !self.generateStructuresEnabled;
                self.updateDisplayState(locale);
                GuiCreateWorldAction::None
            }
            7 => {
                self.bonusChestEnabled = !self.bonusChestEnabled;
                self.updateDisplayState(locale);
                GuiCreateWorldAction::None
            }
            5 => {
                self.cycleWorldType(shiftDown, locale);
                GuiCreateWorldAction::None
            }
            6 => {
                self.allowCheatsWasSetByUser = true;
                self.allowCheats = !self.allowCheats;
                self.updateDisplayState(locale);
                GuiCreateWorldAction::None
            }
            8 if self.selectedWorldType == WorldType::Flat => GuiCreateWorldAction::CustomizeFlat,
            8 if self.selectedWorldType == WorldType::Customized => {
                GuiCreateWorldAction::CustomizeWorld
            }
            _ => GuiCreateWorldAction::None,
        };
        Some(GuiCreateWorldInteraction { action, sound })
    }

    pub fn typedText(&mut self, text: &str, font: &FontRenderer) -> bool {
        let changed = if self.inMoreWorldOptionsDisplay && self.worldSeedField.isFocused() {
            self.worldSeedField.writeText(text, Some(font))
        } else if !self.inMoreWorldOptionsDisplay && self.worldNameField.isFocused() {
            self.worldNameField.writeText(text, Some(font))
        } else {
            false
        };
        if changed {
            self.worldSeed = self.worldSeedField.getText();
            self.worldName = self.worldNameField.getText();
            self.calcSaveDirName();
            self.updateCreateButton();
        }
        changed
    }

    pub fn keyPressed(
        &mut self,
        key: GuiTextFieldKey,
        modifiers: GuiTextFieldModifiers,
        font: &FontRenderer,
    ) -> bool {
        let changed = if self.inMoreWorldOptionsDisplay && self.worldSeedField.isFocused() {
            self.worldSeedField.keyPressed(key, modifiers, font)
        } else if !self.inMoreWorldOptionsDisplay && self.worldNameField.isFocused() {
            self.worldNameField.keyPressed(key, modifiers, font)
        } else {
            false
        };
        if changed {
            self.worldSeed = self.worldSeedField.getText();
            self.worldName = self.worldNameField.getText();
            self.calcSaveDirName();
            self.updateCreateButton();
        }
        changed
    }

    pub fn selectAll(&mut self, font: &FontRenderer) -> bool {
        if self.inMoreWorldOptionsDisplay && self.worldSeedField.isFocused() {
            self.worldSeedField.selectAll(font);
            true
        } else if !self.inMoreWorldOptionsDisplay && self.worldNameField.isFocused() {
            self.worldNameField.selectAll(font);
            true
        } else {
            false
        }
    }

    pub fn enterPressed(&mut self) -> Option<WorldCreationRequest> {
        self.buildCreationRequest()
    }

    pub fn drawScreen(
        &mut self,
        drawList: &mut GuiDrawList,
        font: &mut FontRenderer,
        locale: &Locale,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.GuiScreen.drawDefaultBackground(drawList);
        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            locale.translate_key("selectWorld.create"),
            self.GuiScreen.width / 2,
            20,
            0x00FF_FFFF,
        );
        if self.inMoreWorldOptionsDisplay {
            self.GuiScreen.Gui.drawString(
                font,
                drawList,
                locale.translate_key("selectWorld.enterSeed"),
                self.GuiScreen.width / 2 - 100,
                47,
                10_526_880,
            );
            self.GuiScreen.Gui.drawString(
                font,
                drawList,
                locale.translate_key("selectWorld.seedInfo"),
                self.GuiScreen.width / 2 - 100,
                85,
                10_526_880,
            );
            if self.buttonVisible(4) {
                self.GuiScreen.Gui.drawString(
                    font,
                    drawList,
                    locale.translate_key("selectWorld.mapFeatures.info"),
                    self.GuiScreen.width / 2 - 150,
                    122,
                    10_526_880,
                );
            }
            if self.buttonVisible(6) {
                self.GuiScreen.Gui.drawString(
                    font,
                    drawList,
                    locale.translate_key("selectWorld.allowCommands.info"),
                    self.GuiScreen.width / 2 - 150,
                    172,
                    10_526_880,
                );
            }
            self.worldSeedField.drawTextBox(drawList, font);
            if self.selectedWorldType.showWorldInfoNotice() {
                self.GuiScreen.Gui.drawString(
                    font,
                    drawList,
                    locale.translate_key(self.selectedWorldType.getTranslatedInfo()),
                    self.GuiScreen.width / 2 + 7,
                    122,
                    10_526_880,
                );
            }
        } else {
            self.GuiScreen.Gui.drawString(
                font,
                drawList,
                locale.translate_key("selectWorld.enterName"),
                self.GuiScreen.width / 2 - 100,
                47,
                10_526_880,
            );
            self.GuiScreen.Gui.drawString(
                font,
                drawList,
                &format!(
                    "{} {}",
                    locale.translate_key("selectWorld.resultFolder"),
                    self.saveDirName
                ),
                self.GuiScreen.width / 2 - 100,
                85,
                10_526_880,
            );
            self.worldNameField.drawTextBox(drawList, font);
            self.GuiScreen.Gui.drawString(
                font,
                drawList,
                &self.gameModeDesc1,
                self.GuiScreen.width / 2 - 100,
                137,
                10_526_880,
            );
            self.GuiScreen.Gui.drawString(
                font,
                drawList,
                &self.gameModeDesc2,
                self.GuiScreen.width / 2 - 100,
                149,
                10_526_880,
            );
        }
        self.GuiScreen
            .drawScreen(drawList, font, mouseX, mouseY, partialTicks);
    }
}

pub fn getUncollidingSaveDirName(savesDirectory: &Path, requested: &str) -> String {
    let mut name = requested
        .replace('.', "_")
        .replace('/', "_")
        .replace('"', "_");
    if DISALLOWED_FILENAMES
        .iter()
        .any(|value| name.eq_ignore_ascii_case(value))
    {
        name = format!("_{name}_");
    }
    let saveFormat = AnvilSaveConverter::new(savesDirectory);
    while saveFormat.getWorldInfo(&name).ok().flatten().is_some()
        || savesDirectory.join(&name).exists()
    {
        name.push('-');
    }
    name
}

fn java_string_hash(value: &str) -> i32 {
    value.encode_utf16().fold(0_i32, |hash, unit| {
        hash.wrapping_mul(31).wrapping_add(unit as i32)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_seed_hash_matches_string_hash_code() {
        assert_eq!(java_string_hash("abc"), 96354);
    }

    #[test]
    fn reserved_windows_names_are_wrapped_like_mcp() {
        let root = std::env::temp_dir().join("mc1122-create-world-name-test");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        assert_eq!(getUncollidingSaveDirName(&root, "CON"), "_CON_");
        std::fs::create_dir(root.join("World")).unwrap();
        assert_eq!(getUncollidingSaveDirName(&root, "World"), "World-");
        let _ = std::fs::remove_dir_all(&root);
    }
}
