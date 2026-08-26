use std::path::PathBuf;

use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::optifine::shader::gui::GuiShaderOptions::{
    GuiShaderOptions, GuiShaderOptionsAction,
};
use crate::net::optifine::shader::Shaders::{
    handDepthName, packNameDefault, packNameNone, qualityName, ShaderPackEntry, Shaders,
};
use crate::vulkan::GuiDrawList::GuiDrawList;

const SHADERS_FOLDER_ID: i32 = 201;
const DONE_ID: i32 = 202;
const SHADER_OPTIONS_ID: i32 = 203;
const RIGHT_WIDTH: i32 = 120;
const LIST_TOP: i32 = 30;
const LIST_BOTTOM_MARGIN: i32 = 50;
const SLOT_HEIGHT: i32 = 16;
const CONTENT_SLOT_HEIGHT: i32 = 18;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnumShaderOption {
    Antialiasing,
    NormalMap,
    SpecularMap,
    RenderResMul,
    ShadowResMul,
    HandDepthMul,
    OldHandLight,
    OldLighting,
}

impl EnumShaderOption {
    const ALL: [Self; 8] = [
        Self::Antialiasing,
        Self::NormalMap,
        Self::SpecularMap,
        Self::RenderResMul,
        Self::ShadowResMul,
        Self::HandDepthMul,
        Self::OldHandLight,
        Self::OldLighting,
    ];

