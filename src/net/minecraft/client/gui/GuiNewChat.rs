use crate::net::minecraft::client::gui::ChatLine::ChatLine;
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiIngame::{HudSolidRect, HudText};
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChatFrame {
    pub rectangles: Vec<HudSolidRect>,
    pub texts: Vec<HudText>,
    pub textScale: f32,
}

/// Backend-neutral state and layout port of MCP 1.12.2 `GuiNewChat`.
///
/// The Vulkan backend consumes the resulting rectangles and formatted strings;
/// message lifetime, wrapping, history, scrolling and opacity remain owned by
/// this MCP-named class.
#[derive(Debug, Clone, Default)]
pub struct GuiNewChat {
    sentMessages: Vec<String>,
    chatLines: Vec<ChatLine>,
    drawnChatLines: Vec<ChatLine>,
    scrollPos: i32,
    isScrolled: bool,
    lastMessageSerial: u64,
}

impl GuiNewChat {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clearChatMessages(&mut self, clearSent: bool) {
        self.drawnChatLines.clear();
        self.chatLines.clear();
        self.scrollPos = 0;
        self.isScrolled = false;
        if clearSent {
            self.sentMessages.clear();
        }
    }

    pub fn acceptMessage(
        &mut self,
        serial: u64,
        chatComponent: ITextComponent,
        updateCounter: i32,
        wrapWidth: i32,
    ) {
        if serial <= self.lastMessageSerial {
            return;
        }
        self.lastMessageSerial = serial;
        self.printChatMessage(chatComponent, updateCounter, wrapWidth);
    }

    pub fn acceptMessageWithFont(
        &mut self,
        serial: u64,
        chatComponent: ITextComponent,
        updateCounter: i32,
        wrapWidth: i32,
        fontRenderer: &FontRenderer,
    ) {
        if serial <= self.lastMessageSerial {
            return;
        }
        self.lastMessageSerial = serial;
        self.printChatMessageWithFont(chatComponent, updateCounter, wrapWidth, fontRenderer);
    }

    pub fn printChatMessageWithFont(
        &mut self,
        chatComponent: ITextComponent,
        updateCounter: i32,
        wrapWidth: i32,
        fontRenderer: &FontRenderer,
    ) {
        self.setChatLineWithFont(
            chatComponent,
            0,
            updateCounter,
            false,
            wrapWidth.max(1),
            fontRenderer,
        );
    }

    pub fn printChatMessageWithOptionalDeletionWithFont(
        &mut self,
        chatComponent: ITextComponent,
        chatLineId: i32,
        updateCounter: i32,
        wrapWidth: i32,
        fontRenderer: &FontRenderer,
    ) {
        self.setChatLineWithFont(
            chatComponent,
            chatLineId,
            updateCounter,
            false,
            wrapWidth.max(1),
            fontRenderer,
        );
    }

    pub fn printChatMessage(
        &mut self,
        chatComponent: ITextComponent,
        updateCounter: i32,
        wrapWidth: i32,
    ) {
        self.printChatMessageWithOptionalDeletion(chatComponent, 0, updateCounter, wrapWidth);
    }

    pub fn printChatMessageWithOptionalDeletion(
        &mut self,
        chatComponent: ITextComponent,
        chatLineId: i32,
        updateCounter: i32,
        wrapWidth: i32,
    ) {
        self.setChatLine(
            chatComponent,
            chatLineId,
            updateCounter,
            false,
            wrapWidth.max(1),
        );
    }

    fn setChatLine(
        &mut self,
        chatComponent: ITextComponent,
        chatLineId: i32,
        updateCounter: i32,
        displayOnly: bool,
        wrapWidth: i32,
    ) {
        if chatLineId != 0 {
            self.deleteChatLine(chatLineId);
        }
        let lines = split_formatted_text(chatComponent.getFormattedText(), wrapWidth);
        for line in lines {
            if self.isScrolled && self.scrollPos > 0 {
                self.scroll(1, 20);
            }
            self.drawnChatLines.insert(
                0,
                ChatLine::new(
                    updateCounter,
                    ITextComponent::fromPlainText(line),
                    chatLineId,
                ),
            );
        }
        self.drawnChatLines.truncate(100);
        if !displayOnly {
            self.chatLines
                .insert(0, ChatLine::new(updateCounter, chatComponent, chatLineId));
            self.chatLines.truncate(100);
        }
    }

