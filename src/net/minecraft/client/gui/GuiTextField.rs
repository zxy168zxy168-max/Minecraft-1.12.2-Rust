use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiTextFieldKey {
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GuiTextFieldModifiers {
    pub control: bool,
    pub shift: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiTextFieldRenderState {
    pub text: String,
    pub textX: i32,
    pub textY: i32,
    pub color: i32,
    pub cursorX: i32,
    pub cursorVisible: bool,
    pub cursorBlock: bool,
    pub selectionX: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct GuiTextField {
    pub id: i32,
    pub xPosition: i32,
    pub yPosition: i32,
    pub width: i32,
    pub height: i32,
    text: Vec<u16>,
    maxStringLength: usize,
    cursorCounter: i32,
    enableBackgroundDrawing: bool,
    canLoseFocus: bool,
    isFocused: bool,
    isEnabled: bool,
    lineScrollOffset: usize,
    cursorPosition: usize,
    selectionEnd: usize,
    enabledColor: i32,
    disabledColor: i32,
    visible: bool,
    validator: fn(&str) -> bool,
    maskCharacter: Option<char>,
}

impl GuiTextField {
    pub fn new(id: i32, x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            id,
            xPosition: x,
            yPosition: y,
            width,
            height,
            text: Vec::new(),
            maxStringLength: 32,
            cursorCounter: 0,
            enableBackgroundDrawing: true,
            canLoseFocus: true,
            isFocused: false,
            isEnabled: true,
            lineScrollOffset: 0,
            cursorPosition: 0,
            selectionEnd: 0,
            enabledColor: 14_737_632,
            disabledColor: 7_368_816,
            visible: true,
            validator: |_| true,
            maskCharacter: None,
        }
    }

    pub fn updateCursorCounter(&mut self) {
        self.cursorCounter = self.cursorCounter.wrapping_add(1);
    }
    pub fn setText(&mut self, textIn: &str) {
        if !(self.validator)(textIn) {
            return;
        }
        self.text = textIn.encode_utf16().take(self.maxStringLength).collect();
        self.setCursorPositionEnd(None);
    }
    pub fn getText(&self) -> String {
        String::from_utf16_lossy(&self.text)
    }
    pub fn getSelectedText(&self) -> String {
        let start = self.cursorPosition.min(self.selectionEnd);
        let end = self.cursorPosition.max(self.selectionEnd);
        String::from_utf16_lossy(&self.text[start..end])
    }
    pub fn setValidator(&mut self, validator: fn(&str) -> bool) {
        self.validator = validator;
    }

    pub fn writeText(&mut self, textToWrite: &str, font: Option<&FontRenderer>) -> bool {
        if !self.isEnabled {
            return false;
        }
        let filtered: Vec<u16> = textToWrite
            .chars()
            .filter(|character| isAllowedCharacter(*character))
            .flat_map(|character| {
                let mut buffer = [0_u16; 2];
                character.encode_utf16(&mut buffer).to_vec()
            })
            .collect();
        let start = self.cursorPosition.min(self.selectionEnd);
        let end = self.cursorPosition.max(self.selectionEnd);
        let remaining = self
            .maxStringLength
            .saturating_sub(self.text.len() - (end - start));
        let inserted = filtered.len().min(remaining);
        let mut candidate = Vec::with_capacity(self.text.len() - (end - start) + inserted);
        candidate.extend_from_slice(&self.text[..start]);
        candidate.extend_from_slice(&filtered[..inserted]);
        candidate.extend_from_slice(&self.text[end..]);
        let candidateString = String::from_utf16_lossy(&candidate);
        if !(self.validator)(&candidateString) {
            return false;
        }
        self.text = candidate;
        self.cursorPosition = start + inserted;
        self.setSelectionPos(self.cursorPosition, font);
        true
    }

    pub fn deleteWords(&mut self, num: i32, font: Option<&FontRenderer>) -> bool {
        if self.text.is_empty() {
            return false;
        }
        if self.selectionEnd != self.cursorPosition {
            return self.writeText("", font);
        }
        let target = self.getNthWordFromCursor(num);
        self.deleteFromCursor(target as i32 - self.cursorPosition as i32, font)
    }

    pub fn deleteFromCursor(&mut self, num: i32, font: Option<&FontRenderer>) -> bool {
        if !self.isEnabled || self.text.is_empty() {
            return false;
        }
        if self.selectionEnd != self.cursorPosition {
            return self.writeText("", font);
        }
        let (start, end) = if num < 0 {
            (
                self.cursorPosition
                    .saturating_sub(num.unsigned_abs() as usize),
                self.cursorPosition,
            )
        } else {
            (
                self.cursorPosition,
                (self.cursorPosition + num as usize).min(self.text.len()),
            )
        };
        if start == end {
            return false;
        }
        let mut candidate = self.text.clone();
        candidate.drain(start..end);
        let candidateString = String::from_utf16_lossy(&candidate);
        if !(self.validator)(&candidateString) {
            return false;
        }
        self.text = candidate;
        self.cursorPosition = start;
        self.setSelectionPos(start, font);
        true
    }

    pub fn getNthWordFromCursor(&self, numWords: i32) -> usize {
        self.getNthWordFromPosWS(numWords, self.cursorPosition, true)
    }
    pub fn getNthWordFromPos(&self, n: i32, pos: usize) -> usize {
        self.getNthWordFromPosWS(n, pos, true)
    }
    pub fn getNthWordFromPosWS(&self, n: i32, pos: usize, skipWs: bool) -> usize {
        let mut index = pos.min(self.text.len());
        for _ in 0..n.unsigned_abs() {
            if n >= 0 {
                index = self.text[index..]
                    .iter()
                    .position(|unit| *unit == b' ' as u16)
                    .map(|value| index + value)
                    .unwrap_or(self.text.len());
                while skipWs && index < self.text.len() && self.text[index] == b' ' as u16 {
                    index += 1;
                }
            } else {
                while skipWs && index > 0 && self.text[index - 1] == b' ' as u16 {
                    index -= 1;
                }
                while index > 0 && self.text[index - 1] != b' ' as u16 {
                    index -= 1;
                }
            }
        }
        index
    }

    pub fn keyPressed(
        &mut self,
        key: GuiTextFieldKey,
        modifiers: GuiTextFieldModifiers,
        font: &FontRenderer,
    ) -> bool {
        if !self.isFocused {
            return false;
        }
        match key {
            GuiTextFieldKey::Backspace => {
                if modifiers.control {
                    self.deleteWords(-1, Some(font))
                } else {
                    self.deleteFromCursor(-1, Some(font))
                }
            }
            GuiTextFieldKey::Delete => {
                if modifiers.control {
                    self.deleteWords(1, Some(font))
                } else {
                    self.deleteFromCursor(1, Some(font))
                }
            }
            GuiTextFieldKey::Home => {
                if modifiers.shift {
                    self.setSelectionPos(0, Some(font));
                } else {
                    self.setCursorPosition(0, Some(font));
                }
                true
            }
            GuiTextFieldKey::End => {
                if modifiers.shift {
                    self.setSelectionPos(self.text.len(), Some(font));
                } else {
                    self.setCursorPositionEnd(Some(font));
                }
                true
            }
            GuiTextFieldKey::Left => {
                let target = if modifiers.control {
                    self.getNthWordFromCursor(-1)
                } else {
                    self.cursorPosition.saturating_sub(1)
                };
                if modifiers.shift {
                    self.setSelectionPos(target, Some(font));
                } else {
                    self.setCursorPosition(target, Some(font));
                }
                true
            }
            GuiTextFieldKey::Right => {
                let target = if modifiers.control {
                    self.getNthWordFromCursor(1)
                } else {
                    (self.cursorPosition + 1).min(self.text.len())
                };
                if modifiers.shift {
                    self.setSelectionPos(target, Some(font));
                } else {
                    self.setCursorPosition(target, Some(font));
                }
                true
            }
        }
    }

    pub fn selectAll(&mut self, font: &FontRenderer) {
        self.cursorPosition = self.text.len();
        self.setSelectionPos(0, Some(font));
    }
    pub fn moveCursorBy(&mut self, num: i32, font: Option<&FontRenderer>) {
        let target = if num < 0 {
            self.selectionEnd
                .saturating_sub(num.unsigned_abs() as usize)
        } else {
            (self.selectionEnd + num as usize).min(self.text.len())
        };
        self.setCursorPosition(target, font);
    }
    pub fn setCursorPosition(&mut self, pos: usize, font: Option<&FontRenderer>) {
        self.cursorPosition = pos.min(self.text.len());
        self.setSelectionPos(self.cursorPosition, font);
    }
    pub fn setCursorPositionZero(&mut self, font: Option<&FontRenderer>) {
        self.setCursorPosition(0, font);
    }
    pub fn setCursorPositionEnd(&mut self, font: Option<&FontRenderer>) {
        self.setCursorPosition(self.text.len(), font);
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
        font: &FontRenderer,
    ) -> bool {
        let inside = mouseX >= self.xPosition
            && mouseX < self.xPosition + self.width
            && mouseY >= self.yPosition
            && mouseY < self.yPosition + self.height;
        if self.canLoseFocus {
            self.setFocused(inside);
        }
        if !(self.isFocused && inside && mouseButton == 0) {
            return false;
        }
        let mut relative = mouseX - self.xPosition;
        if self.enableBackgroundDrawing {
            relative -= 4;
        }
        let displayText = self.displayTextUnits();
        let visible = substringUtf16(&displayText, self.lineScrollOffset, displayText.len());
        let trimmed = font.trim_string_to_width(&visible, self.getWidth(), false);
        let clicked = font
            .trim_string_to_width(&trimmed, relative.max(0), false)
            .encode_utf16()
            .count();
        self.setCursorPosition(self.lineScrollOffset + clicked, Some(font));
        true
    }

    pub fn buildRenderState(&self, font: &FontRenderer) -> GuiTextFieldRenderState {
        let color = if self.isEnabled {
            self.enabledColor
        } else {
            self.disabledColor
        };
        let cursor = self.cursorPosition.saturating_sub(self.lineScrollOffset);
        let selection = self.selectionEnd.saturating_sub(self.lineScrollOffset);
        let displayText = self.displayTextUnits();
        let visibleSource = substringUtf16(&displayText, self.lineScrollOffset, displayText.len());
        let visible = font.trim_string_to_width(&visibleSource, self.getWidth(), false);
        let visibleUnits: Vec<u16> = visible.encode_utf16().collect();
        let cursorVisibleInText = cursor <= visibleUnits.len();
        let blink = self.isFocused && self.cursorCounter / 6 % 2 == 0 && cursorVisibleInText;
        let textX = if self.enableBackgroundDrawing {
            self.xPosition + 4
        } else {
            self.xPosition
        };
        let textY = if self.enableBackgroundDrawing {
            self.yPosition + (self.height - 8) / 2
        } else {
            self.yPosition
        };
        let before = if cursorVisibleInText {
            String::from_utf16_lossy(&visibleUnits[..cursor])
        } else {
            visible.clone()
        };
        let beforeWidth = font.get_string_width(&before);
        let hasAfter =
            self.cursorPosition < self.text.len() || self.text.len() >= self.maxStringLength;
        let cursorX = if cursorVisibleInText {
            textX + beforeWidth - if hasAfter { 1 } else { 0 }
        } else if cursor > 0 {
            textX + self.width
        } else {
            textX
        };
        let selectionX = if selection != cursor && selection <= visibleUnits.len() {
            let selectionText = String::from_utf16_lossy(&visibleUnits[..selection]);
            Some(textX + font.get_string_width(&selectionText))
        } else {
            None
        };
        GuiTextFieldRenderState {
            text: visible,
            textX,
            textY,
            color,
            cursorX,
            cursorVisible: blink,
            cursorBlock: hasAfter,
            selectionX,
        }
    }

    pub fn drawTextBox(&self, drawList: &mut GuiDrawList, font: &mut FontRenderer) {
        if !self.visible {
            return;
        }
        if self.enableBackgroundDrawing {
            drawList.draw_rect(
                self.xPosition - 1,
                self.yPosition - 1,
                self.xPosition + self.width + 1,
                self.yPosition + self.height + 1,
                -6_250_336,
            );
            drawList.draw_rect(
                self.xPosition,
                self.yPosition,
                self.xPosition + self.width,
                self.yPosition + self.height,
                -16_777_216,
            );
        }
        let color = if self.isEnabled {
            self.enabledColor
        } else {
            self.disabledColor
        };
        let cursor = self.cursorPosition.saturating_sub(self.lineScrollOffset);
        let selection = self.selectionEnd.saturating_sub(self.lineScrollOffset);
        let displayText = self.displayTextUnits();
        let visibleSource = substringUtf16(&displayText, self.lineScrollOffset, displayText.len());
        let visible = font.trim_string_to_width(&visibleSource, self.getWidth(), false);
        let visibleUnits: Vec<u16> = visible.encode_utf16().collect();
        let cursorVisible = cursor <= visibleUnits.len();
        let blink = self.isFocused && self.cursorCounter / 6 % 2 == 0 && cursorVisible;
        let startX = if self.enableBackgroundDrawing {
            self.xPosition + 4
        } else {
            self.xPosition
        };
        let textY = if self.enableBackgroundDrawing {
            self.yPosition + (self.height - 8) / 2
        } else {
            self.yPosition
        };
        let before = if cursorVisible {
            String::from_utf16_lossy(&visibleUnits[..cursor])
        } else {
            visible.clone()
        };
        let mut currentX = startX;
        if !before.is_empty() {
            currentX =
                font.draw_string_with_shadow(drawList, &before, startX as f32, textY as f32, color);
        }
        let hasAfter =
            self.cursorPosition < self.text.len() || self.text.len() >= self.maxStringLength;
        let cursorX = if cursorVisible {
            if hasAfter {
                currentX - 1
            } else {
                currentX
            }
        } else if cursor > 0 {
            startX + self.width
        } else {
            startX
        };
        if cursorVisible && cursor < visibleUnits.len() {
            let after = String::from_utf16_lossy(&visibleUnits[cursor..]);
            font.draw_string_with_shadow(
                drawList,
                &after,
                (currentX - if hasAfter { 1 } else { 0 }) as f32,
                textY as f32,
                color,
            );
        }
        if blink {
            if hasAfter {
                drawList.draw_rect(
                    cursorX,
                    textY - 1,
                    cursorX + 1,
                    textY + 1 + font.font_height,
                    -3_092_272,
                );
            } else {
                font.draw_string_with_shadow(drawList, "_", cursorX as f32, textY as f32, color);
            }
        }
        if selection != cursor && selection <= visibleUnits.len() {
            let selectionText = String::from_utf16_lossy(&visibleUnits[..selection]);
            let selectionX = startX + font.get_string_width(&selectionText);
            drawList.draw_rect(
                cursorX,
                textY - 1,
                selectionX - 1,
                textY + 1 + font.font_height,
                0x8033_99FF_u32 as i32,
            );
        }
    }

    fn displayTextUnits(&self) -> Vec<u16> {
        match self.maskCharacter {
            Some(character) => {
                let mut encoded = [0u16; 2];
                let units = character.encode_utf16(&mut encoded);
                if units.len() == 1 {
                    vec![units[0]; self.text.len()]
                } else {
                    self.text.clone()
                }
            }
            None => self.text.clone(),
        }
    }

    pub fn setMaskCharacter(&mut self, character: Option<char>) {
        self.maskCharacter = character;
    }

    pub fn setMaxStringLength(&mut self, length: usize) {
        self.maxStringLength = length;
        self.text.truncate(length);
        self.cursorPosition = self.cursorPosition.min(length);
        self.selectionEnd = self.selectionEnd.min(length);
    }
    pub const fn getMaxStringLength(&self) -> usize {
        self.maxStringLength
    }
    pub const fn getCursorPosition(&self) -> usize {
        self.cursorPosition
    }
    pub const fn getEnableBackgroundDrawing(&self) -> bool {
        self.enableBackgroundDrawing
    }
    pub fn setEnableBackgroundDrawing(&mut self, value: bool) {
        self.enableBackgroundDrawing = value;
    }
    pub fn setTextColor(&mut self, color: i32) {
        self.enabledColor = color;
    }
    pub fn setDisabledTextColour(&mut self, color: i32) {
        self.disabledColor = color;
    }
    pub fn setFocused(&mut self, value: bool) {
        if value && !self.isFocused {
            self.cursorCounter = 0;
        }
        self.isFocused = value;
    }
    pub const fn isFocused(&self) -> bool {
        self.isFocused
    }
    pub fn setEnabled(&mut self, value: bool) {
        self.isEnabled = value;
    }
    pub const fn getSelectionEnd(&self) -> usize {
        self.selectionEnd
    }
    pub const fn getWidth(&self) -> i32 {
        if self.enableBackgroundDrawing {
            self.width - 8
        } else {
            self.width
        }
    }
    pub fn setSelectionPos(&mut self, position: usize, font: Option<&FontRenderer>) {
        self.selectionEnd = position.min(self.text.len());
        let Some(font) = font else {
            return;
        };
        self.lineScrollOffset = self.lineScrollOffset.min(self.text.len());
        let displayText = self.displayTextUnits();
        let visible = substringUtf16(&displayText, self.lineScrollOffset, displayText.len());
        let visibleLength = font
            .trim_string_to_width(&visible, self.getWidth(), false)
            .encode_utf16()
            .count();
        let visibleEnd = self.lineScrollOffset + visibleLength;
        if self.selectionEnd == self.lineScrollOffset {
            let reverse = font
                .trim_string_to_width(
                    &String::from_utf16_lossy(&displayText),
                    self.getWidth(),
                    true,
                )
                .encode_utf16()
                .count();
            self.lineScrollOffset = self.lineScrollOffset.saturating_sub(reverse);
        }
        if self.selectionEnd > visibleEnd {
            self.lineScrollOffset += self.selectionEnd - visibleEnd;
        } else if self.selectionEnd <= self.lineScrollOffset {
            self.lineScrollOffset = self.selectionEnd;
        }
    }
    pub fn setCanLoseFocus(&mut self, value: bool) {
        self.canLoseFocus = value;
    }
    pub const fn getVisible(&self) -> bool {
        self.visible
    }
    pub fn setVisible(&mut self, value: bool) {
        self.visible = value;
    }
}

fn substringUtf16(units: &[u16], start: usize, end: usize) -> String {
    String::from_utf16_lossy(&units[start.min(units.len())..end.min(units.len())])
}
fn isAllowedCharacter(character: char) -> bool {
    character != '§' && character >= ' ' && character != '\u{7f}'
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn max_length_uses_java_utf16_units() {
        let mut field = GuiTextField::new(0, 0, 0, 100, 20);
        field.setMaxStringLength(3);
        field.setText("A😀B");
        assert_eq!(field.getText(), "A😀");
    }
}