    const fn id(self) -> i32 {
        match self {
            Self::Antialiasing => 0,
            Self::NormalMap => 1,
            Self::SpecularMap => 2,
            Self::RenderResMul => 3,
            Self::ShadowResMul => 4,
            Self::HandDepthMul => 5,
            Self::OldHandLight => 7,
            Self::OldLighting => 8,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Antialiasing => "Antialiasing",
            Self::NormalMap => "Normal Map",
            Self::SpecularMap => "Specular Map",
            Self::RenderResMul => "Render Quality",
            Self::ShadowResMul => "Shadow Quality",
            Self::HandDepthMul => "Hand Depth",
            Self::OldHandLight => "Old Hand Light",
            Self::OldLighting => "Old Lighting",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiShaderAction {
    None,
    SelectShaderPack(String),
    ReloadShaderPack,
    OpenShaderPacksFolder,
    Done,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiShaderInteraction {
    pub action: GuiShaderAction,
    pub sound: GuiSoundCommand,
}

/// Semantic port of OptiFine 1.12.2 `GuiShader` and `GuiSlotShaders`.
///
/// The pack list remains metadata-only. The eight original right-hand controls,
/// their positions, values, enable rules and cycling behavior are kept separate
/// from the pack-specific `GuiShaderOptions` screen.
#[derive(Debug, Clone)]
pub struct GuiShader {
    pub GuiScreen: GuiScreen,
    pub screenTitle: String,
    shaders: Shaders,
    shaderslist: Vec<ShaderPackEntry>,
    selectedIndex: usize,
    amountScrolled: f32,
    listWidth: i32,
    mouseX: i32,
    mouseY: i32,
    rendererDescription: String,
    updateTimer: i32,
    enumButtons: Vec<(EnumShaderOption, GuiButton)>,
    shaderPacksFolderButton: GuiButton,
    doneButton: GuiButton,
    shaderOptionsButton: GuiButton,
    shaderOptions: Option<GuiShaderOptions>,
    language: String,
    advancedTooltips: bool,
}

impl GuiShader {
    pub fn new(gameDir: PathBuf, rendererDescription: impl Into<String>) -> Self {
        Self::newWithSettings(gameDir, rendererDescription, "en_US", false)
    }

    pub fn newWithSettings(
        gameDir: PathBuf,
        rendererDescription: impl Into<String>,
        language: impl Into<String>,
        advancedTooltips: bool,
    ) -> Self {
        let shaders = Shaders::loadConfig(gameDir);
        let shaderslist = shaders.listOfShaders();
        let selectedIndex = shaders.selectedIndex(&shaderslist);
        let enumButtons = EnumShaderOption::ALL
            .into_iter()
            .map(|option| {
                (
                    option,
                    GuiButton::newWithSize(option.id(), 0, 0, RIGHT_WIDTH, 20, ""),
                )
            })
            .collect();
        Self {
            GuiScreen: GuiScreen::default(),
            screenTitle: "Shaders".to_owned(),
            shaders,
            shaderslist,
            selectedIndex,
            amountScrolled: 0.0,
            listWidth: 0,
            mouseX: 0,
            mouseY: 0,
            rendererDescription: rendererDescription.into(),
            updateTimer: -1,
            enumButtons,
            shaderPacksFolderButton: GuiButton::newWithSize(
                SHADERS_FOLDER_ID,
                0,
                0,
                150,
                20,
                "Shaders Folder",
            ),
            doneButton: GuiButton::newWithSize(DONE_ID, 0, 0, 150, 20, "Done"),
            shaderOptionsButton: GuiButton::newWithSize(
                SHADER_OPTIONS_ID,
                0,
                0,
                RIGHT_WIDTH,
                20,
                "Shader Options...",
            ),
            shaderOptions: None,
            language: language.into(),
            advancedTooltips,
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        if let Some(options) = self.shaderOptions.as_mut() {
            options.initGui(width, height);
            return;
        }

        let controlsX = width - RIGHT_WIDTH - 10;
        self.listWidth = width - RIGHT_WIDTH - 20;
        for (row, (_, button)) in self.enumButtons.iter_mut().enumerate() {
            button.x = controlsX;
            button.y = LIST_TOP + row as i32 * 20;
            button.setWidth(RIGHT_WIDTH);
        }
        let buttonWidth = (self.listWidth / 2 - 10).min(150);
        self.shaderPacksFolderButton.x = self.listWidth / 4 - buttonWidth / 2;
        self.shaderPacksFolderButton.y = height - 25;
        self.shaderPacksFolderButton.setWidth(buttonWidth);
        self.doneButton.x = self.listWidth / 4 * 3 - buttonWidth / 2;
        self.doneButton.y = height - 25;
        self.doneButton.setWidth(buttonWidth);
        self.shaderOptionsButton.x = controlsX;
        self.shaderOptionsButton.y = height - 25;
        self.refreshList();
        self.centerSelection();
        self.updateButtons();
        self.updateButtonText();
    }

    pub fn drawScreen(
        &mut self,
        drawList: &mut GuiDrawList,
        fontRendererObj: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.mouseX = mouseX;
        self.mouseY = mouseY;
        self.GuiScreen.drawDefaultBackground(drawList);
        self.drawContents(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
    }

    pub fn drawScreenInWorld(
        &mut self,
        drawList: &mut GuiDrawList,
        fontRendererObj: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.mouseX = mouseX;
        self.mouseY = mouseY;
        self.GuiScreen.drawDefaultBackgroundInWorld(drawList);
        self.drawContents(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
        shiftDown: bool,
    ) -> Option<GuiShaderInteraction> {
        if self.shaderOptions.is_some() {
            let interaction = {
                let options = self.shaderOptions.as_mut()?;
                options.mouseClicked(mouseX, mouseY, mouseButton, shiftDown)?
            };
            let close = matches!(interaction.action, GuiShaderOptionsAction::Close { .. });
            let action = match interaction.action {
                GuiShaderOptionsAction::None => GuiShaderAction::None,
                GuiShaderOptionsAction::Reload => GuiShaderAction::ReloadShaderPack,
                GuiShaderOptionsAction::Close { reload } => {
                    if reload {
                        GuiShaderAction::ReloadShaderPack
                    } else {
                        GuiShaderAction::None
                    }
                }
            };
            if close {
                self.shaderOptions = None;
                self.initGui(self.GuiScreen.width, self.GuiScreen.height);
            }
            return Some(GuiShaderInteraction {
                action,
                sound: interaction.sound,
            });
        }

        if mouseButton != 0 {
            return None;
        }
        if let Some(index) = self
            .enumButtons
            .iter()
            .position(|(_, button)| button.mousePressed(mouseX, mouseY))
        {
            let sound = self.enumButtons[index].1.playPressSound();
            let option = self.enumButtons[index].0;
            self.changeEnumOption(option, shiftDown);
            if let Err(error) = self.shaders.storeConfig() {
                log::error!("Couldn't save OptiFine shader configuration: {error}");
            }
            self.updateButtonText();
            return Some(GuiShaderInteraction {
                action: GuiShaderAction::ReloadShaderPack,
                sound,
            });
        }
        if self.shaderPacksFolderButton.mousePressed(mouseX, mouseY) {
            return Some(GuiShaderInteraction {
                action: GuiShaderAction::OpenShaderPacksFolder,
                sound: self.shaderPacksFolderButton.playPressSound(),
            });
        }
        if self.doneButton.mousePressed(mouseX, mouseY) {
            if let Err(error) = self.shaders.storeConfig() {
                log::error!("Couldn't save OptiFine shader configuration: {error}");
            }
            return Some(GuiShaderInteraction {
                action: GuiShaderAction::Done,
                sound: self.doneButton.playPressSound(),
            });
        }
        if self.shaderOptionsButton.mousePressed(mouseX, mouseY) {
            let sound = self.shaderOptionsButton.playPressSound();
            self.openShaderOptions();
            return Some(GuiShaderInteraction {
                action: GuiShaderAction::None,
                sound,
            });
        }
        if mouseX < 0
            || mouseX >= self.listWidth
            || mouseY < LIST_TOP
            || mouseY >= self.listBottom()
        {
            return None;
        }
        let contentY = mouseY - LIST_TOP + self.amountScrolled as i32;
        let index = contentY.div_euclid(CONTENT_SLOT_HEIGHT) as usize;
        if index >= self.shaderslist.len() || index == self.selectedIndex {
            return None;
        }
        self.selectedIndex = index;
        let name = self.shaderslist[index].name.clone();
        self.shaders.currentshadername = name.clone();
        self.updateButtons();
        Some(GuiShaderInteraction {
            action: GuiShaderAction::SelectShaderPack(name),
            sound: self.doneButton.playPressSound(),
        })
    }

    pub fn updateScreen(&mut self) -> bool {
        if self.shaderOptions.is_some() {
            return false;
        }
        self.updateTimer -= 1;
        if self.updateTimer > 0 {
            return false;
        }
        let before = self
            .shaderslist
            .iter()
            .map(|entry| entry.name.clone())
            .collect::<Vec<_>>();
        self.refreshList();
        self.updateTimer += 20;
        before
            != self
                .shaderslist
                .iter()
                .map(|entry| entry.name.clone())
                .collect::<Vec<_>>()
    }

    pub fn scroll(&mut self, lines: f32) -> bool {
        if lines == 0.0 || self.shaderOptions.is_some() || self.maxScroll() <= 0.0 {
            return false;
        }
        let previous = self.amountScrolled;
        self.amountScrolled += if lines > 0.0 {
            -(SLOT_HEIGHT as f32 / 2.0)
        } else {
            SLOT_HEIGHT as f32 / 2.0
        };
        self.amountScrolled = self.amountScrolled.clamp(0.0, self.maxScroll());
        self.amountScrolled != previous
    }

    pub fn refreshList(&mut self) {
        let selected = self.shaders.currentshadername.clone();
        self.shaderslist = self.shaders.listOfShaders();
        self.selectedIndex = self
            .shaderslist
            .iter()
            .position(|entry| entry.name == selected)
            .unwrap_or(0);
        self.amountScrolled = self.amountScrolled.clamp(0.0, self.maxScroll());
    }

    pub fn shaderpacksDir(&self) -> PathBuf {
        self.shaders.shaderpacksdir.clone()
    }
    pub fn selectedName(&self) -> &str {
        &self.shaders.currentshadername
    }
    pub fn isOptionsView(&self) -> bool {
        self.shaderOptions.is_some()
    }

    /// Escape/on-close behavior of OptiFine `GuiShaderOptions#onGuiClosed`.
    pub fn closeOptionsView(&mut self) -> bool {
        let action = {
            let Some(options) = self.shaderOptions.as_mut() else {
                return false;
            };
            options.close()
        };
        match action {
            GuiShaderOptionsAction::None => false,
            GuiShaderOptionsAction::Reload => true,
            GuiShaderOptionsAction::Close { reload } => {
                self.shaderOptions = None;
                self.initGui(self.GuiScreen.width, self.GuiScreen.height);
                reload
            }
        }
    }

    pub fn mouseDragged(&mut self, mouseX: i32) -> bool {
        self.shaderOptions
            .as_mut()
            .is_some_and(|options| options.mouseDragged(mouseX))
    }

    pub fn mouseReleased(&mut self) {
        if let Some(options) = self.shaderOptions.as_mut() {
            options.mouseReleased();
        }
    }

    fn drawContents(
        &mut self,
        drawList: &mut GuiDrawList,
        fontRendererObj: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        if let Some(options) = self.shaderOptions.as_mut() {
            options.drawScreen(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
            return;
        }

        self.drawShaderList(drawList, fontRendererObj);
        self.GuiScreen.Gui.drawCenteredString(
            fontRendererObj,
            drawList,
            &format!("{} ", self.screenTitle),
            self.GuiScreen.width / 2,
            15,
            0x00FF_FFFF,
        );
        let glText = format!("OpenGL: {}", self.rendererDescription);
        if fontRendererObj.get_string_width(&glText) < self.GuiScreen.width - 5 {
            fontRendererObj.draw_centered_string_with_shadow(
                drawList,
                &glText,
                self.GuiScreen.width / 2,
                self.GuiScreen.height - 40,
                0x0080_8080,
            );
        } else {
            fontRendererObj.draw_string_with_shadow(
                drawList,
                &glText,
                5.0,
                (self.GuiScreen.height - 40) as f32,
                0x0080_8080,
            );
        }
        for (_, button) in &mut self.enumButtons {
            button.drawButton(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
        }
        self.shaderPacksFolderButton.drawButton(
            drawList,
            fontRendererObj,
            mouseX,
            mouseY,
            partialTicks,
        );
        self.doneButton
            .drawButton(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
        self.shaderOptionsButton.drawButton(
            drawList,
            fontRendererObj,
            mouseX,
            mouseY,
            partialTicks,
        );
    }

    fn updateButtons(&mut self) {
        let active = !self.shaders.currentshadername.is_empty()
            && self.shaders.currentshadername != packNameNone;
        for (option, button) in &mut self.enumButtons {
            button.enabled = *option == EnumShaderOption::Antialiasing || active;
        }
        self.shaderOptionsButton.enabled = active;
    }

    fn updateButtonText(&mut self) {
        for (option, button) in &mut self.enumButtons {
            let value = match option {
                EnumShaderOption::Antialiasing => match self.shaders.configAntialiasingLevel {
                    2 => "FXAA 2x".to_owned(),
                    4 => "FXAA 4x".to_owned(),
                    _ => "OFF".to_owned(),
                },
                EnumShaderOption::NormalMap => onOff(self.shaders.configNormalMap).to_owned(),
                EnumShaderOption::SpecularMap => onOff(self.shaders.configSpecularMap).to_owned(),
                EnumShaderOption::RenderResMul => {
                    qualityName(self.shaders.configRenderResMul).to_owned()
                }
                EnumShaderOption::ShadowResMul => {
                    qualityName(self.shaders.configShadowResMul).to_owned()
                }
                EnumShaderOption::HandDepthMul => {
                    handDepthName(self.shaders.configHandDepthMul).to_owned()
                }
                EnumShaderOption::OldHandLight => {
                    self.shaders.configOldHandLight.userValue().to_owned()
                }
                EnumShaderOption::OldLighting => {
                    self.shaders.configOldLighting.userValue().to_owned()
                }
            };
            button.displayString = format!("{}: {value}", option.label());
        }
    }

    fn changeEnumOption(&mut self, option: EnumShaderOption, previous: bool) {
        match option {
            EnumShaderOption::Antialiasing => self.shaders.nextAntialiasingLevel(),
            EnumShaderOption::NormalMap => {
                self.shaders.configNormalMap = !self.shaders.configNormalMap
            }
            EnumShaderOption::SpecularMap => {
                self.shaders.configSpecularMap = !self.shaders.configSpecularMap;
            }
            EnumShaderOption::RenderResMul => self.shaders.cycleRenderQuality(previous),
            EnumShaderOption::ShadowResMul => self.shaders.cycleShadowQuality(previous),
            EnumShaderOption::HandDepthMul => self.shaders.cycleHandDepth(previous),
            EnumShaderOption::OldHandLight => self.shaders.configOldHandLight.nextValue(),
            EnumShaderOption::OldLighting => self.shaders.configOldLighting.nextValue(),
        }
    }

    fn openShaderOptions(&mut self) {
        if !self.shaderOptionsButton.enabled {
            return;
        }
        let mut pack = self.shaders.loadShaderPack(None);
        if pack.getName() == packNameNone {
            pack.close();
            return;
        }
        match GuiShaderOptions::load(
            &self.shaders.gameDir,
            &mut *pack,
            &self.language,
            self.advancedTooltips,
        ) {
            Ok(mut options) => {
                options.initGui(self.GuiScreen.width, self.GuiScreen.height);
                self.shaderOptions = Some(options);
            }
            Err(error) => {
                log::error!(
                    "Couldn't load shader options for {}: {error}",
                    self.shaders.currentshadername,
                );
            }
        }
        pack.close();
    }

    fn listBottom(&self) -> i32 {
        self.GuiScreen.height - LIST_BOTTOM_MARGIN
    }
    fn contentHeight(&self) -> i32 {
        self.shaderslist.len() as i32 * CONTENT_SLOT_HEIGHT
    }
    fn maxScroll(&self) -> f32 {
        (self.contentHeight() - (self.listBottom() - LIST_TOP)).max(0) as f32
    }

    fn centerSelection(&mut self) {
        let selectedY = self.selectedIndex as i32 * SLOT_HEIGHT;
        let halfViewport = (self.listBottom() - LIST_TOP) / 2;
        if selectedY > halfViewport {
            self.amountScrolled = (selectedY - halfViewport) as f32;
        }
        self.amountScrolled = self.amountScrolled.clamp(0.0, self.maxScroll());
    }

    fn drawShaderList(&self, drawList: &mut GuiDrawList, fontRendererObj: &mut FontRenderer) {
        let bottom = self.listBottom();
        drawList.draw_rect(0, LIST_TOP, self.listWidth, bottom, 0x8000_0000_u32 as i32);
        let firstY = LIST_TOP - self.amountScrolled as i32;
        for (index, entry) in self.shaderslist.iter().enumerate() {
            let y = firstY + index as i32 * CONTENT_SLOT_HEIGHT;
            if y + SLOT_HEIGHT < LIST_TOP || y >= bottom {
                continue;
            }
            if index == self.selectedIndex {
                drawList.draw_rect(
                    2,
                    y,
                    self.listWidth - 2,
                    y + SLOT_HEIGHT,
                    0xFF80_8080_u32 as i32,
                );
                drawList.draw_rect(
                    3,
                    y + 1,
                    self.listWidth - 3,
                    y + SLOT_HEIGHT - 1,
                    0xFF00_0000_u32 as i32,
                );
            }
            let name = if entry.name == packNameNone {
                "OFF"
            } else if entry.name == packNameDefault {
                "(internal)"
            } else {
                entry.name.as_str()
            };
            fontRendererObj.draw_centered_string_with_shadow(
                drawList,
                name,
                self.listWidth / 2,
                y + 1,
                0x00FF_FFFF,
            );
        }
        if let Some((x, y, height)) = self.scrollbarGeometry() {
            drawList.draw_rect(x, LIST_TOP, x + 6, bottom, 0xFF00_0000_u32 as i32);
            drawList.draw_rect(x, y, x + 6, y + height, 0xFF80_8080_u32 as i32);
            drawList.draw_rect(x, y, x + 5, y + height - 1, 0xFFC0_C0C0_u32 as i32);
        }
    }

    fn scrollbarGeometry(&self) -> Option<(i32, i32, i32)> {
        let maxScroll = self.maxScroll();
        if maxScroll <= 0.0 {
            return None;
        }
        let viewport = self.listBottom() - LIST_TOP;
        let content = self.contentHeight().max(1);
        let height = (viewport * viewport / content).clamp(32, viewport - 8);
        let travel = viewport - height;
        let y = LIST_TOP + ((self.amountScrolled / maxScroll) * travel as f32) as i32;
        Some((self.listWidth - 6, y, height))
    }
}

const fn onOff(value: bool) -> &'static str {
    if value {
        "ON"
    } else {
        "OFF"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_pack_list_does_not_construct_shader_options() {
        let game = std::env::temp_dir().join("mc112-gui-shader-lazy-options");
        let screen = GuiShader::new(game, "test");
        assert!(screen.shaderOptions.is_none());
    }

    #[test]
    fn main_screen_has_the_eight_original_optifine_controls() {
        let game = std::env::temp_dir().join("mc112-gui-shader-eight-controls");
        let mut screen = GuiShader::new(game, "test");
        screen.initGui(854, 480);
        assert_eq!(screen.enumButtons.len(), 8);
        assert_eq!(screen.enumButtons[0].1.y, 30);
        assert_eq!(screen.enumButtons[7].1.y, 170);
        assert_eq!(screen.shaderOptionsButton.y, 455);
    }
}
