use crate::launcher::RenderBackend::RenderBackend;
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiOptionSlider::GuiOptionSlider;
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::client::settings::GameSettings::{GameSettings, FRAMERATE_LIMIT_MAX};
use crate::vulkan::GuiDrawList::GuiDrawList;

// MCP 1.12.2 `GameSettings.Options` ordinals used as control IDs.
const GAMMA_ID: i32 = 3;
const RENDER_DISTANCE_ID: i32 = 5;
const FRAMERATE_LIMIT_ID: i32 = 8;
const GRAPHICS_ID: i32 = 11;
const AMBIENT_OCCLUSION_ID: i32 = 12;
const GUI_SCALE_ID: i32 = 13;
const USE_FULLSCREEN_ID: i32 = 22;
const DONE_ID: i32 = 200;
const RENDERER_SETTINGS_ID: i32 = 300;
const SHADERS_ID: i32 = 231;

const RENDER_DISTANCE_MIN: i32 = 2;
const RENDER_DISTANCE_MAX: i32 = 32;
const FRAMERATE_STEP: i32 = 5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuiVideoSettingsAction {
    SetGamma(f32),
    SetRenderDistance(i32),
    SetFramerate { limit: i32, enableVsync: bool },
    ToggleGraphics,
    CycleAmbientOcclusion,
    CycleGuiScale,
    ToggleFullscreen,
    ToggleRenderBackend,
    OpenShaders,
    Done,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiVideoSettingsInteraction {
    pub action: GuiVideoSettingsAction,
    pub sound: Option<GuiSoundCommand>,
}

/// First functional stage of MCP 1.12.2 `GuiVideoSettings`.
///
/// Only settings with real side effects in the current port are exposed. The
/// layout, IDs, slider normalization and widgets.png controls follow 1.12.2;
/// AO, gamma, mipmaps and the remaining controls stay absent until their
/// renderer paths exist, rather than presenting non-functional placeholders.
#[derive(Debug, Clone)]
pub struct GuiVideoSettings {
    pub GuiScreen: GuiScreen,
    pub screenTitle: String,
    graphicsButton: GuiButton,
    ambientOcclusionButton: GuiButton,
    renderDistanceSlider: GuiOptionSlider,
    framerateSlider: GuiOptionSlider,
    guiScaleButton: GuiButton,
    fullscreenButton: GuiButton,
    gammaSlider: GuiOptionSlider,
    rendererButton: GuiButton,
    shadersButton: GuiButton,
    backendNotice: String,
    doneButton: GuiButton,
}

impl Default for GuiVideoSettings {
    fn default() -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            screenTitle: "Video Settings".to_owned(),
            graphicsButton: GuiButton::newWithSize(GRAPHICS_ID, 0, 0, 150, 20, ""),
            ambientOcclusionButton: GuiButton::newWithSize(AMBIENT_OCCLUSION_ID, 0, 0, 150, 20, ""),
            renderDistanceSlider: GuiOptionSlider::new(RENDER_DISTANCE_ID, 0, 0, 0.0, ""),
            framerateSlider: GuiOptionSlider::new(FRAMERATE_LIMIT_ID, 0, 0, 1.0, ""),
            guiScaleButton: GuiButton::newWithSize(GUI_SCALE_ID, 0, 0, 150, 20, ""),
            fullscreenButton: GuiButton::newWithSize(USE_FULLSCREEN_ID, 0, 0, 150, 20, ""),
            gammaSlider: GuiOptionSlider::new(GAMMA_ID, 0, 0, 0.0, ""),
            rendererButton: GuiButton::newWithSize(RENDERER_SETTINGS_ID, 0, 0, 150, 20, ""),
            shadersButton: GuiButton::newWithSize(SHADERS_ID, 0, 0, 150, 20, "Shaders..."),
            backendNotice: String::new(),
            doneButton: GuiButton::new(DONE_ID, 0, 0, "Done"),
        }
    }
}

