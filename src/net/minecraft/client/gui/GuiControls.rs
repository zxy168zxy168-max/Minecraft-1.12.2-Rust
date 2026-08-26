use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiKeyBindingList::{
    GuiKeyBindingList, GuiKeyBindingListAction,
};
use crate::net::minecraft::client::gui::GuiOptionSlider::GuiOptionSlider;
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::client::settings::GameSettings::GameSettings;
use crate::net::minecraft::client::settings::InputKeyCodes::{lwjgl_from_winit, mouse_code};
use crate::net::minecraft::client::settings::KeyBinding::KeyBindingId;
use crate::vulkan::GuiDrawList::GuiDrawList;
use winit::{event::MouseButton, keyboard::KeyCode};

const SENSITIVITY_ID: i32 = 0;
const INVERT_ID: i32 = 1;
const TOUCHSCREEN_ID: i32 = 2;
const AUTO_JUMP_ID: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuiControlsAction {
    None,
    Done,
    SetSensitivity(f32),
    ToggleInvertMouse,
    ToggleTouchscreen,
    ToggleAutoJump,
    SelectKeyBinding(KeyBindingId),
    SetKeyBinding { binding: KeyBindingId, code: i32 },
    ResetKeyBinding(KeyBindingId),
    ResetAll,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiControlsInteraction {
    pub action: GuiControlsAction,
    pub sound: Option<GuiSoundCommand>,
}

/// MCP 1.12.2 `GuiControls` semantic port. The original companion
/// `GuiKeyBindingList` remains a separate Rust module/class responsibility.
#[derive(Debug, Clone)]
pub struct GuiControls {
    pub GuiScreen: GuiScreen,
    pub screenTitle: String,
    sensitivitySlider: GuiOptionSlider,
    keyBindingList: GuiKeyBindingList,
    pub buttonId: Option<KeyBindingId>,
}

impl Default for GuiControls {
    fn default() -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            screenTitle: "Controls".to_owned(),
            sensitivitySlider: GuiOptionSlider::new(SENSITIVITY_ID, 0, 0, 0.5, "Sensitivity: 100%"),
            keyBindingList: GuiKeyBindingList::default(),
            buttonId: None,
        }
    }
}

