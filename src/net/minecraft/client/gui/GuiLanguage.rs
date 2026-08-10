use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::resources::Language::Language;
use crate::net::minecraft::client::resources::LanguageManager::LanguageManager;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::client::settings::GameSettings::GameSettings;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, PartialEq)]
pub enum GuiLanguageAction {
    ToggleUnicode,
    Done,
    SelectLanguage(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiLanguageInteraction {
    pub action: GuiLanguageAction,
    pub sound: Option<GuiSoundCommand>,
}

/// MCP 1.12.2 `GuiLanguage` and its inner `GuiLanguage.List`/`GuiSlot`.
///
/// The Rust representation keeps the original visible geometry and input
/// semantics: 220px list width, 18px rows, top=32, bottom=height-65+4,
/// `GuiSlot#getScrollBarX = width / 2 + 124`, the original selection border,
/// wheel/drag scrolling and the source language row bidi behavior.
#[derive(Debug, Clone)]
pub struct GuiLanguage {
    pub GuiScreen: GuiScreen,
    currentLanguage: String,
    currentLanguageBidirectional: bool,
    languages: Vec<Language>,
    scrollOffset: i32,
    initialClickY: i32,
    scrollMultiplier: f32,
}

const SLOT_HEIGHT: i32 = 18;
const LIST_TOP: i32 = 32;
const LIST_BOTTOM_INSET: i32 = 65;
const LIST_WIDTH: i32 = 220;
const SCROLLBAR_WIDTH: i32 = 6;

/// MCP `GuiSlot#drawContainerBackground`: the list viewport is a scrolling
/// `Gui.OPTIONS_BACKGROUND` quad tinted RGB 32. UVs are expressed in the same
/// 32-pixel texture scale and offset vertically by amountScrolled.
fn draw_container_background(
    drawList: &mut GuiDrawList,
    width: i32,
    top: i32,
    bottom: i32,
    amountScrolled: i32,
) {
    let texture = crate::net::minecraft::client::gui::Gui::OPTIONS_BACKGROUND.clone();
    let color = 0xFF20_2020_u32;
    let widthF = width as f32;
    let topV = (top + amountScrolled) as f32 / 32.0;
    let bottomV = (bottom + amountScrolled) as f32 / 32.0;
    drawList.push_textured_quad(texture, [
        (0.0, bottom as f32, 0.0, bottomV, color),
        (widthF, bottom as f32, widthF / 32.0, bottomV, color),
        (widthF, top as f32, widthF / 32.0, topV, color),
        (0.0, top as f32, 0.0, topV, color),
    ]);
}

/// MCP `GuiSlot#overlayBackground(startY,endY,255,255)`.
fn draw_overlay_background(
    drawList: &mut GuiDrawList,
    width: i32,
    startY: i32,
    endY: i32,
) {
    let texture = crate::net::minecraft::client::gui::Gui::OPTIONS_BACKGROUND.clone();
    let color = 0xFF40_4040_u32;
    let widthF = width as f32;
    drawList.push_textured_quad(texture, [
        (0.0, endY as f32, 0.0, endY as f32 / 32.0, color),
        (widthF, endY as f32, widthF / 32.0, endY as f32 / 32.0, color),
        (widthF, startY as f32, widthF / 32.0, startY as f32 / 32.0, color),
        (0.0, startY as f32, 0.0, startY as f32 / 32.0, color),
    ]);
}

impl GuiLanguage {
    pub fn new(currentLanguage: impl Into<String>) -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            currentLanguage: currentLanguage.into(),
            currentLanguageBidirectional: false,
            languages: Vec::new(),
            scrollOffset: 0,
            initialClickY: -2,
            scrollMultiplier: 1.0,
        }
    }

    pub fn initGui(
        &mut self,
        width: i32,
        height: i32,
        locale: &Locale,
        settings: &GameSettings,
        languageManager: &LanguageManager,
    ) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();

