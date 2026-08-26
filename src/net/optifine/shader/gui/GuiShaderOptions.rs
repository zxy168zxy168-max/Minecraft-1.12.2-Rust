use std::{
    path::Path,
    time::{Duration, Instant},
};

use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::optifine::shader::config::ShaderPackOptions::{
    ShaderOption, ShaderOptionKind, ShaderPackOptions,
};
use crate::net::optifine::shader::IShaderPack::IShaderPack;
use crate::vulkan::GuiDrawList::GuiDrawList;

const RESET_ID: i32 = 201;
const DONE_ID: i32 = 200;
const OPTION_ID_BASE: i32 = 100;
const OPTION_TOP: i32 = 30;
const OPTION_HEIGHT: i32 = 20;
const MAX_ROWS: usize = 9;
const TOOLTIP_DELAY: Duration = Duration::from_millis(700);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiShaderOptionsAction {
    None,
    Reload,
    Close { reload: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiShaderOptionsInteraction {
    pub action: GuiShaderOptionsAction,
    pub sound: GuiSoundCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OptionEntry {
    Profile,
    Option(usize),
    Screen(String),
    Empty,
}

#[derive(Debug, Clone)]
struct OptionControl {
    slotIndex: usize,
    entry: OptionEntry,
    button: GuiButton,
    slider: bool,
}

/// Rust equivalent of OptiFine 1.12.2 `GuiShaderOptions`.
///
/// This is intentionally separate from `GuiShader`: the pack-selection screen
/// never parses shader sources. Options are parsed only when this screen is
/// opened, and nested `[screen]` pages retain the original parent-screen flow.
#[derive(Debug, Clone)]
pub struct GuiShaderOptions {
    pub GuiScreen: GuiScreen,
    pub title: String,
    options: ShaderPackOptions,
    screenName: Option<String>,
    screenStack: Vec<Option<String>>,
    controls: Vec<OptionControl>,
    slotCount: usize,
    columns: usize,
    changed: bool,
    dragging: Option<usize>,
    resetButton: GuiButton,
    doneButton: GuiButton,
    lastMouseX: i32,
    lastMouseY: i32,
    mouseStillSince: Instant,
    advancedTooltips: bool,
}

impl GuiShaderOptions {
    pub fn load(
        gameDir: &Path,
        pack: &mut dyn IShaderPack,
        language: &str,
        advancedTooltips: bool,
    ) -> std::io::Result<Self> {
        // In the original client `Shaders.shaderPackOptions` already belongs to
        // the selected pack. Reuse it before touching the ZIP. Only a cold GUI
        // open (before the renderer has loaded the pack) performs the exact
        // world-128..world128 discovery and source scan.
        let options = if let Some(options) =
            ShaderPackOptions::tryLoadCachedForLanguage(gameDir, pack, language)?
        {
            options
        } else {
            let dimensions = shaderPackDimensions(pack);
            ShaderPackOptions::loadForLanguage(gameDir, pack, &dimensions, language)?
        };
        Ok(Self::new(options, advancedTooltips))
    }

    pub fn new(options: ShaderPackOptions, advancedTooltips: bool) -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            title: "Shader Options".to_owned(),
            options,
            screenName: None,
            screenStack: Vec::new(),
            controls: Vec::new(),
            slotCount: 0,
            columns: 2,
            changed: false,
            dragging: None,
            resetButton: GuiButton::newWithSize(RESET_ID, 0, 0, 120, 20, "Reset"),
            doneButton: GuiButton::newWithSize(DONE_ID, 0, 0, 120, 20, "Done"),
            lastMouseX: 0,
            lastMouseY: 0,
            mouseStillSince: Instant::now(),
            advancedTooltips,
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.GuiScreen.setWorldAndResolution(width, height);
        self.resetButton.x = width / 2 - 140;
        self.resetButton.y = height / 6 + 179;
        self.doneButton.x = width / 2 + 20;
        self.doneButton.y = height / 6 + 179;
        self.rebuildControls();
    }

    pub fn drawScreen(
        &mut self,
        drawList: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.updateMouseStill(mouseX, mouseY);
        let heading = self
            .screenName
            .as_deref()
            .map(|name| self.options.screenText(name))
            .unwrap_or(&self.title);
        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            heading,
            self.GuiScreen.width / 2,
            15,
            0x00FF_FFFF,
        );

        for index in 0..self.controls.len() {
            self.updateControlText(index, font);
            let slider = self.controls[index].slider;
            {
                let control = &mut self.controls[index];
                control
                    .button
                    .drawButton(drawList, font, mouseX, mouseY, partialTicks);
            }
            if slider {
                let control = &self.controls[index];
                self.drawSliderKnob(drawList, control);
            }
        }
        self.resetButton
            .drawButton(drawList, font, mouseX, mouseY, partialTicks);
        self.doneButton
            .drawButton(drawList, font, mouseX, mouseY, partialTicks);
        self.drawTooltip(drawList, font, mouseX, mouseY);
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
        shiftDown: bool,
    ) -> Option<GuiShaderOptionsInteraction> {
        if mouseButton != 0 && mouseButton != 1 {
            return None;
        }
        if mouseButton == 0 && self.resetButton.mousePressed(mouseX, mouseY) {
            self.changed |= self.options.resetAll();
            return Some(GuiShaderOptionsInteraction {
                action: GuiShaderOptionsAction::None,
                sound: self.resetButton.playPressSound(),
            });
        }
        if mouseButton == 0 && self.doneButton.mousePressed(mouseX, mouseY) {
            let action = self.finishOrReturn();
            return Some(GuiShaderOptionsInteraction {
                action,
                sound: self.doneButton.playPressSound(),
            });
        }

        let controlIndex = self
            .controls
            .iter()
            .position(|control| control.button.mousePressed(mouseX, mouseY))?;
        let sound = self.controls[controlIndex].button.playPressSound();
        let entry = self.controls[controlIndex].entry.clone();
        let slider = self.controls[controlIndex].slider;
        match entry {
            OptionEntry::Screen(name) if mouseButton == 0 => {
                self.screenStack.push(self.screenName.clone());
                self.screenName = Some(name);
                self.dragging = None;
                self.rebuildControls();
            }
            OptionEntry::Profile => {
                self.changed |= self.changeProfile(shiftDown, mouseButton == 1);
            }
            OptionEntry::Option(optionIndex) => {
                let normalized = slider.then(|| {
                    let button = &self.controls[controlIndex].button;
                    ((mouseX - (button.x + 4)) as f32 / (button.getButtonWidth() - 8).max(1) as f32)
                        .clamp(0.0, 1.0)
                });
                self.changed |=
                    self.changeOption(optionIndex, shiftDown, mouseButton == 1, normalized);
                if slider && mouseButton == 0 && !shiftDown {
                    self.dragging = Some(controlIndex);
                }
            }
            _ => {}
        }
        Some(GuiShaderOptionsInteraction {
            action: GuiShaderOptionsAction::None,
            sound,
        })
    }

    pub fn mouseDragged(&mut self, mouseX: i32) -> bool {
        let Some(controlIndex) = self.dragging else {
            return false;
        };
        let Some(control) = self.controls.get(controlIndex) else {
            return false;
        };
        let OptionEntry::Option(optionIndex) = &control.entry else {
            return false;
        };
        let optionIndex = *optionIndex;
        let normalized = ((mouseX - (control.button.x + 4)) as f32
            / (control.button.getButtonWidth() - 8).max(1) as f32)
            .clamp(0.0, 1.0);
        let Some(option) = self.options.options.get_mut(optionIndex) else {
            return false;
        };
        if !option.enabled {
            return false;
        }
        let changed = option.setIndexNormalized(normalized);
        self.changed |= changed;
        changed
    }

    pub fn mouseReleased(&mut self) {
        self.dragging = None;
    }

    pub fn close(&mut self) -> GuiShaderOptionsAction {
        self.finishOrReturn()
    }

    pub fn options(&self) -> &ShaderPackOptions {
        &self.options
    }

    fn finishOrReturn(&mut self) -> GuiShaderOptionsAction {
        let reload = self.saveChangedOptions();
        if let Some(previous) = self.screenStack.pop() {
            self.screenName = previous;
            self.dragging = None;
            self.rebuildControls();
            return if reload {
                GuiShaderOptionsAction::Reload
            } else {
                GuiShaderOptionsAction::None
            };
        }
        GuiShaderOptionsAction::Close { reload }
    }

    fn saveChangedOptions(&mut self) -> bool {
        if !self.changed {
            return false;
        }
        if let Err(error) = self.options.save() {
            log::error!(
                "Couldn't save shader options for {}: {error}",
                self.options.packName,
            );
            return false;
        }
        self.changed = false;
        true
    }

    fn changeProfile(&mut self, reset: bool, previous: bool) -> bool {
        if self.options.profiles.is_empty() {
            return false;
        }
        let next = if reset {
            self.options.defaultProfileIndex().unwrap_or(0)
        } else {
            match self.options.activeProfileIndex() {
                Some(active) if previous => {
                    (active + self.options.profiles.len() - 1) % self.options.profiles.len()
                }
                Some(active) => (active + 1) % self.options.profiles.len(),
                None if previous => self.options.profiles.len() - 1,
                None => 0,
            }
        };
        self.options.applyProfile(next)
    }

    fn changeOption(
        &mut self,
        optionIndex: usize,
        reset: bool,
        previous: bool,
        normalized: Option<f32>,
    ) -> bool {
        let Some(option) = self.options.options.get_mut(optionIndex) else {
            return false;
        };
        if !option.enabled {
            return false;
        }
        let before = option.value.clone();
        if reset {
            option.resetValue();
        } else if previous {
            option.prevValue();
        } else if let Some(normalized) = normalized {
            option.setIndexNormalized(normalized);
        } else {
            option.nextValue();
        }
        option.value != before
    }

    fn rebuildControls(&mut self) {
        self.controls.clear();
        let tokens = self.options.screenTokens(self.screenName.as_deref());
        self.slotCount = tokens.len();
        let configured = self
            .options
            .screenColumnCount(self.screenName.as_deref(), 2);
        let required = (self.slotCount + MAX_ROWS - 1) / MAX_ROWS;
        self.columns = configured.max(required).max(1);
        let columnWidth = (self.GuiScreen.width / self.columns as i32)
            .min(200)
            .max(10);
        let startX = (self.GuiScreen.width - columnWidth * self.columns as i32) / 2;

        for (slotIndex, token) in tokens.into_iter().enumerate() {
            let entry = if token == "<profile>" {
                OptionEntry::Profile
            } else if token == "<empty>" {
                OptionEntry::Empty
            } else if let Some(name) = token
                .strip_prefix('[')
                .and_then(|value| value.strip_suffix(']'))
            {
                OptionEntry::Screen(name.to_owned())
            } else if let Some(index) = self
                .options
                .options
                .iter()
                .position(|option| option.name == token)
            {
                OptionEntry::Option(index)
            } else {
                OptionEntry::Empty
            };
            if matches!(entry, OptionEntry::Empty) {
                continue;
            }
            if let OptionEntry::Option(index) = &entry {
                if !self
                    .options
                    .options
                    .get(*index)
                    .is_some_and(|option| option.visible)
                {
                    continue;
                }
            }
            let column = slotIndex % self.columns;
            let row = slotIndex / self.columns;
            let x = startX + column as i32 * columnWidth + 5;
            let y = OPTION_TOP + row as i32 * OPTION_HEIGHT;
            let width = columnWidth - 10;
            let slider = matches!(&entry, OptionEntry::Option(index)
                if self.options.sliders.contains(&self.options.options[*index].name));
            let mut button = GuiButton::newWithSize(
                OPTION_ID_BASE + slotIndex as i32,
                x,
                y,
                width,
                OPTION_HEIGHT,
                "",
            );
            button.enabled = match &entry {
                OptionEntry::Option(index) => self.options.options[*index].enabled,
                _ => true,
            };
            self.controls.push(OptionControl {
                slotIndex,
                entry,
                button,
                slider,
            });
        }
    }

    fn updateControlText(&mut self, controlIndex: usize, font: &FontRenderer) {
        let Some(control) = self.controls.get(controlIndex) else {
            return;
        };
        let width = control.button.getButtonWidth();
        let text = match &control.entry {
            OptionEntry::Profile => {
                let value = self
                    .options
                    .activeProfileIndex()
                    .and_then(|index| self.options.profiles.get(index))
                    .map(|profile| {
                        self.options
                            .translate(&format!("profile.{}", profile.name), &profile.name)
                            .to_owned()
                    })
                    .unwrap_or_else(|| "<custom>".to_owned());
                let default = self
                    .options
                    .defaultProfileIndex()
                    .and_then(|index| self.options.profiles.get(index))
                    .map(|profile| profile.name.as_str());
                let active = self
                    .options
                    .activeProfileIndex()
                    .and_then(|index| self.options.profiles.get(index))
                    .map(|profile| profile.name.as_str());
                let changed = active != default;
                let color = if value == "<custom>" { "§c" } else { "§a" };
                fit_button_text(font, "Profile", &value, width, changed.then_some(color))
            }
            OptionEntry::Option(index) => {
                let Some(option) = self.options.options.get(*index) else {
                    return;
                };
                let name = self.options.optionNameText(option);
                let value = self.options.optionValueText(option);
                let color = option.isChanged().then_some(option_value_color(option));
                fit_button_text(font, name, &value, width, color)
            }
            OptionEntry::Screen(name) => format!("{}...", self.options.screenText(name)),
            OptionEntry::Empty => String::new(),
        };
        if let Some(control) = self.controls.get_mut(controlIndex) {
            control.button.displayString = text;
        }
    }

    fn drawSliderKnob(&self, drawList: &mut GuiDrawList, control: &OptionControl) {
        let OptionEntry::Option(optionIndex) = &control.entry else {
            return;
        };
        let Some(option) = self.options.options.get(*optionIndex) else {
            return;
        };
        let knobX = control.button.x
            + (option.indexNormalized() * (control.button.getButtonWidth() - 8) as f32) as i32;
        let texture = ResourceLocation::parse("textures/gui/widgets.png");
        drawList.draw_textured_modal_rect(texture.clone(), knobX, control.button.y, 0, 66, 4, 20);
        drawList.draw_textured_modal_rect(texture, knobX + 4, control.button.y, 196, 66, 4, 20);
    }

    fn updateMouseStill(&mut self, mouseX: i32, mouseY: i32) {
        if (mouseX - self.lastMouseX).abs() > 5 || (mouseY - self.lastMouseY).abs() > 5 {
            self.lastMouseX = mouseX;
            self.lastMouseY = mouseY;
            self.mouseStillSince = Instant::now();
        }
    }

    fn drawTooltip(
        &self,
        drawList: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
    ) {
        if self.mouseStillSince.elapsed() < TOOLTIP_DELAY {
            return;
        }
        let Some(control) = self
            .controls
            .iter()
            .find(|control| control.button.visible && control.button.contains(mouseX, mouseY))
        else {
            return;
        };
        let (title, description, paths, defaultValue, enabled) = match &control.entry {
            OptionEntry::Option(index) => {
                let Some(option) = self.options.options.get(*index) else {
                    return;
                };
                (
                    self.options.optionNameText(option).to_owned(),
                    self.options.optionDescriptionText(option),
                    Some(option.paths.join(", ")),
                    Some(
                        self.options
                            .translate(
                                &format!("value.{}.{}", option.name, option.valueDefault),
                                &option.valueDefault,
                            )
                            .to_owned(),
                    ),
                    option.enabled,
                )
            }
            OptionEntry::Screen(name) => (
                self.options.screenText(name).to_owned(),
                self.options.screenDescription(name).unwrap_or_default(),
                None,
                None,
                true,
            ),
            _ => return,
        };
        let mut lines = vec![title];
        if !description.trim().is_empty() {
            for sentence in description.trim().trim_start_matches("//").split(". ") {
                let sentence = sentence.trim().trim_end_matches('.');
                if !sentence.is_empty() {
                    lines.push(format!("- {sentence}"));
                }
            }
        }
        if self.advancedTooltips {
            if let OptionEntry::Option(index) = &control.entry {
                if let Some(option) = self.options.options.get(*index) {
                    lines.push(format!("§8ID: {}", option.name));
                }
            }
            if let Some(paths) = paths.filter(|value| !value.is_empty()) {
                lines.push(format!("§8From: {paths}"));
            }
            if let Some(defaultValue) = defaultValue {
                lines.push(format!(
                    "§8Default: {}",
                    if enabled {
                        defaultValue
                    } else {
                        "Ambiguous".to_owned()
                    },
                ));
            }
        }

        let left = self.GuiScreen.width / 2 - 150;
        let mut top = self.GuiScreen.height / 6 - 7;
        if mouseY <= top + 98 {
            top += 105;
        }
        let right = left + 300;
        let bottom = top + 94;
        drawList.draw_rect(left, top, right, bottom, 0xE000_0000_u32 as i32);
        let wrapped = wrap_lines(font, &lines, 290);
        for (index, line) in wrapped.into_iter().take(8).enumerate() {
            let color = if line.ends_with('!') {
                0x00FF_2020
            } else {
                0x00DD_DDDD
            };
            font.draw_string_with_shadow(
                drawList,
                &line,
                (left + 5) as f32,
                (top + 5 + index as i32 * 11) as f32,
                color,
            );
        }
    }
}

fn shaderPackDimensions(pack: &mut dyn IShaderPack) -> Vec<i32> {
    (-128..=128)
        .filter(|dimension| pack.hasDirectory(&format!("/shaders/world{dimension}")))
        .collect()
}

fn fit_button_text(
    font: &FontRenderer,
    name: &str,
    value: &str,
    width: i32,
    valueColor: Option<&str>,
) -> String {
    // OptiFine 1.12.2 reserves the width of ": " + Lang.getOff(),
    // independent of the currently selected value.
    let reserved = font.get_string_width(": OFF") + 5;
    let mut visibleName = name.to_owned();
    while !visibleName.is_empty() && font.get_string_width(&visibleName) + reserved >= width {
        visibleName.pop();
    }
    format!("{visibleName}: {}{value}", valueColor.unwrap_or(""))
}

fn option_value_color(option: &ShaderOption) -> &'static str {
    match option.kind {
        ShaderOptionKind::Switch | ShaderOptionKind::ConstSwitch => {
            if option.value.eq_ignore_ascii_case("true") {
                "§a"
            } else {
                "§c"
            }
        }
        ShaderOptionKind::Variable | ShaderOptionKind::ConstVariable => {
            if option.value.eq_ignore_ascii_case("false")
                || option.value.eq_ignore_ascii_case("off")
            {
                "§c"
            } else {
                "§a"
            }
        }
    }
}

