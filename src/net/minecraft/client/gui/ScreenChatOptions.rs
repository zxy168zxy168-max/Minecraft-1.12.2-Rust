use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiOptionSlider::GuiOptionSlider;
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::client::settings::GameSettings::GameSettings;
use crate::net::minecraft::entity::player::EntityPlayer::EnumChatVisibility;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScreenChatOptionsAction {
    CycleVisibility,
    ToggleColours,
    ToggleLinks,
    ToggleLinksPrompt,
    ToggleReducedDebugInfo,
    SetOpacity(f32),
    SetScale(f32),
    SetHeightFocused(f32),
    SetHeightUnfocused(f32),
    SetWidth(f32),
    Done,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScreenChatOptionsInteraction {
    pub action: ScreenChatOptionsAction,
    pub sound: Option<GuiSoundCommand>,
}

/// MCP 1.12.2 `ScreenChatOptions`.
///
/// The ten controls retain the source order because their two-column position
/// is derived from the option ordinal in `CHAT_OPTIONS`.
#[derive(Debug, Clone)]
pub struct ScreenChatOptions {
    pub GuiScreen: GuiScreen,
    title: String,
    visibility: GuiButton,
    colours: GuiButton,
    links: GuiButton,
    opacity: GuiOptionSlider,
    prompt: GuiButton,
    scale: GuiOptionSlider,
    heightFocused: GuiOptionSlider,
    heightUnfocused: GuiOptionSlider,
    width: GuiOptionSlider,
    reducedDebug: GuiButton,
    done: GuiButton,
}

impl Default for ScreenChatOptions {
    fn default() -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            title: "Chat Settings".to_owned(),
            visibility: GuiButton::newWithSize(0, 0, 0, 150, 20, ""),
            colours: GuiButton::newWithSize(1, 0, 0, 150, 20, ""),
            links: GuiButton::newWithSize(2, 0, 0, 150, 20, ""),
            opacity: GuiOptionSlider::new(3, 0, 0, 1.0, ""),
            prompt: GuiButton::newWithSize(4, 0, 0, 150, 20, ""),
            scale: GuiOptionSlider::new(5, 0, 0, 1.0, ""),
            heightFocused: GuiOptionSlider::new(6, 0, 0, 1.0, ""),
            heightUnfocused: GuiOptionSlider::new(7, 0, 0, 1.0, ""),
            width: GuiOptionSlider::new(8, 0, 0, 1.0, ""),
            reducedDebug: GuiButton::newWithSize(9, 0, 0, 150, 20, ""),
            done: GuiButton::new(200, 0, 0, "Done"),
        }
    }
}

