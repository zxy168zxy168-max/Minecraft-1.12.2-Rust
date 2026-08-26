use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatLine {
    updatedCounter: i32,
    lineString: ITextComponent,
    chatLineID: i32,
}

impl ChatLine {
    pub fn new(updatedCounter: i32, lineString: ITextComponent, chatLineID: i32) -> Self {
        Self {
            updatedCounter,
            lineString,
            chatLineID,
        }
    }

    pub const fn getUpdatedCounter(&self) -> i32 {
        self.updatedCounter
    }
    pub fn getChatComponent(&self) -> &ITextComponent {
        &self.lineString
    }
    pub const fn getChatLineID(&self) -> i32 {
        self.chatLineID
    }
}