fn wrap_lines(font: &FontRenderer, lines: &[String], width: i32) -> Vec<String> {
    let mut output = Vec::new();
    for line in lines {
        let mut current = String::new();
        for word in line.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_owned()
            } else {
                format!("{current} {word}")
            };
            if !current.is_empty() && font.get_string_width(&candidate) > width {
                output.push(std::mem::take(&mut current));
                current.push_str(word);
            } else {
                current = candidate;
            }
        }
        if !current.is_empty() {
            output.push(current);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_layout_expands_columns_to_keep_nine_rows() {
        let options = ShaderPackOptions {
            screens: [(
                "screen".to_owned(),
                (0..19).map(|index| format!("O{index}")).collect(),
            )]
            .into_iter()
            .collect(),
            options: (0..19)
                .map(|index| ShaderOption {
                    name: format!("O{index}"),
                    description: String::new(),
                    value: "true".to_owned(),
                    values: vec!["false".to_owned(), "true".to_owned()],
                    valueDefault: "true".to_owned(),
                    paths: vec!["test.fsh".to_owned()],
                    enabled: true,
                    visible: true,
                    kind: ShaderOptionKind::Switch,
                    constType: None,
                })
                .collect(),
            ..ShaderPackOptions::default()
        };
        let mut screen = GuiShaderOptions::new(options, false);
        screen.initGui(800, 480);
        assert_eq!(screen.columns, 3);
        assert_eq!(screen.controls.len(), 19);
    }

    #[test]
    fn empty_screen_tokens_reserve_layout_without_drawing_buttons() {
        let options = ShaderPackOptions {
            screens: [(
                "screen".to_owned(),
                vec!["<empty>".to_owned(), "A".to_owned()],
            )]
            .into_iter()
            .collect(),
            options: vec![ShaderOption {
                name: "A".to_owned(),
                description: String::new(),
                value: "true".to_owned(),
                values: vec!["false".to_owned(), "true".to_owned()],
                valueDefault: "true".to_owned(),
                paths: vec!["test.fsh".to_owned()],
                enabled: true,
                visible: true,
                kind: ShaderOptionKind::Switch,
                constType: None,
            }],
            ..ShaderPackOptions::default()
        };
        let mut screen = GuiShaderOptions::new(options, false);
        screen.initGui(800, 480);
        assert_eq!(screen.slotCount, 2);
        assert_eq!(screen.controls.len(), 1);
        assert_eq!(screen.controls[0].slotIndex, 1);
    }
}