    fn setChatLineWithFont(
        &mut self,
        chatComponent: ITextComponent,
        chatLineId: i32,
        updateCounter: i32,
        displayOnly: bool,
        wrapWidth: i32,
        fontRenderer: &FontRenderer,
    ) {
        if chatLineId != 0 {
            self.deleteChatLine(chatLineId);
        }
        let lines = fontRenderer
            .list_formatted_string_to_width(chatComponent.getFormattedText(), wrapWidth.max(1));
        for line in lines {
            if self.isScrolled && self.scrollPos > 0 {
                self.scroll(1, 20);
            }
            self.drawnChatLines.insert(
                0,
                ChatLine::new(
                    updateCounter,
                    ITextComponent::fromPlainText(line),
                    chatLineId,
                ),
            );
        }
        self.drawnChatLines.truncate(100);
        if !displayOnly {
            self.chatLines
                .insert(0, ChatLine::new(updateCounter, chatComponent, chatLineId));
            self.chatLines.truncate(100);
        }
    }

    /// Finds and deletes a chat line by ID, matching MCP 1.12.2
    /// `GuiNewChat#deleteChatLine`. All wrapped display lines with the ID
    /// are removed, while only the first stored source line is removed.
    pub fn deleteChatLine(&mut self, id: i32) {
        self.drawnChatLines
            .retain(|line| line.getChatLineID() != id);
        if let Some(index) = self
            .chatLines
            .iter()
            .position(|line| line.getChatLineID() == id)
        {
            self.chatLines.remove(index);
        }
    }

    pub fn refreshChat(&mut self, wrapWidth: i32) {
        let lines = self.chatLines.clone();
        self.drawnChatLines.clear();
        self.resetScroll();
        for line in lines.into_iter().rev() {
            self.setChatLine(
                line.getChatComponent().clone(),
                line.getChatLineID(),
                line.getUpdatedCounter(),
                true,
                wrapWidth,
            );
        }
    }

    pub fn getSentMessages(&self) -> &[String] {
        &self.sentMessages
    }

    pub fn addToSentMessages(&mut self, message: impl Into<String>) {
        let message = message.into();
        if self.sentMessages.last() != Some(&message) {
            self.sentMessages.push(message);
        }
    }

    pub fn resetScroll(&mut self) {
        self.scrollPos = 0;
        self.isScrolled = false;
    }

    pub fn scroll(&mut self, amount: i32, lineCount: i32) {
        self.scrollPos += amount;
        let maximum = (self.drawnChatLines.len() as i32 - lineCount).max(0);
        self.scrollPos = self.scrollPos.clamp(0, maximum);
        if self.scrollPos == 0 {
            self.isScrolled = false;
        } else {
            self.isScrolled = true;
        }
    }

    pub fn calculateChatboxWidth(scale: f32) -> i32 {
        (scale * 280.0 + 40.0) as i32
    }

    pub fn calculateChatboxHeight(scale: f32) -> i32 {
        (scale * 160.0 + 20.0) as i32
    }