impl ScreenChatOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initGui(&mut self, width: i32, height: i32, locale: &Locale, settings: &GameSettings) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.title = locale.translate_key("options.chat.title").to_owned();

        let left = width / 2 - 155;
        let right = width / 2 + 5;
        let top = height / 6;
        set_position(&mut self.visibility, left, top);
        set_position(&mut self.colours, right, top);
        set_slider_position(&mut self.links, left, top + 24);
        self.opacity.GuiButton.x = right;
        self.opacity.GuiButton.y = top + 24;
        set_position(&mut self.prompt, left, top + 48);
        self.scale.GuiButton.x = right;
        self.scale.GuiButton.y = top + 48;
        self.heightFocused.GuiButton.x = left;
        self.heightFocused.GuiButton.y = top + 72;
        self.heightUnfocused.GuiButton.x = right;
        self.heightUnfocused.GuiButton.y = top + 72;
        self.width.GuiButton.x = left;
        self.width.GuiButton.y = top + 96;
        set_position(&mut self.reducedDebug, right, top + 96);
        self.done.x = width / 2 - 100;
        self.done.y = height / 6 + 144;

        self.sync(locale, settings);
    }

    fn sync(&mut self, locale: &Locale, settings: &GameSettings) {
        self.visibility.displayString = format!(
            "{}: {}",
            locale.translate_key("options.chat.visibility"),
            visibility_name(locale, settings.chatVisibility),
        );
        self.colours.displayString = bool_label(locale, "options.chat.color", settings.chatColours);
        self.links.displayString = bool_label(locale, "options.chat.links", settings.chatLinks);
        self.prompt.displayString = bool_label(
            locale,
            "options.chat.links.prompt",
            settings.chatLinksPrompt,
        );
        self.reducedDebug.displayString = bool_label(
            locale,
            "options.reducedDebugInfo",
            settings.reducedDebugInfo,
        );
        set_slider(
            &mut self.opacity,
            locale,
            "options.chat.opacity",
            settings.chatOpacity,
            ChatSliderKind::Opacity,
        );
        set_slider(
            &mut self.scale,
            locale,
            "options.chat.scale",
            settings.chatScale,
            ChatSliderKind::Percent,
        );
        set_slider(
            &mut self.heightFocused,
            locale,
            "options.chat.height.focused",
            settings.chatHeightFocused,
            ChatSliderKind::Height,
        );
        set_slider(
            &mut self.heightUnfocused,
            locale,
            "options.chat.height.unfocused",
            settings.chatHeightUnfocused,
            ChatSliderKind::Height,
        );
        set_slider(
            &mut self.width,
            locale,
            "options.chat.width",
            settings.chatWidth,
            ChatSliderKind::Width,
        );
        self.done.displayString = locale.translate_key("gui.done").to_owned();
    }

    pub fn drawScreen(
        &mut self,
        draw: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.draw(draw, font, mouseX, mouseY, partialTicks, false);
    }

    pub fn drawScreenInWorld(
        &mut self,
        draw: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.draw(draw, font, mouseX, mouseY, partialTicks, true);
    }

    fn draw(
        &mut self,
        draw: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
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
            20,
            0x00FF_FFFF,
        );
        self.visibility
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
        self.colours
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
        self.links
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
        self.opacity
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
        self.prompt
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
        self.scale
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
        self.heightFocused
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
        self.heightUnfocused
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
        self.width
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
        self.reducedDebug
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
        self.done
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
        locale: &Locale,
        _settings: &GameSettings,
    ) -> Option<ScreenChatOptionsInteraction> {
        if mouseButton != 0 {
            return None;
        }
        if self.visibility.mousePressed(mouseX, mouseY) {
            return Some(button_interaction(
                &self.visibility,
                ScreenChatOptionsAction::CycleVisibility,
            ));
        }
        if self.colours.mousePressed(mouseX, mouseY) {
            return Some(button_interaction(
                &self.colours,
                ScreenChatOptionsAction::ToggleColours,
            ));
        }
        if self.links.mousePressed(mouseX, mouseY) {
            return Some(button_interaction(
                &self.links,
                ScreenChatOptionsAction::ToggleLinks,
            ));
        }
        if let Some(value) = self.opacity.mousePressed(mouseX, mouseY) {
            set_slider(
                &mut self.opacity,
                locale,
                "options.chat.opacity",
                value,
                ChatSliderKind::Opacity,
            );
            return Some(slider_interaction(
                &mut self.opacity,
                value,
                ScreenChatOptionsAction::SetOpacity,
            ));
        }
        if self.prompt.mousePressed(mouseX, mouseY) {
            return Some(button_interaction(
                &self.prompt,
                ScreenChatOptionsAction::ToggleLinksPrompt,
            ));
        }
        if let Some(value) = self.scale.mousePressed(mouseX, mouseY) {
            set_slider(
                &mut self.scale,
                locale,
                "options.chat.scale",
                value,
                ChatSliderKind::Percent,
            );
            return Some(slider_interaction(
                &mut self.scale,
                value,
                ScreenChatOptionsAction::SetScale,
            ));
        }
        if let Some(value) = self.heightFocused.mousePressed(mouseX, mouseY) {
            set_slider(
                &mut self.heightFocused,
                locale,
                "options.chat.height.focused",
                value,
                ChatSliderKind::Height,
            );
            return Some(slider_interaction(
                &mut self.heightFocused,
                value,
                ScreenChatOptionsAction::SetHeightFocused,
            ));
        }
        if let Some(value) = self.heightUnfocused.mousePressed(mouseX, mouseY) {
            set_slider(
                &mut self.heightUnfocused,
                locale,
                "options.chat.height.unfocused",
                value,
                ChatSliderKind::Height,
            );
            return Some(slider_interaction(
                &mut self.heightUnfocused,
                value,
                ScreenChatOptionsAction::SetHeightUnfocused,
            ));
        }
        if let Some(value) = self.width.mousePressed(mouseX, mouseY) {
            set_slider(
                &mut self.width,
                locale,
                "options.chat.width",
                value,
                ChatSliderKind::Width,
            );
            return Some(slider_interaction(
                &mut self.width,
                value,
                ScreenChatOptionsAction::SetWidth,
            ));
        }
        if self.reducedDebug.mousePressed(mouseX, mouseY) {
            return Some(button_interaction(
                &self.reducedDebug,
                ScreenChatOptionsAction::ToggleReducedDebugInfo,
            ));
        }
        self.done
            .mousePressed(mouseX, mouseY)
            .then(|| button_interaction(&self.done, ScreenChatOptionsAction::Done))
    }

    pub fn mouseDragged(
        &mut self,
        mouseX: i32,
        locale: &Locale,
    ) -> Option<ScreenChatOptionsAction> {
        if let Some(value) = self.opacity.mouseDragged(mouseX) {
            set_slider(
                &mut self.opacity,
                locale,
                "options.chat.opacity",
                value,
                ChatSliderKind::Opacity,
            );
            return Some(ScreenChatOptionsAction::SetOpacity(value.clamp(0.0, 1.0)));
        }
        if let Some(value) = self.scale.mouseDragged(mouseX) {
            set_slider(
                &mut self.scale,
                locale,
                "options.chat.scale",
                value,
                ChatSliderKind::Percent,
            );
            return Some(ScreenChatOptionsAction::SetScale(value.clamp(0.0, 1.0)));
        }
        if let Some(value) = self.heightFocused.mouseDragged(mouseX) {
            set_slider(
                &mut self.heightFocused,
                locale,
                "options.chat.height.focused",
                value,
                ChatSliderKind::Height,
            );
            return Some(ScreenChatOptionsAction::SetHeightFocused(
                value.clamp(0.0, 1.0),
            ));
        }
        if let Some(value) = self.heightUnfocused.mouseDragged(mouseX) {
            set_slider(
                &mut self.heightUnfocused,
                locale,
                "options.chat.height.unfocused",
                value,
                ChatSliderKind::Height,
            );
            return Some(ScreenChatOptionsAction::SetHeightUnfocused(
                value.clamp(0.0, 1.0),
            ));
        }
        if let Some(value) = self.width.mouseDragged(mouseX) {
            set_slider(
                &mut self.width,
                locale,
                "options.chat.width",
                value,
                ChatSliderKind::Width,
            );
            return Some(ScreenChatOptionsAction::SetWidth(value.clamp(0.0, 1.0)));
        }
        None
    }

    pub fn mouseReleased(&mut self, mouseX: i32, mouseY: i32) {
        self.opacity.mouseReleased(mouseX, mouseY);
        self.scale.mouseReleased(mouseX, mouseY);
        self.heightFocused.mouseReleased(mouseX, mouseY);
        self.heightUnfocused.mouseReleased(mouseX, mouseY);
        self.width.mouseReleased(mouseX, mouseY);
    }
}

