use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::client::settings::GameSettings::GameSettings;
use crate::net::minecraft::entity::player::EnumPlayerModelParts::EnumPlayerModelParts;
use crate::net::minecraft::util::EnumHandSide::EnumHandSide;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiCustomizeSkinAction {
    TogglePart(EnumPlayerModelParts),
    ToggleMainHand,
    Done,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiCustomizeSkinInteraction {
    pub action: GuiCustomizeSkinAction,
    pub sound: GuiSoundCommand,
}

/// MCP 1.12.2 `GuiCustomizeSkin`, retaining the seven-part enumeration order,
/// main-hand option position and Done-row parity adjustment.
#[derive(Debug, Clone)]
pub struct GuiCustomizeSkin {
    pub GuiScreen: GuiScreen,
    title: String,
    partButtons: Vec<(EnumPlayerModelParts, GuiButton)>,
    hand: GuiButton,
    done: GuiButton,
}

impl Default for GuiCustomizeSkin {
    fn default() -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            title: "Skin Customization".to_owned(),
            partButtons: EnumPlayerModelParts::VALUES
                .into_iter()
                .map(|part| {
                    (
                        part,
                        GuiButton::newWithSize(part.getPartId(), 0, 0, 150, 20, ""),
                    )
                })
                .collect(),
            hand: GuiButton::newWithSize(199, 0, 0, 150, 20, ""),
            done: GuiButton::new(200, 0, 0, "Done"),
        }
    }
}

impl GuiCustomizeSkin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initGui(&mut self, width: i32, height: i32, locale: &Locale, settings: &GameSettings) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.title = locale
            .translate_key("options.skinCustomisation.title")
            .to_owned();
        let top = height / 6;
        let mut index = 0usize;
        for (part, button) in &mut self.partButtons {
            button.x = width / 2 - 155 + (index % 2) as i32 * 160;
            button.y = top + (index / 2) as i32 * 24;
            button.displayString = part_label(
                locale,
                *part,
                settings.modelPartFlags & part.getPartMask() != 0,
            );
            index += 1;
        }
        self.hand.x = width / 2 - 155 + (index % 2) as i32 * 160;
        self.hand.y = top + (index / 2) as i32 * 24;
        self.hand.displayString = format!(
            "{}: {}",
            locale.translate_key("options.mainHand"),
            locale.translate_key(match settings.mainHand {
                EnumHandSide::Left => "options.mainHand.left",
                EnumHandSide::Right => "options.mainHand.right",
            }),
        );
        index += 1;
        if index % 2 == 1 {
            index += 1;
        }
        self.done.x = width / 2 - 100;
        self.done.y = top + (index / 2) as i32 * 24;
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
        for (_, button) in &mut self.partButtons {
            button.drawButton(draw, font, mouseX, mouseY, partialTicks);
        }
        self.hand
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
        self.done
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
    }

    pub fn mouseClicked(
        &self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
    ) -> Option<GuiCustomizeSkinInteraction> {
        if mouseButton != 0 {
            return None;
        }
        for (part, button) in &self.partButtons {
            if button.mousePressed(mouseX, mouseY) {
                return Some(GuiCustomizeSkinInteraction {
                    action: GuiCustomizeSkinAction::TogglePart(*part),
                    sound: button.playPressSound(),
                });
            }
        }
        if self.hand.mousePressed(mouseX, mouseY) {
            return Some(GuiCustomizeSkinInteraction {
                action: GuiCustomizeSkinAction::ToggleMainHand,
                sound: self.hand.playPressSound(),
            });
        }
        self.done
            .mousePressed(mouseX, mouseY)
            .then(|| GuiCustomizeSkinInteraction {
                action: GuiCustomizeSkinAction::Done,
                sound: self.done.playPressSound(),
            })
    }
}

fn part_label(locale: &Locale, part: EnumPlayerModelParts, enabled: bool) -> String {
    format!(
        "{}: {}",
        locale.translate_key(&format!("options.modelPart.{}", part.getPartName())),
        locale.translate_key(if enabled { "options.on" } else { "options.off" }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_layout_places_main_hand_after_seventh_part() {
        let mut screen = GuiCustomizeSkin::new();
        screen.initGui(854, 480, &Locale::default(), &GameSettings::default());
        assert_eq!(screen.hand.x, 432);
        assert_eq!(screen.hand.y, 152);
        assert_eq!(screen.done.y, 176);
    }
}