        let unicodeLabel = format!(
            "{}: {}",
            locale.translate_key("options.forceUnicodeFont"),
            if settings.forceUnicodeFont {
                locale.translate_key("options.on")
            } else {
                locale.translate_key("options.off")
            }
        );
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            100,
            width / 2 - 155,
            height - 38,
            150,
            20,
            unicodeLabel,
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            6,
            width / 2 - 155 + 160,
            height - 38,
            150,
            20,
            locale.translate_key("gui.done"),
        ));

        self.languages = languageManager.getLanguages().into_iter().cloned().collect();
        self.currentLanguage = languageManager.getCurrentLanguage().getLanguageCode().to_owned();
        self.currentLanguageBidirectional = languageManager.isCurrentLanguageBidirectional();
        self.clampScroll();
    }

    fn listBottom(&self) -> i32 {
        self.GuiScreen.height - LIST_BOTTOM_INSET + 4
    }

    fn listLeft(&self) -> i32 {
        self.GuiScreen.width / 2 - LIST_WIDTH / 2
    }

    fn listRight(&self) -> i32 {
        self.GuiScreen.width / 2 + LIST_WIDTH / 2
    }

    /// MCP `GuiSlot#getScrollBarX`.
    fn scrollBarX(&self) -> i32 {
        self.GuiScreen.width / 2 + 124
    }

    fn maxScroll(&self) -> i32 {
        let contentHeight = self.languages.len() as i32 * SLOT_HEIGHT;
        (contentHeight - (self.listBottom() - LIST_TOP - 4)).max(0)
    }

    fn clampScroll(&mut self) {
        self.scrollOffset = self.scrollOffset.clamp(0, self.maxScroll());
    }

    fn scrollBy(&mut self, amount: i32) {
        self.scrollOffset += amount;
        self.clampScroll();
        self.initialClickY = -2;
    }

    /// MCP `GuiSlot#handleMouseInput` wheel branch: half one slot per notch.
    pub fn scroll(&mut self, lines: f32) -> bool {
        let amount = (lines * SLOT_HEIGHT as f32 / 2.0) as i32;
        if amount == 0 {
            return false;
        }
        self.scrollBy(-amount);
        true
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
        self.drawScreenCommon(drawList, font, locale, mouseX, mouseY, partialTicks, false);
    }

    /// MCP `GuiScreen#drawDefaultBackground` depends on whether a world is
    /// loaded.  The language list itself still draws its scrolling
    /// OPTIONS_BACKGROUND viewport, but the screen behind that viewport must
    /// remain the paused world instead of the main-menu dirt background.
    pub fn drawScreenInWorld(
        &mut self,
        drawList: &mut GuiDrawList,
        font: &mut FontRenderer,
        locale: &Locale,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.drawScreenCommon(drawList, font, locale, mouseX, mouseY, partialTicks, true);
    }

    fn drawScreenCommon(
        &mut self,
        drawList: &mut GuiDrawList,
        font: &mut FontRenderer,
        locale: &Locale,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
        worldLoaded: bool,
    ) {
        // `GuiLanguage.List#drawBackground` delegates to the outer
        // GuiLanguage/GuiScreen default background.
        if worldLoaded {
            self.GuiScreen.drawDefaultBackgroundInWorld(drawList);
        } else {
            self.GuiScreen.drawDefaultBackground(drawList);
        }
        self.clampScroll();
        draw_container_background(
            drawList,
            self.GuiScreen.width,
            LIST_TOP,
            self.listBottom(),
            self.scrollOffset,
        );

        let listLeft = self.listLeft();
        let listRight = self.listRight();
        for (index, language) in self.languages.iter().enumerate() {
            let rowY = LIST_TOP + 4 - self.scrollOffset + index as i32 * SLOT_HEIGHT;
            let rowHeight = SLOT_HEIGHT - 4;
            if rowY < LIST_TOP - SLOT_HEIGHT || rowY > self.listBottom() {
                continue;
            }

            if language.getLanguageCode() == self.currentLanguage {
                // MCP `GuiSlot#func_192638_a`: outer gray selection and inner black border.
                drawList.draw_rect(
                    listLeft,
                    rowY - 2,
                    listRight,
                    rowY + rowHeight + 2,
                    0xFF80_8080_u32 as i32,
                );
                drawList.draw_rect(
                    listLeft + 1,
                    rowY - 1,
                    listRight - 1,
                    rowY + rowHeight + 1,
                    0xFF00_0000_u32 as i32,
                );
            }

            if rowY >= LIST_TOP - SLOT_HEIGHT && rowY <= self.listBottom() {
                // MCP GuiLanguage.List temporarily forces bidi for every language name.
                font.set_bidi_flag(true);
                self.GuiScreen.Gui.drawCenteredString(
                    font,
                    drawList,
                    &language.to_string(),
                    self.GuiScreen.width / 2,
                    rowY + 1,
                    0x00FF_FFFF,
                );
                font.set_bidi_flag(self.currentLanguageBidirectional);
            }
        }

        // MCP `GuiSlot#overlayBackground`: redraw the dirt/options texture
        // outside the slot viewport at RGB 64, then draw the 4px black fades.
        draw_overlay_background(drawList, self.GuiScreen.width, 0, LIST_TOP);
        draw_overlay_background(
            drawList,
            self.GuiScreen.width,
            self.listBottom(),
            self.GuiScreen.height,
        );
        drawList.draw_gradient_rect(
            0, LIST_TOP, self.GuiScreen.width, LIST_TOP + 4,
            0xFF00_0000_u32 as i32, 0x0000_0000,
        );
        drawList.draw_gradient_rect(
            0, self.listBottom() - 4, self.GuiScreen.width, self.listBottom(),
            0x0000_0000, 0xFF00_0000_u32 as i32,
        );

        let maxScroll = self.maxScroll();
        if maxScroll > 0 {
            let trackX = self.scrollBarX();
            let trackRight = trackX + SCROLLBAR_WIDTH;
            let contentHeight = (self.languages.len() as i32 * SLOT_HEIGHT).max(1);
            let viewport = self.listBottom() - LIST_TOP;
            let mut thumbHeight = viewport * viewport / contentHeight;
            thumbHeight = thumbHeight.clamp(32, viewport - 8);
            let mut thumbY = self.scrollOffset * (viewport - thumbHeight) / maxScroll + LIST_TOP;
            if thumbY < LIST_TOP {
                thumbY = LIST_TOP;
            }
            drawList.draw_rect(trackX, LIST_TOP, trackRight, self.listBottom(), 0xFF00_0000_u32 as i32);
            drawList.draw_rect(trackX, thumbY, trackRight, thumbY + thumbHeight, 0xFF80_8080_u32 as i32);
            drawList.draw_rect(trackX, thumbY, trackRight - 1, thumbY + thumbHeight - 1, 0xFFC0_C0C0_u32 as i32);
        }

        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            locale.translate_key("options.language"),
            self.GuiScreen.width / 2,
            16,
            0x00FF_FFFF,
        );
        let warning = format!("({})", locale.translate_key("options.languageWarning"));
        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            &warning,
            self.GuiScreen.width / 2,
            self.GuiScreen.height - 56,
            0x0080_8080,
        );
        self.GuiScreen.drawScreen(drawList, font, mouseX, mouseY, partialTicks);
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
    ) -> Option<GuiLanguageInteraction> {
        if mouseButton != 0 {
            return None;
        }

        if mouseY >= LIST_TOP && mouseY <= self.listBottom() {
            let k = mouseY - LIST_TOP + self.scrollOffset - 4;
            let index = k / SLOT_HEIGHT;
            let rowHit = mouseX < self.scrollBarX()
                && mouseX >= self.listLeft()
                && mouseX <= self.listRight()
                && index >= 0
                && k >= 0
                && (index as usize) < self.languages.len();

            let interaction = if rowHit {
                self.languages.get(index as usize).map(|language| GuiLanguageInteraction {
                    action: GuiLanguageAction::SelectLanguage(language.getLanguageCode().to_owned()),
                    // `GuiSlot#elementClicked` does not play the button press sound.
                    sound: None,
                })
            } else {
                None
            };

            let scrollX = self.scrollBarX();
            if mouseX >= scrollX && mouseX <= scrollX + SCROLLBAR_WIDTH {
                let maxScroll = self.maxScroll().max(1);
                let contentHeight = (self.languages.len() as i32 * SLOT_HEIGHT).max(1);
                let viewport = self.listBottom() - LIST_TOP;
                let mut thumbHeight = viewport * viewport / contentHeight;
                thumbHeight = thumbHeight.clamp(32, viewport - 8);
                self.scrollMultiplier = -1.0 / ((viewport - thumbHeight) as f32 / maxScroll as f32);
            } else {
                self.scrollMultiplier = 1.0;
            }
            self.initialClickY = mouseY;

            if interaction.is_some() {
                return interaction;
            }
        } else {
            self.initialClickY = -2;
        }

        self.GuiScreen.buttonList.iter().find_map(|button| {
            if !button.mousePressed(mouseX, mouseY) {
                return None;
            }
            let action = match button.id {
                100 => GuiLanguageAction::ToggleUnicode,
                6 => GuiLanguageAction::Done,
                _ => return None,
            };
            Some(GuiLanguageInteraction {
                action,
                sound: Some(button.playPressSound()),
            })
        })
    }

    pub fn mouseDragged(&mut self, mouseY: i32) -> bool {
        if self.initialClickY < 0 {
            return false;
        }
        self.scrollOffset -= ((mouseY - self.initialClickY) as f32 * self.scrollMultiplier) as i32;
        self.initialClickY = mouseY;
        self.clampScroll();
        true
    }

    pub fn mouseReleased(&mut self) {
        self.initialClickY = -1;
    }
}