impl GuiControls {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initGui(
        &mut self,
        width: i32,
        height: i32,
        locale: &Locale,
        settings: &GameSettings,
        font: &FontRenderer,
    ) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.screenTitle = translated_or(locale, "controls.title", "Controls");
        self.sensitivitySlider.GuiButton.x = width / 2 + 5;
        self.sensitivitySlider.GuiButton.y = 18;
        self.sensitivitySlider
            .setSliderValue(settings.mouseSensitivity.clamp(0.0, 1.0));
        self.sensitivitySlider
            .setDisplayString(sensitivity_label(locale, settings.mouseSensitivity));

        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            INVERT_ID,
            width / 2 - 155,
            18,
            150,
            20,
            bool_label(locale, "options.invertMouse", settings.invertMouse),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            TOUCHSCREEN_ID,
            width / 2 - 155,
            42,
            150,
            20,
            bool_label(locale, "options.touchscreen", settings.touchscreen),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            AUTO_JUMP_ID,
            width / 2 + 5,
            42,
            150,
            20,
            bool_label(locale, "options.autoJump", settings.autoJump),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            201,
            width / 2 - 155,
            height - 29,
            150,
            20,
            translated_or(locale, "controls.resetAll", "Reset All"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            200,
            width / 2 + 5,
            height - 29,
            150,
            20,
            translated_or(locale, "gui.done", "Done"),
        ));
        self.keyBindingList
            .initGui(width, height, locale, settings, font);
    }

    pub fn drawScreen(
        &mut self,
        draw: &mut GuiDrawList,
        font: &mut FontRenderer,
        locale: &Locale,
        settings: &GameSettings,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
        worldLoaded: bool,
    ) {
        if worldLoaded {
            self.GuiScreen.drawDefaultBackgroundInWorld(draw);
        } else {
            self.GuiScreen.drawDefaultBackground(draw);
        }
        self.keyBindingList.drawScreen(
            draw,
            font,
            locale,
            settings,
            self.buttonId,
            mouseX,
            mouseY,
            partialTicks,
        );

        // MCP GuiControls draws its title after GuiKeyBindingList so the list's
        // top overlay cannot cover the title.
        self.GuiScreen.Gui.drawCenteredString(
            font,
            draw,
            &self.screenTitle,
            self.GuiScreen.width / 2,
            8,
            0x00FF_FFFF,
        );
        self.sensitivitySlider
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
        if let Some(resetAll) = self
            .GuiScreen
            .buttonList
            .iter_mut()
            .find(|button| button.id == 201)
        {
            resetAll.enabled = settings
                .keyBindings
                .iter()
                .any(|binding| !binding.isDefault());
        }
        for button in &mut self.GuiScreen.buttonList {
            if button.id == INVERT_ID {
                button.displayString =
                    bool_label(locale, "options.invertMouse", settings.invertMouse);
            }
            if button.id == TOUCHSCREEN_ID {
                button.displayString =
                    bool_label(locale, "options.touchscreen", settings.touchscreen);
            }
            if button.id == AUTO_JUMP_ID {
                button.displayString = bool_label(locale, "options.autoJump", settings.autoJump);
            }
        }
        self.GuiScreen
            .drawScreen(draw, font, mouseX, mouseY, partialTicks);
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        button: MouseButton,
        locale: &Locale,
        settings: &GameSettings,
    ) -> Option<GuiControlsInteraction> {
        if let Some(binding) = self.buttonId {
            if let Some(code) = mouse_code(button) {
                self.buttonId = None;
                return Some(GuiControlsInteraction {
                    action: GuiControlsAction::SetKeyBinding { binding, code },
                    sound: None,
                });
            }
            return None;
        }
        if button != MouseButton::Left {
            return None;
        }

        if let Some(interaction) = self.keyBindingList.mouseClicked(mouseX, mouseY, settings) {
            let action = match interaction.action {
                GuiKeyBindingListAction::None => GuiControlsAction::None,
                GuiKeyBindingListAction::Select(id) => {
                    self.buttonId = Some(id);
                    GuiControlsAction::SelectKeyBinding(id)
                }
                GuiKeyBindingListAction::Reset(id) => GuiControlsAction::ResetKeyBinding(id),
            };
            return Some(GuiControlsInteraction {
                action,
                sound: interaction.sound,
            });
        }

        if let Some(value) = self.sensitivitySlider.mousePressed(mouseX, mouseY) {
            self.sensitivitySlider
                .setDisplayString(sensitivity_label(locale, value));
            return Some(GuiControlsInteraction {
                action: GuiControlsAction::SetSensitivity(value),
                sound: None,
            });
        }
        for guiButton in &self.GuiScreen.buttonList {
            if !guiButton.mousePressed(mouseX, mouseY) {
                continue;
            }
            let action = match guiButton.id {
                INVERT_ID => GuiControlsAction::ToggleInvertMouse,
                TOUCHSCREEN_ID => GuiControlsAction::ToggleTouchscreen,
                AUTO_JUMP_ID => GuiControlsAction::ToggleAutoJump,
                201 => GuiControlsAction::ResetAll,
                200 => GuiControlsAction::Done,
                _ => continue,
            };
            return Some(GuiControlsInteraction {
                action,
                sound: Some(guiButton.playPressSound()),
            });
        }
        None
    }

    pub fn mouseDragged(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        locale: &Locale,
    ) -> Option<GuiControlsInteraction> {
        if self.keyBindingList.mouseDragged(mouseY) {
            return Some(GuiControlsInteraction {
                action: GuiControlsAction::None,
                sound: None,
            });
        }
        self.sensitivitySlider.mouseDragged(mouseX).map(|value| {
            self.sensitivitySlider
                .setDisplayString(sensitivity_label(locale, value));
            GuiControlsInteraction {
                action: GuiControlsAction::SetSensitivity(value),
                sound: None,
            }
        })
    }

    pub fn mouseReleased(&mut self, mouseX: i32, mouseY: i32) {
        self.keyBindingList.mouseReleased();
        self.sensitivitySlider.mouseReleased(mouseX, mouseY);
    }

    pub fn keyPressed(
        &mut self,
        key: KeyCode,
        eventText: Option<&str>,
    ) -> Option<GuiControlsInteraction> {
        let binding = self.buttonId?;
        // MCP GuiControls#keyTyped stores Escape as NONE (0). When LWJGL
        // reports keyCode == 0 but supplies a typed character, vanilla stores
        // that UTF-16 code unit in the +256 namespace.
        let code = if key == KeyCode::Escape {
            0
        } else if let Some(code) = lwjgl_from_winit(key) {
            code
        } else {
            let unit = eventText?.encode_utf16().next()?;
            if unit == 0 {
                return None;
            }
            unit as i32 + 256
        };
        self.buttonId = None;
        Some(GuiControlsInteraction {
            action: GuiControlsAction::SetKeyBinding { binding, code },
            sound: None,
        })
    }

    pub fn scroll(&mut self, lines: f32) -> bool {
        self.keyBindingList.handleMouseWheel(lines)
    }
}

fn translated_or(locale: &Locale, key: &str, fallback: &str) -> String {
    let value = locale.translate_key(key);
    if value == key {
        fallback.to_owned()
    } else {
        value.to_owned()
    }
}

fn bool_label(locale: &Locale, key: &str, value: bool) -> String {
    let name = translated_or(locale, key, key);
    let state = translated_or(
        locale,
        if value { "options.on" } else { "options.off" },
        if value { "ON" } else { "OFF" },
    );
    format!("{name}: {state}")
}

fn sensitivity_label(locale: &Locale, value: f32) -> String {
    let prefix = translated_or(locale, "options.sensitivity", "Sensitivity");
    let normalized = value.clamp(0.0, 1.0);
    if normalized == 0.0 {
        format!(
            "{prefix}: {}",
            translated_or(locale, "options.sensitivity.min", "*yawn*")
        )
    } else if normalized == 1.0 {
        format!(
            "{prefix}: {}",
            translated_or(locale, "options.sensitivity.max", "HYPERSPEED!!!")
        )
    } else {
        format!("{prefix}: {}%", (normalized * 200.0) as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitivity_text_matches_mcp_special_cases() {
        let locale = Locale::default();
        assert!(sensitivity_label(&locale, 0.5).ends_with("100%"));
        assert!(sensitivity_label(&locale, 0.0).contains("*yawn*"));
        assert!(sensitivity_label(&locale, 1.0).contains("HYPERSPEED"));
    }
}
