use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiOptionSlider::GuiOptionSlider;
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::client::settings::GameSettings::GameSettings;
use crate::vulkan::GuiDrawList::GuiDrawList;

const FOV_ID: i32 = 0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuiOptionsAction {
    SetFov(f32),
    ToggleForceSprint,
    OpenSkinCustomisation,
    OpenSounds,
    OpenVideoSettings,
    OpenControls,
    OpenLanguage,
    OpenChatSettings,
    OpenResourcePacks,
    OpenSnooper,
    Done,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiOptionsInteraction {
    pub action: GuiOptionsAction,
    pub sound: Option<GuiSoundCommand>,
}

/// MCP 1.12.2 `GuiOptions`. `SCREEN_OPTIONS` contains the floating-point FOV
/// option, therefore the top-left control is a `GuiOptionSlider`, not a cycle
/// button.
#[derive(Debug, Clone)]
pub struct GuiOptions {
    pub GuiScreen: GuiScreen,
    pub title: String,
    fovSlider: GuiOptionSlider,
}

impl Default for GuiOptions {
    fn default() -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            title: "Options".to_owned(),
            fovSlider: GuiOptionSlider::new(FOV_ID, 0, 0, 0.5, "FOV: Normal"),
        }
    }
}

impl GuiOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initGui(&mut self, width: i32, height: i32, locale: &Locale, settings: &GameSettings) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.title = locale.translate_key("options.title").to_owned();

        self.fovSlider.GuiButton.x = width / 2 - 155;
        self.fovSlider.GuiButton.y = height / 6 - 12;
        self.fovSlider
            .setSliderValue(normalize_fov(settings.fovSetting));
        self.fovSlider
            .setDisplayString(fov_label(locale, settings.fovSetting));

        // Explicit user-requested extension retained in the otherwise unused
        // right-hand row; it does not replace or alter the vanilla FOV slider.
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            201,
            width / 2 + 5,
            height / 6 - 12,
            150,
            20,
            force_sprint_label(locale, settings.forceSprint),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            110,
            width / 2 - 155,
            height / 6 + 42,
            150,
            20,
            locale.translate_key("options.skinCustomisation"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            106,
            width / 2 + 5,
            height / 6 + 42,
            150,
            20,
            locale.translate_key("options.sounds"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            101,
            width / 2 - 155,
            height / 6 + 66,
            150,
            20,
            locale.translate_key("options.video"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            100,
            width / 2 + 5,
            height / 6 + 66,
            150,
            20,
            locale.translate_key("options.controls"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            102,
            width / 2 - 155,
            height / 6 + 90,
            150,
            20,
            locale.translate_key("options.language"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            103,
            width / 2 + 5,
            height / 6 + 90,
            150,
            20,
            locale.translate_key("options.chat.title"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            105,
            width / 2 - 155,
            height / 6 + 114,
            150,
            20,
            locale.translate_key("options.resourcepack"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            104,
            width / 2 + 5,
            height / 6 + 114,
            150,
            20,
            locale.translate_key("options.snooper.view"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::new(
            200,
            width / 2 - 100,
            height / 6 + 168,
            locale.translate_key("gui.done"),
        ));
    }

    pub fn drawScreen(
        &mut self,
        drawList: &mut GuiDrawList,
        fontRendererObj: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.drawScreenWithWorld(
            drawList,
            fontRendererObj,
            mouseX,
            mouseY,
            partialTicks,
            false,
        );
    }

    pub fn drawScreenInWorld(
        &mut self,
        drawList: &mut GuiDrawList,
        fontRendererObj: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.drawScreenWithWorld(
            drawList,
            fontRendererObj,
            mouseX,
            mouseY,
            partialTicks,
            true,
        );
    }

    fn drawScreenWithWorld(
        &mut self,
        drawList: &mut GuiDrawList,
        fontRendererObj: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
        worldLoaded: bool,
    ) {
        if worldLoaded {
            self.GuiScreen.drawDefaultBackgroundInWorld(drawList);
        } else {
            self.GuiScreen.drawDefaultBackground(drawList);
        }
        self.GuiScreen.Gui.drawCenteredString(
            fontRendererObj,
            drawList,
            &self.title,
            self.GuiScreen.width / 2,
            15,
            0x00FF_FFFF,
        );
        self.fovSlider
            .drawButton(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
        self.GuiScreen
            .drawScreen(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
        locale: &Locale,
    ) -> Option<GuiOptionsInteraction> {
        if mouseButton != 0 {
            return None;
        }
        if let Some(normalized) = self.fovSlider.mousePressed(mouseX, mouseY) {
            let fov = denormalize_fov(normalized);
            self.fovSlider.setDisplayString(fov_label(locale, fov));
            return Some(GuiOptionsInteraction {
                action: GuiOptionsAction::SetFov(fov),
                sound: None,
            });
        }
        self.GuiScreen.buttonList.iter().find_map(|button| {
            if !button.mousePressed(mouseX, mouseY) {
                return None;
            }
            let action = match button.id {
                201 => GuiOptionsAction::ToggleForceSprint,
                110 => GuiOptionsAction::OpenSkinCustomisation,
                106 => GuiOptionsAction::OpenSounds,
                101 => GuiOptionsAction::OpenVideoSettings,
                100 => GuiOptionsAction::OpenControls,
                102 => GuiOptionsAction::OpenLanguage,
                103 => GuiOptionsAction::OpenChatSettings,
                105 => GuiOptionsAction::OpenResourcePacks,
                104 => GuiOptionsAction::OpenSnooper,
                200 => GuiOptionsAction::Done,
                _ => return None,
            };
            Some(GuiOptionsInteraction {
                action,
                sound: Some(button.playPressSound()),
            })
        })
    }

    pub fn mouseDragged(&mut self, mouseX: i32, locale: &Locale) -> Option<GuiOptionsInteraction> {
        self.fovSlider.mouseDragged(mouseX).map(|normalized| {
            let fov = denormalize_fov(normalized);
            self.fovSlider.setDisplayString(fov_label(locale, fov));
            GuiOptionsInteraction {
                action: GuiOptionsAction::SetFov(fov),
                sound: None,
            }
        })
    }

    pub fn mouseReleased(&mut self, mouseX: i32, mouseY: i32) {
        self.fovSlider.mouseReleased(mouseX, mouseY);
    }
}

fn normalize_fov(fov: f32) -> f32 {
    ((fov.clamp(30.0, 110.0) - 30.0) / 80.0).clamp(0.0, 1.0)
}
fn denormalize_fov(value: f32) -> f32 {
    (30.0 + value.clamp(0.0, 1.0) * 80.0).round()
}

fn fov_label(locale: &Locale, fov: f32) -> String {
    let prefix = locale.translate_key("options.fov");
    // GameSettings#getKeyBinding: `options.fov.min` is the vanilla 70-degree
    // "Normal" value, not Options.FOV.valueMin (30 degrees).
    if fov == 70.0 {
        let text = translated_or(locale, "options.fov.min", "Normal");
        format!("{prefix}: {text}")
    } else if fov == 110.0 {
        let text = translated_or(locale, "options.fov.max", "Quake Pro");
        format!("{prefix}: {text}")
    } else {
        format!("{prefix}: {}", fov as i32)
    }
}

fn translated_or(locale: &Locale, key: &str, fallback: &str) -> String {
    let translated = locale.translate_key(key);
    if translated == key {
        fallback.to_owned()
    } else {
        translated.to_owned()
    }
}

fn force_sprint_label(locale: &Locale, enabled: bool) -> String {
    let state = locale.translate_key(if enabled { "options.on" } else { "options.off" });
    format!("Force Sprint: {state}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fov_is_the_vanilla_30_to_110_slider() {
        assert_eq!(denormalize_fov(0.0), 30.0);
        assert_eq!(denormalize_fov(0.5), 70.0);
        assert_eq!(denormalize_fov(1.0), 110.0);
        assert_eq!(normalize_fov(70.0), 0.5);
        let mut locale = Locale::default();
        locale.load_bytes(b"options.fov=FOV\noptions.fov.min=Normal\noptions.fov.max=Quake Pro\n");
        assert_eq!(fov_label(&locale, 30.0), "FOV: 30");
        assert_eq!(fov_label(&locale, 70.0), "FOV: Normal");
        assert_eq!(fov_label(&locale, 110.0), "FOV: Quake Pro");
    }

    #[test]
    fn force_sprint_uses_a_distinct_extension_button() {
        let mut locale = Locale::default();
        locale.load_bytes(b"options.on=ON\noptions.off=OFF\n");
        assert_eq!(force_sprint_label(&locale, false), "Force Sprint: OFF");
        assert_eq!(force_sprint_label(&locale, true), "Force Sprint: ON");
    }
}
