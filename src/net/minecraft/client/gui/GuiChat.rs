use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiTextField::{
    GuiTextField, GuiTextFieldKey, GuiTextFieldModifiers, GuiTextFieldRenderState,
};
use crate::net::minecraft::network::play::client::CPacketTabComplete::CPacketTabComplete;

#[derive(Debug, Clone)]
pub struct GuiChat {
    historyBuffer: String,
    sentHistoryCursor: usize,
    inputField: GuiTextField,
    defaultInputFieldText: String,
    // MCP `TabCompleter` state belongs to the current GuiChat instance.
    didComplete: bool,
    requestedCompletions: bool,
    completionIdx: usize,
    completions: Vec<String>,
    completionDisplayLine: Option<String>,
}

impl GuiChat {
    pub fn new(
        defaultText: impl Into<String>,
        width: i32,
        height: i32,
        sentMessageCount: usize,
    ) -> Self {
        let defaultInputFieldText = defaultText.into();
        let mut inputField = GuiTextField::new(0, 4, height - 12, width - 4, 12);
        inputField.setMaxStringLength(256);
        inputField.setEnableBackgroundDrawing(false);
        inputField.setFocused(true);
        inputField.setText(&defaultInputFieldText);
        inputField.setCanLoseFocus(false);
        Self {
            historyBuffer: String::new(),
            sentHistoryCursor: sentMessageCount,
            inputField,
            defaultInputFieldText,
            didComplete: false,
            requestedCompletions: false,
            completionIdx: 0,
            completions: Vec::new(),
            completionDisplayLine: None,
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.inputField.xPosition = 4;
        self.inputField.yPosition = height - 12;
        self.inputField.width = width - 4;
        self.inputField.height = 12;
    }

    pub fn updateScreen(&mut self) {
        self.inputField.updateCursorCounter();
    }

    pub fn typedText(&mut self, text: &str, font: &FontRenderer) -> bool {
        self.resetRequested();
        self.resetDidComplete();
        self.inputField.writeText(text, Some(font))
    }

    pub fn keyPressed(
        &mut self,
        key: GuiTextFieldKey,
        modifiers: GuiTextFieldModifiers,
        font: &FontRenderer,
    ) -> bool {
        self.resetRequested();
        self.resetDidComplete();
        self.inputField.keyPressed(key, modifiers, font)
    }

    pub fn selectAll(&mut self, font: &FontRenderer) {
        self.inputField.selectAll(font);
    }
    pub fn getText(&self) -> String {
        self.inputField.getText()
    }
    pub fn getTrimmedText(&self) -> String {
        self.inputField.getText().trim().to_owned()
    }
    pub fn renderState(&self, font: &FontRenderer) -> GuiTextFieldRenderState {
        self.inputField.buildRenderState(font)
    }

    /// MCP `TabCompleter#complete`. On the first press this returns the exact
    /// serverbound request. Once the server response is installed by
    /// `setCompletions`, subsequent presses cycle the retained candidates.
    pub fn complete(&mut self, font: &FontRenderer) -> Option<CPacketTabComplete> {
        self.resetRequested();
        self.completionDisplayLine = None;
        if self.didComplete {
            self.inputField.deleteFromCursor(0, Some(font));
            let cursor = self.inputField.getCursorPosition();
            let wordStart = self.inputField.getNthWordFromPosWS(-1, cursor, false);
            self.inputField
                .deleteFromCursor(wordStart as i32 - cursor as i32, Some(font));
            if self.completionIdx >= self.completions.len() {
                self.completionIdx = 0;
            }
            if let Some(completion) = self.completions.get(self.completionIdx).cloned() {
                self.completionIdx += 1;
                self.inputField.writeText(&completion, Some(font));
            }
            if self.completions.len() > 1 {
                self.completionDisplayLine = Some(self.completions.join(", "));
            }
            None
        } else {
            let cursor = self.inputField.getCursorPosition();
            let prefix = utf16_prefix(&self.inputField.getText(), cursor);
            self.completions.clear();
            self.completionIdx = 0;
            if prefix.is_empty() {
                return None;
            }
            self.requestedCompletions = true;
            Some(CPacketTabComplete::new(prefix, None, false))
        }
    }

    /// MCP `TabCompleter#setCompletions`. Returns the comma-separated line
    /// which GuiChat displays through `GuiNewChat` with deletion id 1.
    pub fn setCompletions(
        &mut self,
        newCompletions: &[String],
        font: &FontRenderer,
    ) -> Option<String> {
        if !self.requestedCompletions {
            return None;
        }
        self.requestedCompletions = false;
        self.didComplete = false;
        self.completions.clear();
        self.completions.extend(
            newCompletions
                .iter()
                .filter(|value| !value.is_empty())
                .cloned(),
        );

        let cursor = self.inputField.getCursorPosition();
        let text = self.inputField.getText();
        let wordStart = self.inputField.getNthWordFromPosWS(-1, cursor, false);
        let currentWord = utf16_slice(&text, wordStart, cursor);
        let common = common_prefix(newCompletions);
        if !common.is_empty() && !currentWord.eq_ignore_ascii_case(&common) {
            self.inputField.deleteFromCursor(0, Some(font));
            let cursor = self.inputField.getCursorPosition();
            let start = self.inputField.getNthWordFromPosWS(-1, cursor, false);
            self.inputField
                .deleteFromCursor(start as i32 - cursor as i32, Some(font));
            self.inputField.writeText(&common, Some(font));
            self.completionDisplayLine = None;
        } else if !self.completions.is_empty() {
            self.didComplete = true;
            let _ = self.complete(font);
        }

        self.takeCompletionDisplayLine()
    }

    pub fn takeCompletionDisplayLine(&mut self) -> Option<String> {
        self.completionDisplayLine.take()
    }

    pub fn resetDidComplete(&mut self) {
        self.didComplete = false;
    }
    pub fn resetRequested(&mut self) {
        self.requestedCompletions = false;
    }

    pub fn getSentHistory(&mut self, msgPos: i32, sentMessages: &[String]) {
        self.resetRequested();
        self.resetDidComplete();
        let count = sentMessages.len();
        let target = (self.sentHistoryCursor as i32 + msgPos).clamp(0, count as i32) as usize;
        if target == self.sentHistoryCursor {
            return;
        }
        if target == count {
            self.sentHistoryCursor = count;
            self.inputField.setText(&self.historyBuffer);
        } else {
            if self.sentHistoryCursor == count {
                self.historyBuffer = self.inputField.getText();
            }
            self.inputField.setText(&sentMessages[target]);
            self.sentHistoryCursor = target;
        }
    }

    pub fn defaultText(&self) -> &str {
        &self.defaultInputFieldText
    }
}

fn utf16_prefix(value: &str, units: usize) -> String {
    String::from_utf16_lossy(&value.encode_utf16().take(units).collect::<Vec<_>>())
}

fn utf16_slice(value: &str, start: usize, end: usize) -> String {
    let units = value.encode_utf16().collect::<Vec<_>>();
    String::from_utf16_lossy(&units[start.min(units.len())..end.min(units.len())])
}

fn common_prefix(values: &[String]) -> String {
    let Some(first) = values.first() else {
        return String::new();
    };
    let mut prefix = first.chars().collect::<Vec<_>>();
    for value in &values[1..] {
        let chars = value.chars().collect::<Vec<_>>();
        let matching = prefix
            .iter()
            .zip(chars.iter())
            .take_while(|(left, right)| left == right)
            .count();
        prefix.truncate(matching);
        if prefix.is_empty() {
            break;
        }
    }
    prefix.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_restores_unsent_buffer_at_end() {
        let mut chat = GuiChat::new("draft", 320, 240, 2);
        chat.historyBuffer = "draft".to_owned();
        chat.getSentHistory(-1, &["one".to_owned(), "two".to_owned()]);
        assert_eq!(chat.getText(), "two");
        chat.getSentHistory(1, &["one".to_owned(), "two".to_owned()]);
        assert_eq!(chat.getText(), "draft");
    }

    #[test]
    fn completion_requests_prefix_then_cycles_server_results() {
        let font = FontRenderer::test_metric_renderer();
        let mut chat = GuiChat::new("/give Pla", 320, 240, 0);
        let request = chat
            .complete(&font)
            .expect("first tab requests server completions");
        assert_eq!(request.getMessage(), "/give Pla");
        let display = chat.setCompletions(&["Player905".to_owned(), "Player906".to_owned()], &font);
        assert_eq!(display, None);
        assert_eq!(chat.getText(), "/give Player90");
        // A second request is required after the common prefix was inserted,
        // matching TabCompleter#setCompletions.
        let request = chat
            .complete(&font)
            .expect("common prefix requests the narrowed set");
        assert_eq!(request.getMessage(), "/give Player90");
        let display = chat.setCompletions(&["Player905".to_owned(), "Player906".to_owned()], &font);
        assert_eq!(display.as_deref(), Some("Player905, Player906"));
        assert_eq!(chat.getText(), "/give Player905");
        assert!(chat.complete(&font).is_none());
        assert_eq!(
            chat.takeCompletionDisplayLine().as_deref(),
            Some("Player905, Player906")
        );
        assert_eq!(chat.getText(), "/give Player906");
    }
}