fn set_position(button: &mut GuiButton, x: i32, y: i32) {
    button.x = x;
    button.y = y;
}

fn set_slider_position(button: &mut GuiButton, x: i32, y: i32) {
    set_position(button, x, y);
}

fn button_interaction(
    button: &GuiButton,
    action: ScreenChatOptionsAction,
) -> ScreenChatOptionsInteraction {
    ScreenChatOptionsInteraction {
        action,
        sound: Some(button.playPressSound()),
    }
}

fn slider_interaction(
    slider: &mut GuiOptionSlider,
    value: f32,
    make: fn(f32) -> ScreenChatOptionsAction,
) -> ScreenChatOptionsInteraction {
    let value = value.clamp(0.0, 1.0);
    slider.setSliderValue(value);
    ScreenChatOptionsInteraction {
        action: make(value),
        sound: Some(slider.playPressSound()),
    }
}

#[derive(Clone, Copy)]
enum ChatSliderKind {
    Opacity,
    Percent,
    Height,
    Width,
}

fn set_slider(
    slider: &mut GuiOptionSlider,
    locale: &Locale,
    key: &str,
    value: f32,
    kind: ChatSliderKind,
) {
    let value = value.clamp(0.0, 1.0);
    slider.setSliderValue(value);
    let amount = match kind {
        ChatSliderKind::Opacity => format!("{}%", (value * 90.0 + 10.0) as i32),
        ChatSliderKind::Percent if value == 0.0 => locale.translate_key("options.off").to_owned(),
        ChatSliderKind::Percent => format!("{}%", (value * 100.0) as i32),
        ChatSliderKind::Height => format!("{}px", (value * 160.0 + 20.0).floor() as i32),
        ChatSliderKind::Width => format!("{}px", (value * 280.0 + 40.0).floor() as i32),
    };
    slider.setDisplayString(format!("{}: {}", locale.translate_key(key), amount));
}

fn bool_label(locale: &Locale, key: &str, value: bool) -> String {
    format!(
        "{}: {}",
        locale.translate_key(key),
        locale.translate_key(if value { "options.on" } else { "options.off" }),
    )
}

fn visibility_name(locale: &Locale, visibility: EnumChatVisibility) -> String {
    locale
        .translate_key(match visibility {
            EnumChatVisibility::Full => "options.chat.visibility.full",
            EnumChatVisibility::System => "options.chat.visibility.system",
            EnumChatVisibility::Hidden => "options.chat.visibility.hidden",
        })
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_order_places_reduced_debug_in_tenth_slot() {
        let mut screen = ScreenChatOptions::new();
        let settings = GameSettings::default();
        let locale = Locale::default();
        screen.initGui(854, 480, &locale, &settings);
        assert_eq!(screen.visibility.y, 80);
        assert_eq!(screen.opacity.GuiButton.y, 104);
        assert_eq!(screen.reducedDebug.y, 176);
        assert_eq!(screen.done.y, 224);
    }
}
