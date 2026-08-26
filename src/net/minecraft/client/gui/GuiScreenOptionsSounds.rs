use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiOptionSlider::GuiOptionSlider;
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::client::settings::GameSettings::GameSettings;
use crate::net::minecraft::util::SoundCategory::SoundCategory;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuiScreenOptionsSoundsAction {
    SetSoundLevel(SoundCategory, f32),
    ToggleSubtitles,
    Done,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiScreenOptionsSoundsInteraction {
    pub action: GuiScreenOptionsSoundsAction,
    pub sound: Option<GuiSoundCommand>,
}

/// MCP 1.12.2 `GuiScreenOptionsSounds`: master volume, nine category sliders,
/// subtitles and Done. Slider values write the same `soundCategory_*` keys.
#[derive(Debug, Clone)]
pub struct GuiScreenOptionsSounds {
    pub GuiScreen: GuiScreen,
    title: String,
    sliders: Vec<(SoundCategory, GuiOptionSlider)>,
    subtitles: GuiButton,
    done: GuiButton,
}

impl Default for GuiScreenOptionsSounds {
    fn default() -> Self {
        let sliders = SoundCategory::ALL
            .into_iter()
            .map(|category| {
                (
                    category,
                    GuiOptionSlider::new(100 + category.index() as i32, 0, 0, 1.0, ""),
                )
            })
            .collect();
        Self {
            GuiScreen: GuiScreen::default(),
            title: "Music & Sounds".to_owned(),
            sliders,
            subtitles: GuiButton::newWithSize(201, 0, 0, 150, 20, ""),
            done: GuiButton::new(200, 0, 0, "Done"),
        }
    }
}

impl GuiScreenOptionsSounds {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initGui(&mut self, width: i32, height: i32, locale: &Locale, settings: &GameSettings) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.title = locale.translate_key("options.sounds.title").to_owned();
        let top = height / 6 - 12;
        for (index, (category, slider)) in self.sliders.iter_mut().enumerate() {
            if *category == SoundCategory::Master {
                slider.GuiButton.x = width / 2 - 155;
                slider.GuiButton.y = top;
                slider.GuiButton.setWidth(310);
            } else {
                let ordinal = index - 1;
                slider.GuiButton.x = if ordinal % 2 == 0 {
                    width / 2 - 155
                } else {
                    width / 2 + 5
                };
                slider.GuiButton.y = top + 24 + (ordinal / 2) as i32 * 24;
                slider.GuiButton.setWidth(150);
            }
            let value = settings.getSoundLevel(*category);
            slider.setSliderValue(value);
            slider.setDisplayString(volume_label(locale, *category, value));
        }
        self.subtitles.x = width / 2 - 75;
        self.subtitles.y = top + 144;
        self.subtitles.displayString =
            bool_label(locale, "options.showSubtitles", settings.showSubtitles);
        self.done.x = width / 2 - 100;
        self.done.y = height / 6 + 168;
        self.done.displayString = locale.translate_key("gui.done").to_owned();
    }

    pub fn drawScreen(
        &mut self,
        draw: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partial: f32,
    ) {
        self.draw(draw, font, mouseX, mouseY, partial, false);
    }
    pub fn drawScreenInWorld(
        &mut self,
        draw: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partial: f32,
    ) {
        self.draw(draw, font, mouseX, mouseY, partial, true);
    }
    fn draw(
        &mut self,
        draw: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partial: f32,
        world: bool,
    ) {
        if world {
            self.GuiScreen.drawDefaultBackgroundInWorld(draw);
        } else {
            self.GuiScreen.drawDefaultBackground(draw);
        }
        self.GuiScreen.Gui.drawCenteredString(
            font,
            draw,
            &self.title,
            self.GuiScreen.width / 2,
            15,
            0x00FF_FFFF,
        );
        for (_, slider) in &mut self.sliders {
            slider.drawButton(draw, font, mouseX, mouseY, partial);
        }
        self.subtitles
            .drawButton(draw, font, mouseX, mouseY, partial);
        self.done.drawButton(draw, font, mouseX, mouseY, partial);
    }

    pub fn mouseClicked(
        &mut self,
        x: i32,
        y: i32,
        button: i32,
        locale: &Locale,
        settings: &GameSettings,
    ) -> Option<GuiScreenOptionsSoundsInteraction> {
        if button != 0 {
            return None;
        }
        for (category, slider) in &mut self.sliders {
            if let Some(value) = slider.mousePressed(x, y) {
                let value = value.clamp(0.0, 1.0);
                slider.setSliderValue(value);
                slider.setDisplayString(volume_label(locale, *category, value));
                return Some(GuiScreenOptionsSoundsInteraction {
                    action: GuiScreenOptionsSoundsAction::SetSoundLevel(*category, value),
                    sound: None,
                });
            }
        }
        if self.subtitles.mousePressed(x, y) {
            self.subtitles.displayString =
                bool_label(locale, "options.showSubtitles", !settings.showSubtitles);
            return Some(GuiScreenOptionsSoundsInteraction {
                action: GuiScreenOptionsSoundsAction::ToggleSubtitles,
                sound: Some(self.subtitles.playPressSound()),
            });
        }
        self.done
            .mousePressed(x, y)
            .then(|| GuiScreenOptionsSoundsInteraction {
                action: GuiScreenOptionsSoundsAction::Done,
                sound: Some(self.done.playPressSound()),
            })
    }

    pub fn mouseDragged(
        &mut self,
        x: i32,
        locale: &Locale,
    ) -> Option<GuiScreenOptionsSoundsAction> {
        for (category, slider) in &mut self.sliders {
            if let Some(value) = slider.mouseDragged(x) {
                let value = value.clamp(0.0, 1.0);
                slider.setSliderValue(value);
                slider.setDisplayString(volume_label(locale, *category, value));
                return Some(GuiScreenOptionsSoundsAction::SetSoundLevel(
                    *category, value,
                ));
            }
        }
        None
    }
    pub fn mouseReleased(&mut self, x: i32, y: i32) -> Option<GuiSoundCommand> {
        let releasedPressedSlider = self.sliders.iter().any(|(_, slider)| slider.dragging);
        for (_, slider) in &mut self.sliders {
            slider.mouseReleased(x, y);
        }
        releasedPressedSlider.then(|| self.done.playPressSound())
    }
}

fn volume_label(locale: &Locale, category: SoundCategory, value: f32) -> String {
    let key = match category {
        SoundCategory::Master => "soundCategory.master",
        SoundCategory::Music => "soundCategory.music",
        SoundCategory::Records => "soundCategory.record",
        SoundCategory::Weather => "soundCategory.weather",
        SoundCategory::Blocks => "soundCategory.block",
        SoundCategory::Hostile => "soundCategory.hostile",
        SoundCategory::Neutral => "soundCategory.neutral",
        SoundCategory::Players => "soundCategory.player",
        SoundCategory::Ambient => "soundCategory.ambient",
        SoundCategory::Voice => "soundCategory.voice",
    };
    let amount = if value <= 0.0 {
        locale.translate_key("options.off").to_owned()
    } else {
        format!("{}%", (value * 100.0) as i32)
    };
    format!("{}: {}", locale.translate_key(key), amount)
}
fn bool_label(locale: &Locale, key: &str, value: bool) -> String {
    format!(
        "{}: {}",
        locale.translate_key(key),
        locale.translate_key(if value { "options.on" } else { "options.off" })
    )
}
