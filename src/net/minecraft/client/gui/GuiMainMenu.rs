use crate::compat::Java::{string_hash_code, JavaRandom};
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::client::resources::SimpleReloadableResourceManager::ResourceManager;
use crate::net::minecraft::util::math::MathHelper::sin as minecraft_sin;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiButtonLanguage::GuiButtonLanguage;
use crate::net::optifine::CustomPanorama::VANILLA_PANORAMA_PATH;
use crate::net::optifine::CustomPanoramaProperties::CustomPanoramaProperties;
use crate::vulkan::GuiDrawList::{GuiDrawList, PanoramaCommand};

const SPLASH_TEXTS: &str = "texts/splashes.txt";
const MINECRAFT_TITLE_TEXTURES: &str = "textures/gui/title/minecraft.png";
const EDITION_TEXTURE: &str = "textures/gui/title/edition.png";
const COPYRIGHT_TEXT: &str = "Copyright Mojang AB. Do not distribute!";
const MORE_INFO_TEXT: &str = "Please click §nhere§r for more information.";
const EXCLUDED_SPLASH_HASH: i32 = 125_780_783;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainMenuDate {
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MainMenuAction {
    OpenOptions,
    OpenLanguage,
    OpenWorldSelection,
    OpenMultiplayer,
    OpenAccounts,
    Shutdown,
    OpenCopyrightCredits,
    OpenCompatibilityWarning { link: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MainMenuInteraction {
    pub action: MainMenuAction,
    /// Present only for a pressed `GuiButton`, matching GuiScreen's call to
    /// `playPressSound` before `actionPerformed`.
    pub sound: Option<GuiSoundCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MainMenuControl {
    Button(GuiButton),
    Language(GuiButtonLanguage),
}

impl MainMenuControl {
    pub const fn id(&self) -> i32 {
        match self {
            Self::Button(button) => button.id,
            Self::Language(button) => button.id(),
        }
    }

    pub const fn y(&self) -> i32 {
        match self {
            Self::Button(button) => button.y,
            Self::Language(button) => button.y(),
        }
    }

    fn draw(
        &mut self,
        draw_list: &mut GuiDrawList,
        font_renderer: &mut FontRenderer,
        mouse_x: i32,
        mouse_y: i32,
        partial_ticks: f32,
    ) {
        match self {
            Self::Button(button) => {
                button.drawButton(draw_list, font_renderer, mouse_x, mouse_y, partial_ticks)
            }
            Self::Language(button) => button.drawButton(draw_list, mouse_x, mouse_y),
        }
    }

    fn mousePressed(&self, mouse_x: i32, mouse_y: i32) -> bool {
        match self {
            Self::Button(button) => button.mousePressed(mouse_x, mouse_y),
            Self::Language(button) => button.mousePressed(mouse_x, mouse_y),
        }
    }

    fn playPressSound(&self) -> GuiSoundCommand {
        match self {
            Self::Button(button) => button.playPressSound(),
            Self::Language(button) => button.playPressSound(),
        }
    }
}

/// State and draw-order port of MCP/OptiFine `GuiMainMenu`.
///
/// The class remains renderer-independent. `render_skybox` is represented by a
/// dedicated panorama command because Vulkan cannot reproduce the old
/// framebuffer-copy calls literally; the command contains the exact source
/// textures, timer, and OptiFine blur counts needed for the equivalent pass
/// graph.
#[derive(Debug, Clone)]
pub struct GuiMainMenu {
    minceraftRoll: f32,
    splashText: String,
    panoramaTimer: f32,
    width: i32,
    height: i32,
    buttonList: Vec<MainMenuControl>,
    widthCopyright: i32,
    widthCopyrightRest: i32,
    openGLWarning1Width: i32,
    openGLWarning2Width: i32,
    openGLWarningX1: i32,
    openGLWarningY1: i32,
    openGLWarningX2: i32,
    openGLWarningY2: i32,
    openGLWarning1: String,
    openGLWarning2: String,
    openGLWarningLink: String,
    customPanoramaProperties: Option<CustomPanoramaProperties>,
}

impl GuiMainMenu {
    pub fn new(
        resources: &ResourceManager,
        random: &mut JavaRandom,
        customPanoramaProperties: Option<CustomPanoramaProperties>,
    ) -> Self {
        let splashText = select_splash(resources, random);
        let minceraftRoll = random.next_f32();
        Self {
            minceraftRoll,
            splashText,
            panoramaTimer: 0.0,
            width: 0,
            height: 0,
            buttonList: Vec::new(),
            widthCopyright: 0,
            widthCopyrightRest: 0,
            openGLWarning1Width: 0,
            openGLWarning2Width: 0,
            openGLWarningX1: 0,
            openGLWarningY1: 0,
            openGLWarningX2: 0,
            openGLWarningY2: 0,
            openGLWarning1: String::new(),
            openGLWarning2: MORE_INFO_TEXT.to_owned(),
            openGLWarningLink: String::new(),
            customPanoramaProperties,
        }
    }

    pub const fn doesGuiPauseGame(&self) -> bool {
        false
    }
    pub fn keyTyped(&mut self, _typed_char: char, _key_code: i32) {}
    pub const fn getPanoramaTimer(&self) -> f32 {
        self.panoramaTimer
    }
    pub fn getSplashText(&self) -> &str {
        &self.splashText
    }
    pub fn getButtonList(&self) -> &[MainMenuControl] {
        &self.buttonList
    }
    pub const fn width(&self) -> i32 {
        self.width
    }
    pub const fn height(&self) -> i32 {
        self.height
    }

    /// Allows the backend capability layer to expose the same warning region
    /// used by the original OpenGL check without baking Vulkan policy into GUI.
    pub fn setOpenGLWarning(
        &mut self,
        first_line: impl Into<String>,
        second_line: impl Into<String>,
        link: impl Into<String>,
    ) {
        self.openGLWarning1 = first_line.into();
        self.openGLWarning2 = second_line.into();
        self.openGLWarningLink = link.into();
    }

    pub fn initGui(
        &mut self,
        width: i32,
        height: i32,
        date: MainMenuDate,
        locale: &Locale,
        font_renderer: &FontRenderer,
    ) {
        self.width = width;
        self.height = height;
        self.buttonList.clear();
        self.widthCopyright = font_renderer.get_string_width(COPYRIGHT_TEXT);
        self.widthCopyrightRest = width - self.widthCopyright - 2;

        self.splashText = match (date.month, date.day) {
            (12, 24) => "Merry X-mas!".to_owned(),
            (1, 1) => "Happy new year!".to_owned(),
            (10, 31) => "OOoooOOOoooo! Spooky!".to_owned(),
            _ => self.splashText.clone(),
        };

        let first_button_y = height / 4 + 48;
        self.buttonList.push(MainMenuControl::Button(GuiButton::new(
            1,
            width / 2 - 100,
            first_button_y,
            locale.translate_key("menu.singleplayer"),
        )));
        self.buttonList.push(MainMenuControl::Button(GuiButton::new(
            2,
            width / 2 - 100,
            first_button_y + 24,
            locale.translate_key("menu.multiplayer"),
        )));
        self.buttonList.push(MainMenuControl::Button(GuiButton::new(
            14,
            width / 2 - 100,
            first_button_y + 48,
            "Accounts",
        )));
        self.buttonList
            .push(MainMenuControl::Button(GuiButton::newWithSize(
                0,
                width / 2 - 100,
                first_button_y + 84,
                98,
                20,
                locale.translate_key("menu.options"),
            )));
        self.buttonList
            .push(MainMenuControl::Button(GuiButton::newWithSize(
                4,
                width / 2 + 2,
                first_button_y + 84,
                98,
                20,
                locale.translate_key("menu.quit"),
            )));
        self.buttonList
            .push(MainMenuControl::Language(GuiButtonLanguage::new(
                5,
                width / 2 - 124,
                first_button_y + 84,
            )));

        self.openGLWarning1Width = font_renderer.get_string_width(&self.openGLWarning1);
        self.openGLWarning2Width = font_renderer.get_string_width(&self.openGLWarning2);
        let warning_width = self.openGLWarning1Width.max(self.openGLWarning2Width);
        self.openGLWarningX1 = (width - warning_width) / 2;
        self.openGLWarningY1 = self.buttonList[0].y() - 24;
        self.openGLWarningX2 = self.openGLWarningX1 + warning_width;
        self.openGLWarningY2 = self.openGLWarningY1 + 24;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn drawScreen(
        &mut self,
        draw_list: &mut GuiDrawList,
        font_renderer: &mut FontRenderer,
        mouse_x: i32,
        mouse_y: i32,
        partial_ticks: f32,
        system_time_millis: u64,
        version_type: &str,
        mouse_inside_window: bool,
    ) {
        self.panoramaTimer += partial_ticks;
        let panorama = self.getPanoramaParameters();
        draw_list.panorama(PanoramaCommand {
            textures: panorama.0,
            panorama_timer: self.panoramaTimer,
            first_blur_passes: panorama.1,
            second_blur_passes: panorama.2,
            final_blur_pairs: panorama.3,
            screen_width: self.width,
            screen_height: self.height,
        });

        let (overlay1_top, overlay1_bottom, overlay2_top, overlay2_bottom) = self
            .customPanoramaProperties
            .as_ref()
            .map(|properties| {
                (
                    properties.getOverlay1Top(),
                    properties.getOverlay1Bottom(),
                    properties.getOverlay2Top(),
                    properties.getOverlay2Bottom(),
                )
            })
            .unwrap_or((-2_130_706_433, 16_777_215, 0, i32::MIN));
        if overlay1_top != 0 || overlay1_bottom != 0 {
            draw_list.draw_gradient_rect(
                0,
                0,
                self.width,
                self.height,
                overlay1_top,
                overlay1_bottom,
            );
        }
        if overlay2_top != 0 || overlay2_bottom != 0 {
            draw_list.draw_gradient_rect(
                0,
                0,
                self.width,
                self.height,
                overlay2_top,
                overlay2_bottom,
            );
        }

        let title_x = self.width / 2 - 137;
        let title_texture = ResourceLocation::parse(MINECRAFT_TITLE_TEXTURES);
        if (self.minceraftRoll as f64) < 1.0e-4_f64 {
            draw_list.draw_textured_modal_rect(title_texture.clone(), title_x, 30, 0, 0, 99, 44);
            draw_list.draw_textured_modal_rect(
                title_texture.clone(),
                title_x + 99,
                30,
                129,
                0,
                27,
                44,
            );
            draw_list.draw_textured_modal_rect(
                title_texture.clone(),
                title_x + 125,
                30,
                126,
                0,
                3,
                44,
            );
            draw_list.draw_textured_modal_rect(
                title_texture.clone(),
                title_x + 128,
                30,
                99,
                0,
                26,
                44,
            );
            draw_list.draw_textured_modal_rect(title_texture, title_x + 155, 30, 0, 45, 155, 44);
        } else {
            draw_list.draw_textured_modal_rect(title_texture.clone(), title_x, 30, 0, 0, 155, 44);
            draw_list.draw_textured_modal_rect(title_texture, title_x + 155, 30, 0, 45, 155, 44);
        }
        draw_list.draw_modal_rect_with_custom_sized_texture(
            ResourceLocation::parse(EDITION_TEXTURE),
            (title_x + 88) as f32,
            67.0,
            0.0,
            0.0,
            98.0,
            14.0,
            128.0,
            16.0,
        );

        draw_list.push_matrix();
        draw_list.translate((self.width / 2 + 90) as f32, 70.0);
        draw_list.rotate_degrees(-20.0);
        // MCP calls `MathHelper.sin`, not the platform libm directly. This
        // preserves both the vanilla 65,536-entry sine table and OptiFine
        // Fast Math behavior selected by GameSettings.
        let pulse_angle =
            ((system_time_millis % 1000) as f32 / 1000.0) * (std::f32::consts::PI * 2.0);
        let pulse = minecraft_sin(pulse_angle).abs() * 0.1;
        let mut scale = 1.8 - pulse;
        scale = scale * 100.0 / (font_renderer.get_string_width(&self.splashText) + 32) as f32;
        draw_list.scale(scale, scale);
        font_renderer.draw_centered_string_with_shadow(draw_list, &self.splashText, 0, -8, -256);
        draw_list.pop_matrix();

        let version = if version_type.eq_ignore_ascii_case("release") {
            "Minecraft 1.12.2".to_owned()
        } else {
            format!("Minecraft 1.12.2/{version_type}")
        };
        font_renderer.draw_string(
            draw_list,
            &version,
            2.0,
            (self.height - 10) as f32,
            -1,
            false,
        );
        font_renderer.draw_string(
            draw_list,
            COPYRIGHT_TEXT,
            self.widthCopyrightRest as f32,
            (self.height - 10) as f32,
            -1,
            false,
        );
        if mouse_x > self.widthCopyrightRest
            && mouse_x < self.widthCopyrightRest + self.widthCopyright
            && mouse_y > self.height - 10
            && mouse_y < self.height
            && mouse_inside_window
        {
            draw_list.draw_rect(
                self.widthCopyrightRest,
                self.height - 1,
                self.widthCopyrightRest + self.widthCopyright,
                self.height,
                -1,
            );
        }

        if !self.openGLWarning1.is_empty() {
            draw_list.draw_rect(
                self.openGLWarningX1 - 2,
                self.openGLWarningY1 - 2,
                self.openGLWarningX2 + 2,
                self.openGLWarningY2 - 1,
                1_428_160_512,
            );
            font_renderer.draw_string(
                draw_list,
                &self.openGLWarning1,
                self.openGLWarningX1 as f32,
                self.openGLWarningY1 as f32,
                -1,
                false,
            );
            font_renderer.draw_string(
                draw_list,
                &self.openGLWarning2,
                ((self.width - self.openGLWarning2Width) / 2) as f32,
                (self.buttonList[0].y() - 12) as f32,
                -1,
                false,
            );
        }

        // `GuiScreen.drawScreen` draws the button list after screen-specific
        // content and warnings. Preserve that exact ordering.
        for control in &mut self.buttonList {
            control.draw(draw_list, font_renderer, mouse_x, mouse_y, partial_ticks);
        }
    }

    pub fn mouseClicked(
        &self,
        mouse_x: i32,
        mouse_y: i32,
        mouse_button: i32,
    ) -> Option<MainMenuInteraction> {
        if mouse_button == 0 {
            for control in &self.buttonList {
                if control.mousePressed(mouse_x, mouse_y) {
                    let action = action_for_button(control.id())?;
                    return Some(MainMenuInteraction {
                        action,
                        sound: Some(control.playPressSound()),
                    });
                }
            }
        }

        if !self.openGLWarning1.is_empty()
            && !self.openGLWarningLink.is_empty()
            && mouse_x >= self.openGLWarningX1
            && mouse_x <= self.openGLWarningX2
            && mouse_y >= self.openGLWarningY1
            && mouse_y <= self.openGLWarningY2
        {
            return Some(MainMenuInteraction {
                action: MainMenuAction::OpenCompatibilityWarning {
                    link: self.openGLWarningLink.clone(),
                },
                sound: None,
            });
        }

        if mouse_x > self.widthCopyrightRest
            && mouse_x < self.widthCopyrightRest + self.widthCopyright
            && mouse_y > self.height - 10
            && mouse_y < self.height
        {
            return Some(MainMenuInteraction {
                action: MainMenuAction::OpenCopyrightCredits,
                sound: None,
            });
        }
        None
    }

    fn getPanoramaParameters(&self) -> ([ResourceLocation; 6], i32, i32, i32) {
        if let Some(properties) = &self.customPanoramaProperties {
            return (
                properties.getPanoramaLocations().clone(),
                properties.getBlur1(),
                properties.getBlur2(),
                properties.getBlur3(),
            );
        }
        (
            std::array::from_fn(|index| {
                ResourceLocation::parse(format!("{VANILLA_PANORAMA_PATH}/panorama_{index}.png"))
            }),
            64,
            3,
            3,
        )
    }
}

fn action_for_button(id: i32) -> Option<MainMenuAction> {
    match id {
        0 => Some(MainMenuAction::OpenOptions),
        5 => Some(MainMenuAction::OpenLanguage),
        1 => Some(MainMenuAction::OpenWorldSelection),
        2 => Some(MainMenuAction::OpenMultiplayer),
        14 => Some(MainMenuAction::OpenAccounts),
        4 => Some(MainMenuAction::Shutdown),
        _ => None,
    }
}

fn select_splash(resources: &ResourceManager, random: &mut JavaRandom) -> String {
    let Ok(resource) = resources.get_resource(&ResourceLocation::parse(SPLASH_TEXTS)) else {
        return "missingno".to_owned();
    };
    let values = String::from_utf8_lossy(&resource.bytes)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if values.is_empty() {
        return "missingno".to_owned();
    }
    loop {
        let selected = values[random.next_i32_bound(values.len() as i32) as usize].clone();
        if string_hash_code(&selected) != EXCLUDED_SPLASH_HASH {
            return selected;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_action_ids_match_mcp() {
        assert_eq!(action_for_button(0), Some(MainMenuAction::OpenOptions));
        assert_eq!(
            action_for_button(1),
            Some(MainMenuAction::OpenWorldSelection)
        );
        assert_eq!(action_for_button(2), Some(MainMenuAction::OpenMultiplayer));
        assert_eq!(action_for_button(4), Some(MainMenuAction::Shutdown));
        assert_eq!(action_for_button(5), Some(MainMenuAction::OpenLanguage));
        assert_eq!(action_for_button(14), Some(MainMenuAction::OpenAccounts));
    }

    #[test]
    fn pressed_button_emits_click_before_action() {
        let menu = GuiMainMenu {
            minceraftRoll: 1.0,
            splashText: String::new(),
            panoramaTimer: 0.0,
            width: 320,
            height: 240,
            buttonList: vec![MainMenuControl::Button(GuiButton::new(
                2,
                10,
                20,
                "Multiplayer",
            ))],
            widthCopyright: 0,
            widthCopyrightRest: 0,
            openGLWarning1Width: 0,
            openGLWarning2Width: 0,
            openGLWarningX1: 0,
            openGLWarningY1: 0,
            openGLWarningX2: 0,
            openGLWarningY2: 0,
            openGLWarning1: String::new(),
            openGLWarning2: String::new(),
            openGLWarningLink: String::new(),
            customPanoramaProperties: None,
        };
        let interaction = menu.mouseClicked(10, 20, 0).expect("button interaction");
        assert_eq!(interaction.action, MainMenuAction::OpenMultiplayer);
        let sound = interaction.sound.expect("button click sound");
        assert_eq!(sound.event.to_string(), "minecraft:ui.button.click");
        assert_eq!(sound.pitch, 1.0);
    }

    #[test]
    fn vanilla_panorama_has_six_exact_paths() {
        let menu = GuiMainMenu {
            minceraftRoll: 1.0,
            splashText: String::new(),
            panoramaTimer: 0.0,
            width: 854,
            height: 480,
            buttonList: Vec::new(),
            widthCopyright: 0,
            widthCopyrightRest: 0,
            openGLWarning1Width: 0,
            openGLWarning2Width: 0,
            openGLWarningX1: 0,
            openGLWarningY1: 0,
            openGLWarningX2: 0,
            openGLWarningY2: 0,
            openGLWarning1: String::new(),
            openGLWarning2: String::new(),
            openGLWarningLink: String::new(),
            customPanoramaProperties: None,
        };
        let (textures, blur1, blur2, blur3) = menu.getPanoramaParameters();
        assert_eq!(
            textures[0].getPath(),
            "textures/gui/title/background/panorama_0.png"
        );
        assert_eq!(
            textures[5].getPath(),
            "textures/gui/title/background/panorama_5.png"
        );
        assert_eq!((blur1, blur2, blur3), (64, 3, 3));
    }
}
