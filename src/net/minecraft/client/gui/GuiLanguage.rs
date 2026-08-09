use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::resources::Language::Language;
use crate::net::minecraft::client::resources::LanguageManager::LanguageManager;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::client::settings::GameSettings::GameSettings;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, PartialEq)]
pub enum GuiLanguageAction { ToggleUnicode, Done, SelectLanguage(String) }
#[derive(Debug, Clone, PartialEq)]
pub struct GuiLanguageInteraction { pub action: GuiLanguageAction, pub sound: Option<GuiSoundCommand> }

/// MCP `GuiLanguage`. The Java class's inner `List` (a `GuiSlot`) is ported
/// inline, preserving the GuiSlot layout and interaction:
/// - rows are 18px tall between y=32 and height-65+4; the row content is
///   inset 4px from the top and the slot hit-test shares that -4 inset;
/// - the list spans the full screen width, the scrollbar is the rightmost
///   6px and only appears when content overflows;
/// - the thumb is `(bottom-top)^2 / contentHeight`, clamped to
///   [32, bottom-top-8], and one wheel notch scrolls half a slot;
/// - dragging anywhere inside the list scrolls it (initialClickY tracks the
///   drag start; -1 idle, -2 disabled, >=0 dragging), matching
///   `GuiSlot#handleMouseInput`/`mouseReleased`.
#[derive(Debug, Clone)]
pub struct GuiLanguage {
    pub GuiScreen: GuiScreen,
    currentLanguage: String,
    languages: Vec<Language>,
    scrollOffset: i32,
    initialClickY: i32,
    scrollMultiplier: f32,
}

const SLOT_HEIGHT: i32 = 18;
const LIST_TOP: i32 = 32;
const LIST_BOTTOM_INSET: i32 = 65;
const SCROLLBAR_WIDTH: i32 = 6;