    #[allow(clippy::too_many_arguments)]
    pub fn buildFrame(
        &self,
        guiHeight: i32,
        updateCounter: i32,
        chatOpen: bool,
        chatOpacity: f32,
        chatScale: f32,
        chatWidthSetting: f32,
        chatHeightFocused: f32,
        chatHeightUnfocused: f32,
    ) -> ChatFrame {
        let chatScale = chatScale.clamp(0.0, 1.0).max(0.01);
        let chatWidth = Self::calculateChatboxWidth(chatWidthSetting.clamp(0.0, 1.0));
        let chatHeightSetting = if chatOpen {
            chatHeightFocused
        } else {
            chatHeightUnfocused
        };
        let lineCount =
            (Self::calculateChatboxHeight(chatHeightSetting.clamp(0.0, 1.0)) / 9).max(1);
        let opacity = chatOpacity.clamp(0.0, 1.0) * 0.9 + 0.1;
        let scaledWidth = ((chatWidth as f32) / chatScale).ceil() as i32;
        let baseX = 2.0_f32;
        let baseY = (guiHeight - 40) as f32;
        let mut visibleLines = 0_i32;
        let mut frame = ChatFrame {
            textScale: chatScale,
            ..ChatFrame::default()
        };

        for lineIndex in 0..lineCount {
            let sourceIndex = lineIndex + self.scrollPos;
            let Some(line) = self.drawnChatLines.get(sourceIndex as usize) else {
                break;
            };
            let age = updateCounter - line.getUpdatedCounter();
            if age >= 200 && !chatOpen {
                continue;
            }
            let alpha = if chatOpen {
                255
            } else {
                let mut fade = 1.0 - age.max(0) as f64 / 200.0;
                fade = (fade * 10.0).clamp(0.0, 1.0);
                (255.0 * fade * fade) as i32
            };
            let alpha = ((alpha as f32) * opacity) as i32;
            if alpha <= 3 {
                continue;
            }
            visibleLines += 1;
            let rowY = -lineIndex * 9;
            let rectX = (baseX - 2.0 * chatScale).floor() as i32;
            let rectY = (baseY + (rowY - 9) as f32 * chatScale).floor() as i32;
            frame.rectangles.push(HudSolidRect::new(
                rectX,
                rectY,
                ((scaledWidth + 4) as f32 * chatScale).ceil() as i32,
                (9.0 * chatScale).ceil() as i32,
                ((alpha / 2) as u32) << 24,
            ));
            frame.texts.push(HudText {
                text: line.getChatComponent().getFormattedText().to_owned(),
                x: baseX.floor() as i32,
                y: (baseY + (rowY - 8) as f32 * chatScale).floor() as i32,
                color: ((alpha as u32) << 24) | 0x00FF_FFFF,
                outline: true,
            });
        }

        if chatOpen {
            let total = self.drawnChatLines.len() as i32;
            let fontHeight = 9;
            let totalHeight = total * fontHeight + total;
            let visibleHeight = visibleLines * fontHeight + visibleLines;
            if totalHeight != visibleHeight && total > 0 {
                let scrollOffset = self.scrollPos * visibleHeight / total;
                let thumbHeight = visibleHeight * visibleHeight / totalHeight.max(1);
                let alpha = if scrollOffset > 0 { 170 } else { 96 };
                let color = if self.isScrolled {
                    0x00CC_3333
                } else {
                    0x0033_3333
                };
                let y = guiHeight - 40 - scrollOffset;
                frame.rectangles.push(HudSolidRect::new(
                    0,
                    y - thumbHeight,
                    2,
                    thumbHeight,
                    ((alpha as u32) << 24) | color,
                ));
                frame.rectangles.push(HudSolidRect::new(
                    2,
                    y - thumbHeight,
                    1,
                    thumbHeight,
                    ((alpha as u32) << 24) | 0x00CC_CCCC,
                ));
            }
        }
        frame
    }
}

pub(crate) fn split_formatted_text(text: &str, maximumWidth: i32) -> Vec<String> {
    let maximumChars = (maximumWidth.max(6) / 6).max(1) as usize;
    let mut output = Vec::new();
    for logicalLine in text.split('\n') {
        let mut current = String::new();
        let mut visible = 0_usize;
        let mut formatting = false;
        let mut activeCodes = String::new();
        for character in logicalLine.chars() {
            if formatting {
                current.push(character);
                if character.eq_ignore_ascii_case(&'r') {
                    activeCodes.clear();
                } else {
                    activeCodes.push('§');
                    activeCodes.push(character);
                }
                formatting = false;
                continue;
            }
            if character == '§' {
                current.push(character);
                formatting = true;
                continue;
            }
            if visible >= maximumChars {
                output.push(current);
                current = activeCodes.clone();
                visible = 0;
            }
            current.push(character);
            visible += 1;
        }
        output.push(current);
    }
    if output.is_empty() {
        output.push(String::new());
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_one_hundred_drawn_lines() {
        let mut chat = GuiNewChat::new();
        for index in 0..120 {
            chat.printChatMessage(ITextComponent::fromPlainText(index.to_string()), index, 320);
        }
        assert_eq!(chat.drawnChatLines.len(), 100);
        assert_eq!(chat.chatLines.len(), 100);
    }

    #[test]
    fn delete_chat_line_removes_all_wrapped_rows_and_one_source_line() {
        let mut chat = GuiNewChat::new();
        chat.printChatMessageWithOptionalDeletion(
            ITextComponent::fromPlainText("abcdefghij"),
            7,
            1,
            12,
        );
        assert!(chat.drawnChatLines.len() > 1);
        assert_eq!(chat.chatLines.len(), 1);
        chat.deleteChatLine(7);
        assert!(chat
            .drawnChatLines
            .iter()
            .all(|line| line.getChatLineID() != 7));
        assert!(chat.chatLines.iter().all(|line| line.getChatLineID() != 7));
    }

    #[test]
    fn sent_history_avoids_adjacent_duplicates() {
        let mut chat = GuiNewChat::new();
        chat.addToSentMessages("hello");
        chat.addToSentMessages("hello");
        chat.addToSentMessages("world");
        assert_eq!(chat.getSentMessages(), ["hello", "world"]);
    }
}