impl GuiVideoSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initGui(&mut self, width: i32, height: i32, locale: &Locale, settings: &GameSettings) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.screenTitle = locale.translate_key("options.videoTitle").to_owned();

        let left = width / 2 - 155;
        let right = width / 2 + 5;
        let row0 = height / 6 - 12;
        let row1 = row0 + 21;
        let row2 = row1 + 21;

        self.graphicsButton.x = left;
        self.graphicsButton.y = row0;
        self.renderDistanceSlider.GuiButton.x = right;
        self.renderDistanceSlider.GuiButton.y = row0;
        self.ambientOcclusionButton.x = left;
        self.ambientOcclusionButton.y = row1;
        self.framerateSlider.GuiButton.x = right;
        self.framerateSlider.GuiButton.y = row1;
        self.guiScaleButton.x = left;
        self.guiScaleButton.y = row2;
        self.fullscreenButton.x = right;
        self.fullscreenButton.y = row2;
        self.gammaSlider.GuiButton.x = left;
        self.gammaSlider.GuiButton.y = row2 + 21;
        self.rendererButton.x = left;
        self.rendererButton.y = row2 + 42;
        self.shadersButton.x = right;
        self.shadersButton.y = row2 + 42;
        self.doneButton.x = width / 2 - 100;
        self.doneButton.y = height / 6 + 168;

        self.syncFromSettings(locale, settings);
        self.doneButton.displayString = locale.translate_key("gui.done").to_owned();
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
            &self.screenTitle,
            self.GuiScreen.width / 2,
            15,
            0x00FF_FFFF,
        );
        self.graphicsButton
            .drawButton(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
        self.ambientOcclusionButton.drawButton(
            drawList,
            fontRendererObj,
            mouseX,
            mouseY,
            partialTicks,
        );
        self.renderDistanceSlider.drawButton(
            drawList,
            fontRendererObj,
            mouseX,
            mouseY,
            partialTicks,
        );
        self.framerateSlider
            .drawButton(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
        self.guiScaleButton
            .drawButton(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
        self.fullscreenButton
            .drawButton(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
        self.gammaSlider
            .drawButton(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
        self.rendererButton
            .drawButton(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
        self.shadersButton
            .drawButton(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
        self.GuiScreen.Gui.drawCenteredString(
            fontRendererObj,
            drawList,
            &self.backendNotice,
            self.GuiScreen.width / 2,
            self.GuiScreen.height / 6 + 132,
            0x00AA_AAAA,
        );
        self.doneButton
            .drawButton(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
        locale: &Locale,
        settings: &GameSettings,
    ) -> Option<GuiVideoSettingsInteraction> {
        if mouseButton != 0 {
            return None;
        }

        if let Some(normalized) = self.renderDistanceSlider.mousePressed(mouseX, mouseY) {
            let value = denormalizeRenderDistance(normalized);
            self.renderDistanceSlider
                .setSliderValue(normalizeRenderDistance(value));
            self.renderDistanceSlider
                .setDisplayString(renderDistanceLabel(locale, value));
            return Some(GuiVideoSettingsInteraction {
                action: GuiVideoSettingsAction::SetRenderDistance(value),
                sound: Some(self.renderDistanceSlider.playPressSound()),
            });
        }

        if let Some(normalized) = self.framerateSlider.mousePressed(mouseX, mouseY) {
            let (limit, enableVsync) = denormalizeFramerate(normalized);
            self.framerateSlider
                .setSliderValue(normalizeFramerate(limit, enableVsync));
            self.framerateSlider
                .setDisplayString(framerateLabel(locale, limit, enableVsync));
            return Some(GuiVideoSettingsInteraction {
                action: GuiVideoSettingsAction::SetFramerate { limit, enableVsync },
                sound: Some(self.framerateSlider.playPressSound()),
            });
        }

        if let Some(normalized) = self.gammaSlider.mousePressed(mouseX, mouseY) {
            let value = normalized.clamp(0.0, 1.0);
            self.gammaSlider.setSliderValue(value);
            self.gammaSlider.setDisplayString(gammaLabel(locale, value));
            return Some(GuiVideoSettingsInteraction {
                action: GuiVideoSettingsAction::SetGamma(value),
                sound: Some(self.gammaSlider.playPressSound()),
            });
        }

        if self.ambientOcclusionButton.mousePressed(mouseX, mouseY) {
            let next = (settings.ambientOcclusion + 1).rem_euclid(3);
            self.ambientOcclusionButton.displayString = ambientOcclusionLabel(locale, next);
            return Some(GuiVideoSettingsInteraction {
                action: GuiVideoSettingsAction::CycleAmbientOcclusion,
                sound: Some(self.ambientOcclusionButton.playPressSound()),
            });
        }

        if self.graphicsButton.mousePressed(mouseX, mouseY) {
            let next = !settings.fancyGraphics;
            self.graphicsButton.displayString = graphicsLabel(locale, next);
            return Some(GuiVideoSettingsInteraction {
                action: GuiVideoSettingsAction::ToggleGraphics,
                sound: Some(self.graphicsButton.playPressSound()),
            });
        }
        if self.guiScaleButton.mousePressed(mouseX, mouseY) {
            return Some(GuiVideoSettingsInteraction {
                action: GuiVideoSettingsAction::CycleGuiScale,
                sound: Some(self.guiScaleButton.playPressSound()),
            });
        }
        if self.fullscreenButton.mousePressed(mouseX, mouseY) {
            let next = !settings.fullScreen;
            self.fullscreenButton.displayString = booleanLabel(locale, "options.fullscreen", next);
            return Some(GuiVideoSettingsInteraction {
                action: GuiVideoSettingsAction::ToggleFullscreen,
                sound: Some(self.fullscreenButton.playPressSound()),
            });
        }
        if self.rendererButton.mousePressed(mouseX, mouseY) {
            let next = settings.renderBackend.toggled();
            self.rendererButton.displayString = rendererLabel(next);
            self.syncBackendNotice(settings.activeRenderBackend, next);
            return Some(GuiVideoSettingsInteraction {
                action: GuiVideoSettingsAction::ToggleRenderBackend,
                sound: Some(self.rendererButton.playPressSound()),
            });
        }
        if self.shadersButton.mousePressed(mouseX, mouseY) {
            return Some(GuiVideoSettingsInteraction {
                action: GuiVideoSettingsAction::OpenShaders,
                sound: Some(self.shadersButton.playPressSound()),
            });
        }
        self.doneButton
            .mousePressed(mouseX, mouseY)
            .then(|| GuiVideoSettingsInteraction {
                action: GuiVideoSettingsAction::Done,
                sound: Some(self.doneButton.playPressSound()),
            })
    }

    pub fn mouseDragged(&mut self, mouseX: i32, locale: &Locale) -> Option<GuiVideoSettingsAction> {
        if let Some(normalized) = self.renderDistanceSlider.mouseDragged(mouseX) {
            let value = denormalizeRenderDistance(normalized);
            self.renderDistanceSlider
                .setSliderValue(normalizeRenderDistance(value));
            self.renderDistanceSlider
                .setDisplayString(renderDistanceLabel(locale, value));
            return Some(GuiVideoSettingsAction::SetRenderDistance(value));
        }
        if let Some(normalized) = self.framerateSlider.mouseDragged(mouseX) {
            let (limit, enableVsync) = denormalizeFramerate(normalized);
            self.framerateSlider
                .setSliderValue(normalizeFramerate(limit, enableVsync));
            self.framerateSlider
                .setDisplayString(framerateLabel(locale, limit, enableVsync));
            return Some(GuiVideoSettingsAction::SetFramerate { limit, enableVsync });
        }
        if let Some(normalized) = self.gammaSlider.mouseDragged(mouseX) {
            let value = normalized.clamp(0.0, 1.0);
            self.gammaSlider.setSliderValue(value);
            self.gammaSlider.setDisplayString(gammaLabel(locale, value));
            return Some(GuiVideoSettingsAction::SetGamma(value));
        }
        None
    }

    pub fn mouseReleased(&mut self, mouseX: i32, mouseY: i32) {
        self.renderDistanceSlider.mouseReleased(mouseX, mouseY);
        self.framerateSlider.mouseReleased(mouseX, mouseY);
        self.gammaSlider.mouseReleased(mouseX, mouseY);
    }

    pub fn syncFromSettings(&mut self, locale: &Locale, settings: &GameSettings) {
        self.graphicsButton.displayString = graphicsLabel(locale, settings.fancyGraphics);
        self.ambientOcclusionButton.displayString =
            ambientOcclusionLabel(locale, settings.ambientOcclusion);
        self.renderDistanceSlider
            .setSliderValue(normalizeRenderDistance(settings.renderDistanceChunks));
        self.renderDistanceSlider
            .setDisplayString(renderDistanceLabel(locale, settings.renderDistanceChunks));
        self.framerateSlider.setSliderValue(normalizeFramerate(
            settings.limitFramerate,
            settings.enableVsync,
        ));
        self.framerateSlider.setDisplayString(framerateLabel(
            locale,
            settings.limitFramerate,
            settings.enableVsync,
        ));
        self.guiScaleButton.displayString = guiScaleLabel(locale, settings.guiScale);
        self.fullscreenButton.displayString =
            booleanLabel(locale, "options.fullscreen", settings.fullScreen);
        self.gammaSlider
            .setSliderValue(settings.gammaSetting.clamp(0.0, 1.0));
        self.gammaSlider
            .setDisplayString(gammaLabel(locale, settings.gammaSetting));
        self.rendererButton.displayString = rendererLabel(settings.renderBackend);
        self.shadersButton.displayString = "Shaders...".to_owned();
        // OptiFine 1.12.2 shader packs are OpenGL/GLSL resources. The current
        // native backend, rather than the next-launch selection, controls
        // whether the pack-management screen can be entered.
        self.shadersButton.enabled = settings.activeRenderBackend == RenderBackend::OpenGl;
        self.syncBackendNotice(settings.activeRenderBackend, settings.renderBackend);
    }

    fn syncBackendNotice(&mut self, active: RenderBackend, selected: RenderBackend) {
        self.backendNotice = if active != selected {
            format!(
                "Restart required: {} will be used next launch",
                selected.displayName()
            )
        } else if active == RenderBackend::Vulkan {
            "OptiFine shader packs require the OpenGL renderer".to_owned()
        } else {
            "OpenGL active; shader-pack management is available, rendering migration incomplete"
                .to_owned()
        };
    }
}

fn normalizeRenderDistance(value: i32) -> f32 {
    (value.clamp(RENDER_DISTANCE_MIN, RENDER_DISTANCE_MAX) - RENDER_DISTANCE_MIN) as f32
        / (RENDER_DISTANCE_MAX - RENDER_DISTANCE_MIN) as f32
}

fn denormalizeRenderDistance(normalized: f32) -> i32 {
    (RENDER_DISTANCE_MIN as f32
        + normalized.clamp(0.0, 1.0) * (RENDER_DISTANCE_MAX - RENDER_DISTANCE_MIN) as f32)
        .round() as i32
}

fn normalizeFramerate(limit: i32, enableVsync: bool) -> f32 {
    let sliderValue = if enableVsync {
        0
    } else {
        limit.clamp(FRAMERATE_STEP, FRAMERATE_LIMIT_MAX)
    };
    sliderValue as f32 / FRAMERATE_LIMIT_MAX as f32
}

fn denormalizeFramerate(normalized: f32) -> (i32, bool) {
    let raw = (normalized.clamp(0.0, 1.0) * FRAMERATE_LIMIT_MAX as f32).round() as i32;
    let snapped = ((raw + FRAMERATE_STEP / 2) / FRAMERATE_STEP * FRAMERATE_STEP)
        .clamp(0, FRAMERATE_LIMIT_MAX);
    if snapped == 0 {
        (FRAMERATE_LIMIT_MAX, true)
    } else {
        (snapped, false)
    }
}

fn graphicsLabel(locale: &Locale, fancy: bool) -> String {
    let valueKey = if fancy {
        "options.graphics.fancy"
    } else {
        "options.graphics.fast"
    };
    format!(
        "{}: {}",
        locale.translate_key("options.graphics"),
        locale.translate_key(valueKey)
    )
}

fn renderDistanceLabel(locale: &Locale, value: i32) -> String {
    let chunks = replaceSingleFormat(locale.translate_key("options.chunks"), &value.to_string());
    format!(
        "{}: {chunks}",
        locale.translate_key("options.renderDistance")
    )
}

fn framerateLabel(locale: &Locale, limit: i32, enableVsync: bool) -> String {
    let value = if enableVsync {
        locale.translate_key("options.vsync").to_owned()
    } else if limit >= FRAMERATE_LIMIT_MAX {
        locale
            .translate_key("options.framerateLimit.max")
            .to_owned()
    } else {
        replaceSingleFormat(
            locale.translate_key("options.framerate"),
            &limit.to_string(),
        )
    };
    format!(
        "{}: {value}",
        locale.translate_key("options.framerateLimit")
    )
}

fn gammaLabel(locale: &Locale, gamma: f32) -> String {
    let gamma = gamma.clamp(0.0, 1.0);
    let value = if gamma <= f32::EPSILON {
        locale.translate_key("options.gamma.min").to_owned()
    } else if (gamma - 1.0).abs() <= f32::EPSILON {
        locale.translate_key("options.gamma.max").to_owned()
    } else {
        format!("+{}%", (gamma * 100.0) as i32)
    };
    format!("{}: {value}", locale.translate_key("options.gamma"))
}

fn ambientOcclusionLabel(locale: &Locale, value: i32) -> String {
    let valueKey = match value.clamp(0, 2) {
        0 => "options.ao.off",
        1 => "options.ao.min",
        _ => "options.ao.max",
    };
    format!(
        "{}: {}",
        locale.translate_key("options.ao"),
        locale.translate_key(valueKey)
    )
}

fn guiScaleLabel(locale: &Locale, guiScale: i32) -> String {
    let value = match guiScale {
        0 => locale.translate_key("options.guiScale.auto").to_owned(),
        1 => locale.translate_key("options.guiScale.small").to_owned(),
        2 => locale.translate_key("options.guiScale.normal").to_owned(),
        3 => locale.translate_key("options.guiScale.large").to_owned(),
        value => format!("{value}x"),
    };
    format!("{}: {value}", locale.translate_key("options.guiScale"))
}

fn booleanLabel(locale: &Locale, key: &str, value: bool) -> String {
    format!(
        "{}: {}",
        locale.translate_key(key),
        locale.translate_key(if value { "options.on" } else { "options.off" }),
    )
}

fn rendererLabel(backend: RenderBackend) -> String {
    format!("Renderer: {}", backend.displayName())
}

fn replaceSingleFormat(pattern: &str, value: &str) -> String {
    pattern.replacen("%1$s", value, 1).replacen("%s", value, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framerate_slider_preserves_optifine_1122_vsync_and_unlimited_sentinels() {
        assert_eq!(denormalizeFramerate(0.0), (FRAMERATE_LIMIT_MAX, true));
        assert_eq!(denormalizeFramerate(1.0), (FRAMERATE_LIMIT_MAX, false));
        assert_eq!(denormalizeFramerate(60.0 / 260.0), (60, false));
        assert_eq!(normalizeFramerate(FRAMERATE_LIMIT_MAX, true), 0.0);
        assert_eq!(normalizeFramerate(FRAMERATE_LIMIT_MAX, false), 1.0);
    }

    #[test]
    fn render_distance_slider_uses_mcp_2_to_32_range() {
        assert_eq!(denormalizeRenderDistance(0.0), 2);
        assert_eq!(denormalizeRenderDistance(1.0), 32);
        assert!((normalizeRenderDistance(17) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn gamma_label_matches_mcp_min_max_and_percent_branches() {
        let mut locale = Locale::default();
        locale.load_bytes(
            b"options.gamma=Brightness\noptions.gamma.min=Moody\noptions.gamma.max=Bright\n",
        );
        assert!(gammaLabel(&locale, 0.0).contains("Moody"));
        assert!(gammaLabel(&locale, 1.0).contains("Bright"));
        assert!(gammaLabel(&locale, 0.42).contains("+42%"));
    }
}