impl GuiLanguage {
    pub fn new(currentLanguage: impl Into<String>) -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            currentLanguage: currentLanguage.into(),
            languages: Vec::new(),
            scrollOffset: 0,
            initialClickY: -1,
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
            if settings.forceUnicodeFont { locale.translate_key("options.on") } else { locale.translate_key("options.off") }
        );
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(100, width / 2 - 155, height - 38, 150, 20, unicodeLabel));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(6, width / 2 + 5, height - 38, 150, 20, locale.translate_key("gui.done")));
        // MCP `GuiLanguage.List` constructor: snapshot the sorted languages
        // and the current selection. The scroll offset survives re-inits so a
        // language switch does not jump the list (vanilla keeps the slot's
        // amountScrolled across elementClicked).
        self.languages = languageManager.getLanguages().into_iter().cloned().collect();
        self.currentLanguage = languageManager.getCurrentLanguage().getLanguageCode().to_owned();
    }

    fn listBottom(&self) -> i32 {
        self.GuiScreen.height - LIST_BOTTOM_INSET + 4
    }

    /// MCP `GuiSlot#getScrollBarX` is `right - 6` (the list spans the screen);
    /// the project owner requested the scrollbar sit 150px further left.
    fn scrollBarX(&self) -> i32 {
        self.GuiScreen.width - SCROLLBAR_WIDTH - 150
    }

    /// MCP `GuiSlot#getMaxScroll`: `max(0, contentHeight - (bottom - top - 4))`.
    fn maxScroll(&self) -> i32 {
        let contentHeight = self.languages.len() as i32 * SLOT_HEIGHT;
        (contentHeight - (self.listBottom() - LIST_TOP - 4)).max(0)
    }

    /// MCP `GuiSlot#bindAmountScrolled`.
    fn clampScroll(&mut self) {
        self.scrollOffset = self.scrollOffset.clamp(0, self.maxScroll());
    }

    /// MCP `GuiSlot#scrollBy`: also disables dragging until the button is
    /// released (the wheel path in `handleMouseInput`).
    fn scrollBy(&mut self, amount: i32) {
        self.scrollOffset += amount;
        self.clampScroll();
        self.initialClickY = -2;
    }

    /// MCP `GuiSlot#handleMouseInput` wheel handling: one notch is half a slot.
    pub fn scroll(&mut self, lines: f32) -> bool {
        let amount = (lines * SLOT_HEIGHT as f32 / 2.0) as i32;
        if amount == 0 { return false; }
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
        // MCP GuiSlot.drawScreen: background first (the List overrides
        // drawBackground with GuiLanguage.drawDefaultBackground), then the
        // scrollbar, rows and selection highlight, then the screen's own
        // title, warning and buttons.
        self.GuiScreen.drawDefaultBackground(drawList);
        let maxScroll = self.maxScroll();
        let (trackX, trackRight) = (self.scrollBarX(), self.scrollBarX() + SCROLLBAR_WIDTH);
        if maxScroll > 0 {
            drawList.draw_rect(trackX, LIST_TOP, trackRight, self.listBottom(), 0xFF00_0000_u32 as i32);
        }
        // MCP `GuiSlot#func_192638_a`: every row at
        // `top + 4 - amountScrolled + i * slotHeight`; rows outside the
        // viewport are skipped. The selected row gets a full-width
        // translucent highlight spanning k-2 .. k+slotHeight-2.
        for (index, language) in self.languages.iter().enumerate() {
            let rowY = LIST_TOP + 4 - self.scrollOffset + index as i32 * SLOT_HEIGHT;
            let rowHeight = SLOT_HEIGHT - 4;
            if rowY + rowHeight < LIST_TOP || rowY > self.listBottom() {
                continue;
            }
            if language.getLanguageCode() == self.currentLanguage {
                drawList.draw_rect(0, rowY - 2, self.GuiScreen.width, rowY + rowHeight + 2, 0x8080_8080_u32 as i32);
            }
            // MCP `GuiLanguage.List#func_192637_a`: "name (region)" centered,
            // one pixel below the row top.
            self.GuiScreen.Gui.drawCenteredString(font, drawList, &language.to_string(), self.GuiScreen.width / 2, rowY + 1, 0x00FF_FFFF);
        }
        if maxScroll > 0 {
            // MCP GuiSlot.drawScreen scrollbar: dark track, grey thumb
            // clamped to [32, bottom-top-8], light 1px inner edge.
            let contentHeight = self.languages.len() as i32 * SLOT_HEIGHT;
            let mut thumbHeight = (self.listBottom() - LIST_TOP) * (self.listBottom() - LIST_TOP) / contentHeight;
            thumbHeight = thumbHeight.clamp(32, self.listBottom() - LIST_TOP - 8);
            let mut thumbY = self.scrollOffset * (self.listBottom() - LIST_TOP - thumbHeight) / maxScroll + LIST_TOP;
            if thumbY < LIST_TOP {
                thumbY = LIST_TOP;
            }
            drawList.draw_rect(trackX, thumbY, trackRight, thumbY + thumbHeight, 0xFF80_8080_u32 as i32);
            drawList.draw_rect(trackX, thumbY, trackRight - 1, thumbY + thumbHeight - 1, 0xFFC0_C0C0_u32 as i32);
        }
        self.GuiScreen.Gui.drawCenteredString(font, drawList, locale.translate_key("options.language"), self.GuiScreen.width / 2, 16, 0x00FF_FFFF);
        let warning = format!("({})", locale.translate_key("options.languageWarning"));
        self.GuiScreen.Gui.drawCenteredString(font, drawList, &warning, self.GuiScreen.width / 2, self.GuiScreen.height - 56, 0x0080_8080);
        self.GuiScreen.drawScreen(drawList, font, mouseX, mouseY, partialTicks);
    }

    /// MCP `GuiSlot#handleMouseInput` press handling. Like the vanilla
    /// method, one press does all three in order: the full-width row hit-test
    /// (elementClicked), then arming the drag multiplier for the scrollbar
    /// (thumb follows the cursor 1:1) or plain list dragging, then arming
    /// `initialClickY` so the subsequent `mouseDragged` scrolls.
    pub fn mouseClicked(&mut self, mouseX: i32, mouseY: i32, mouseButton: i32) -> Option<GuiLanguageInteraction> {
        if mouseButton != 0 { return None; }
        if mouseY >= LIST_TOP && mouseY <= self.listBottom() {
            // MCP GuiSlot slot hit-test: `(y - top + amountScrolled - 4) / slotHeight`.
            let hitIndex = ((mouseY - LIST_TOP + self.scrollOffset - 4) / SLOT_HEIGHT).max(0) as usize;
            let interaction = self.languages.get(hitIndex).map(|language| GuiLanguageInteraction {
                // GuiSlot#elementClicked plays no sound.
                action: GuiLanguageAction::SelectLanguage(language.getLanguageCode().to_owned()),
                sound: None,
            });
            if mouseX >= self.scrollBarX() && mouseX <= self.scrollBarX() + SCROLLBAR_WIDTH {
                // MCP: pressing the scrollbar scales the drag multiplier so
                // one pixel of thumb travel maps to one scroll unit.
                let mut maxScroll = self.maxScroll();
                if maxScroll < 1 { maxScroll = 1; }
                let mut thumbHeight = (self.listBottom() - LIST_TOP) * (self.listBottom() - LIST_TOP) / (self.languages.len() as i32 * SLOT_HEIGHT);
                thumbHeight = thumbHeight.clamp(32, self.listBottom() - LIST_TOP - 8);
                self.scrollMultiplier = -1.0 / ((self.listBottom() - LIST_TOP - thumbHeight) as f32 / maxScroll as f32);
            } else {
                self.scrollMultiplier = 1.0;
            }
            self.initialClickY = mouseY;
            return interaction;
        }
        self.initialClickY = -2;
        self.GuiScreen.buttonList.iter().find_map(|button| {
            if !button.mousePressed(mouseX, mouseY) { return None; }
            let action = match button.id {
                100 => GuiLanguageAction::ToggleUnicode,
                6 => GuiLanguageAction::Done,
                _ => return None,
            };
            Some(GuiLanguageInteraction { action, sound: Some(button.playPressSound()) })
        })
    }

    /// MCP `GuiSlot#handleMouseInput` drag handling: while the button is held
    /// and `initialClickY` is armed, scroll by the mouse delta times the
    /// armed multiplier.
    pub fn mouseDragged(&mut self, mouseY: i32) -> bool {
        if self.initialClickY < 0 {
            return false;
        }
        self.scrollOffset -= ((mouseY - self.initialClickY) as f32 * self.scrollMultiplier) as i32;
        self.initialClickY = mouseY;
        self.clampScroll();
        true
    }

    /// MCP `GuiSlot#mouseReleased`: re-arm the drag for the next press.
    pub fn mouseReleased(&mut self) {
        self.initialClickY = -1;
    }
}
