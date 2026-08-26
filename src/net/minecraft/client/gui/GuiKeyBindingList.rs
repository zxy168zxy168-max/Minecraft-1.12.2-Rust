use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::client::settings::GameSettings::GameSettings;
use crate::net::minecraft::client::settings::InputKeyCodes::display_name;
use crate::net::minecraft::client::settings::KeyBinding::{category_order, KeyBindingId};
use crate::vulkan::GuiDrawList::GuiDrawList;

const LIST_TOP: i32 = 63;
const LIST_BOTTOM_MARGIN: i32 = 32;
const SLOT_HEIGHT: i32 = 20;

#[derive(Debug, Clone)]
enum GuiKeyBindingListEntry {
    Category(String),
    Binding(KeyBindingId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiKeyBindingListAction {
    None,
    Select(KeyBindingId),
    Reset(KeyBindingId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiKeyBindingListInteraction {
    pub action: GuiKeyBindingListAction,
    pub sound: Option<GuiSoundCommand>,
}

/// MCP 1.12.2 `GuiKeyBindingList` semantic port.
///
/// Java keeps a direct reference to the owning `GuiControls`. Rust passes the
/// currently selected binding into draw calls and reports row actions back to
/// the owner instead, preserving the original class responsibility without a
/// self-referential object graph.
#[derive(Debug, Clone, Default)]
pub struct GuiKeyBindingList {
    /// Real current-screen width (`GuiControls.width`). Category rows use it.
    screenWidth: i32,
    /// MCP GuiSlot width. GuiKeyBindingList passes `controls.width + 45` to super.
    width: i32,
    height: i32,
    listEntries: Vec<GuiKeyBindingListEntry>,
    maxListLabelWidth: i32,
    amountScrolled: i32,
    draggingScrollBar: bool,
    lastDragY: i32,
}

impl GuiKeyBindingList {
    pub fn new(
        width: i32,
        height: i32,
        locale: &Locale,
        settings: &GameSettings,
        font: &FontRenderer,
    ) -> Self {
        let mut this = Self::default();
        this.initGui(width, height, locale, settings, font);
        this
    }

    pub fn initGui(
        &mut self,
        width: i32,
        height: i32,
        locale: &Locale,
        settings: &GameSettings,
        font: &FontRenderer,
    ) {
        self.screenWidth = width;
        self.width = width + 45;
        self.height = height;
        self.rebuildEntries(locale, settings, font);
        self.bindAmountScrolled();
    }

    fn rebuildEntries(&mut self, locale: &Locale, settings: &GameSettings, font: &FontRenderer) {
        // MCP clones GameSettings.keyBindings and sorts the clone before
        // inserting category entries. Keep the authoritative binding table in
        // GameSettings and store only stable KeyBindingId references here.
        let mut ids = KeyBindingId::ALL.to_vec();
        ids.sort_by(|a, b| {
            let left = settings.keyBinding(*a);
            let right = settings.keyBinding(*b);
            category_order(&left.keyCategory)
                .cmp(&category_order(&right.keyCategory))
                .then_with(|| {
                    locale
                        .translate_key(&left.keyDescription)
                        .cmp(locale.translate_key(&right.keyDescription))
                })
        });

        self.listEntries.clear();
        self.maxListLabelWidth = 0;
        let mut category: Option<String> = None;
        for id in ids {
            let binding = settings.keyBinding(id);
            if category.as_deref() != Some(binding.keyCategory.as_str()) {
                category = Some(binding.keyCategory.clone());
                self.listEntries.push(GuiKeyBindingListEntry::Category(
                    binding.keyCategory.clone(),
                ));
            }
            let label = locale.translate_key(&binding.keyDescription);
            self.maxListLabelWidth = self.maxListLabelWidth.max(font.get_string_width(label));
            self.listEntries.push(GuiKeyBindingListEntry::Binding(id));
        }
    }

    pub fn drawScreen(
        &self,
        draw: &mut GuiDrawList,
        font: &mut FontRenderer,
        locale: &Locale,
        settings: &GameSettings,
        selectedBinding: Option<KeyBindingId>,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        let bottom = self.bottom();
        // GuiSlot: left + width/2 - getListWidth()/2 + 2, where the
        // GuiKeyBindingList constructor supplies width=controls.width+45 and
        // getListWidth() is 220+32. Keep the inherited width exactly; it is
        // deliberately 45 px wider than the Controls screen.
        let rowLeft = self.width / 2 - 124;

        for (index, entry) in self.listEntries.iter().enumerate() {
            let y = LIST_TOP + 4 + index as i32 * SLOT_HEIGHT - self.amountScrolled;
            if y + SLOT_HEIGHT <= LIST_TOP || y >= bottom {
                continue;
            }
            match entry {
                GuiKeyBindingListEntry::Category(category) => {
                    let label = locale.translate_key(category);
                    font.draw_centered_string_with_shadow(
                        draw,
                        label,
                        self.screenWidth / 2,
                        y + 6,
                        0x00FF_FFFF,
                    );
                }
                GuiKeyBindingListEntry::Binding(id) => {
                    let binding = settings.keyBinding(*id);
                    let desc = locale.translate_key(&binding.keyDescription);
                    font.draw_string_with_shadow(
                        draw,
                        desc,
                        (rowLeft + 90 - self.maxListLabelWidth) as f32,
                        (y + (SLOT_HEIGHT - 4) / 2 - font.font_height / 2) as f32,
                        0x00FF_FFFF,
                    );

                    let conflict =
                        binding.keyCode != 0
                            && settings.keyBindings.iter().enumerate().any(
                                |(otherIndex, other)| {
                                    otherIndex != id.index() && other.keyCode == binding.keyCode
                                },
                            );
                    let mut display = keyDisplayString(locale, binding.keyCode);
                    if selectedBinding == Some(*id) {
                        display = format!("§f> §e{display}§f <");
                    } else if conflict {
                        display = format!("§c{display}");
                    }

                    let mut change = GuiButton::newWithSize(
                        1000 + id.index() as i32,
                        rowLeft + 105,
                        y,
                        75,
                        20,
                        display,
                    );
                    change.drawButton(draw, font, mouseX, mouseY, partialTicks);
                    let mut reset = GuiButton::newWithSize(
                        2000 + id.index() as i32,
                        rowLeft + 190,
                        y,
                        50,
                        20,
                        translatedOr(locale, "controls.reset", "Reset"),
                    );
                    reset.enabled = !binding.isDefault();
                    reset.drawButton(draw, font, mouseX, mouseY, partialTicks);
                }
            }
        }

        // GuiSlot masks entries outside [top,bottom] and renders the same
        // four-pixel fades. GuiKeyBindingList moves the inherited scrollbar
        // from width/2+124 to width/2+139.
        draw.draw_rect(0, 0, self.width, LIST_TOP, 0xFF20_2020_u32 as i32);
        draw.draw_rect(0, bottom, self.width, self.height, 0xFF20_2020_u32 as i32);
        draw.draw_gradient_rect(
            0,
            LIST_TOP,
            self.width,
            LIST_TOP + 4,
            0xFF00_0000_u32 as i32,
            0x0100_0000,
        );
        draw.draw_gradient_rect(
            0,
            bottom - 4,
            self.width,
            bottom,
            0x0100_0000,
            0xFF00_0000_u32 as i32,
        );
        self.drawScrollBar(draw);
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        settings: &GameSettings,
    ) -> Option<GuiKeyBindingListInteraction> {
        let bottom = self.bottom();
        let scrollBarLeft = self.getScrollBarX();
        if mouseX >= scrollBarLeft
            && mouseX <= scrollBarLeft + 6
            && mouseY >= LIST_TOP
            && mouseY <= bottom
        {
            self.draggingScrollBar = self.getMaxScroll() > 0;
            self.lastDragY = mouseY;
            return Some(GuiKeyBindingListInteraction {
                action: GuiKeyBindingListAction::None,
                sound: None,
            });
        }

        if mouseY < LIST_TOP || mouseY >= bottom {
            return None;
        }
        let relativeY = mouseY - LIST_TOP + self.amountScrolled - 4;
        if relativeY < 0 {
            return None;
        }
        let rowIndex = relativeY / SLOT_HEIGHT;
        let GuiKeyBindingListEntry::Binding(id) = self.listEntries.get(rowIndex as usize)? else {
            return None;
        };
        let rowLeft = self.width / 2 - 124;
        let rowY = LIST_TOP + 4 + rowIndex * SLOT_HEIGHT - self.amountScrolled;

        let change = GuiButton::newWithSize(0, rowLeft + 105, rowY, 75, 20, "");
        if change.mousePressed(mouseX, mouseY) {
            return Some(GuiKeyBindingListInteraction {
                action: GuiKeyBindingListAction::Select(*id),
                sound: Some(change.playPressSound()),
            });
        }

        let mut reset = GuiButton::newWithSize(0, rowLeft + 190, rowY, 50, 20, "");
        reset.enabled = !settings.keyBinding(*id).isDefault();
        if reset.mousePressed(mouseX, mouseY) {
            return Some(GuiKeyBindingListInteraction {
                action: GuiKeyBindingListAction::Reset(*id),
                sound: Some(reset.playPressSound()),
            });
        }
        None
    }

    pub fn mouseDragged(&mut self, mouseY: i32) -> bool {
        if !self.draggingScrollBar {
            return false;
        }
        let delta = mouseY - self.lastDragY;
        self.lastDragY = mouseY;
        if delta != 0 {
            if let Some((_, _, thumbHeight, travel, maxScroll)) = self.scrollbarGeometry() {
                if travel > 0 {
                    // This is algebraically identical to GuiSlot's negative
                    // scrollbar scrollMultiplier applied to the mouse delta.
                    self.amountScrolled +=
                        ((delta as f32 * maxScroll as f32) / travel as f32) as i32;
                    self.bindAmountScrolled();
                }
                debug_assert!(thumbHeight >= 32 || self.bottom() - LIST_TOP < 40);
            }
        }
        true
    }

    pub fn mouseReleased(&mut self) {
        self.draggingScrollBar = false;
    }

    pub fn handleMouseWheel(&mut self, lines: f32) -> bool {
        if lines == 0.0 {
            return false;
        }
        let old = self.amountScrolled;
        self.amountScrolled -= (lines.signum() * SLOT_HEIGHT as f32 / 2.0) as i32;
        self.bindAmountScrolled();
        old != self.amountScrolled
    }

    pub fn getListWidth(&self) -> i32 {
        220 + 32
    }
    pub fn getScrollBarX(&self) -> i32 {
        self.width / 2 + 124 + 15
    }
    pub fn getMaxScroll(&self) -> i32 {
        let visible = (self.bottom() - LIST_TOP - 4).max(0);
        (self.getContentHeight() - visible).max(0)
    }

    fn getContentHeight(&self) -> i32 {
        self.listEntries.len() as i32 * SLOT_HEIGHT
    }
    fn bottom(&self) -> i32 {
        self.height - LIST_BOTTOM_MARGIN
    }
    fn bindAmountScrolled(&mut self) {
        self.amountScrolled = self.amountScrolled.clamp(0, self.getMaxScroll());
    }

    fn scrollbarGeometry(&self) -> Option<(i32, i32, i32, i32, i32)> {
        let maxScroll = self.getMaxScroll();
        if maxScroll <= 0 {
            return None;
        }
        let bottom = self.bottom();
        let viewport = bottom - LIST_TOP;
        if viewport <= 8 {
            return None;
        }
        let content = self.getContentHeight().max(1);
        let thumbHeight = (viewport * viewport / content).clamp(32, viewport - 8);
        let travel = (viewport - thumbHeight).max(1);
        let thumbY = (LIST_TOP + self.amountScrolled * travel / maxScroll).max(LIST_TOP);
        Some((self.getScrollBarX(), thumbY, thumbHeight, travel, maxScroll))
    }

    fn drawScrollBar(&self, draw: &mut GuiDrawList) {
        let Some((left, thumbY, thumbHeight, _, _)) = self.scrollbarGeometry() else {
            return;
        };
        let right = left + 6;
        draw.draw_rect(left, LIST_TOP, right, self.bottom(), 0xFF00_0000_u32 as i32);
        draw.draw_rect(
            left,
            thumbY,
            right,
            thumbY + thumbHeight,
            0xFF80_8080_u32 as i32,
        );
        draw.draw_rect(
            left,
            thumbY,
            right - 1,
            thumbY + thumbHeight - 1,
            0xFFC0_C0C0_u32 as i32,
        );
    }
}

fn translatedOr(locale: &Locale, key: &str, fallback: &str) -> String {
    let value = locale.translate_key(key);
    if value == key {
        fallback.to_owned()
    } else {
        value.to_owned()
    }
}

fn keyDisplayString(locale: &Locale, code: i32) -> String {
    if code == -100 {
        return translatedOr(locale, "key.mouse.left", "Button 1");
    }
    if code == -99 {
        return translatedOr(locale, "key.mouse.right", "Button 2");
    }
    if code == -98 {
        return translatedOr(locale, "key.mouse.middle", "Button 3");
    }
    if code < 0 {
        let template = translatedOr(locale, "key.mouseButton", "Button %1$s");
        let number = (code + 101).to_string();
        // Minecraft language files use Java Formatter's indexed `%1$s` token;
        // tolerate `%s` as well for third-party language packs.
        if template.contains("%1$s") {
            return template.replacen("%1$s", &number, 1);
        }
        return template.replacen("%s", &number, 1);
    }
    display_name(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_list_geometry_uses_key_binding_overrides() {
        let list = GuiKeyBindingList {
            screenWidth: 854,
            width: 854 + 45,
            height: 480,
            ..Default::default()
        };
        assert_eq!(list.getListWidth(), 252);
        assert_eq!(list.getScrollBarX(), (854 + 45) / 2 + 139);
        assert_eq!(list.width / 2 - 124, (854 + 45) / 2 - 124);
    }
}
