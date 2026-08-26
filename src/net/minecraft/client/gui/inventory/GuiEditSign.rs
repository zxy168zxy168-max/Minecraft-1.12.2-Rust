use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::tileentity::TileEntitySign::TileEntitySign;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::vulkan::GuiDrawList::GuiDrawList;

/// MCP 1.12.2 `GuiEditSign` state and input semantics.
///
/// The original screen renders `TileEntitySignRenderer` under a fixed model-view
/// transform. The Vulkan GUI path uses the exact front-face regions of
/// `textures/entity/sign.png` at the same 93.75 GUI scale, while retaining the
/// original line positioning, edit markers and button geometry.
#[derive(Debug, Clone)]
pub struct GuiEditSign {
    pub GuiScreen: GuiScreen,
    position: BlockPos,
    blockId: i32,
    metadata: i32,
    lines: [String; 4],
    updateCounter: i32,
    editLine: usize,
    doneRequested: bool,
}

impl GuiEditSign {
    pub fn new(sign: &TileEntitySign, blockIdIn: i32, metadataIn: i32) -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            position: sign.pos,
            blockId: blockIdIn,
            metadata: metadataIn,
            lines: std::array::from_fn(|index| {
                sign.signText[index].getUnformattedText().to_owned()
            }),
            updateCounter: 0,
            editLine: 0,
            doneRequested: false,
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32, locale: &Locale) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.GuiScreen.addButton(GuiButton::new(
            0,
            width / 2 - 100,
            height / 4 + 120,
            locale.translate_key("gui.done"),
        ));
    }

    pub fn updateScreen(&mut self) {
        self.updateCounter = self.updateCounter.wrapping_add(1);
    }

    pub fn drawScreen(
        &mut self,
        drawList: &mut GuiDrawList,
        fontRendererObj: &mut FontRenderer,
        locale: &Locale,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.GuiScreen.drawDefaultBackgroundInWorld(drawList);
        fontRendererObj.draw_centered_string_with_shadow(
            drawList,
            locale.translate_key("sign.edit"),
            self.GuiScreen.width / 2,
            40,
            0xFF_FFFF,
        );

        self.drawSignPreview(drawList, fontRendererObj);
        self.GuiScreen
            .drawScreen(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
    }

    fn drawSignPreview(&self, drawList: &mut GuiDrawList, fontRendererObj: &mut FontRenderer) {
        let texture = ResourceLocation::parse("textures/entity/sign.png");
        let centerX = self.GuiScreen.width as f32 * 0.5;
        // ModelSign board: 24x12 model pixels, rendered at 0.0625 and then
        // GuiEditSign's 93.75 scale => 140.625 x 70.3125 screen pixels.
        let boardWidth = 140.625_f32;
        let boardHeight = 70.3125_f32;
        let boardLeft = centerX - boardWidth * 0.5;
        let boardTop = 58.0_f32;
        drawList.push_textured_quad(
            texture.clone(),
            [
                (
                    boardLeft,
                    boardTop + boardHeight,
                    2.0 / 64.0,
                    14.0 / 32.0,
                    0xFFFF_FFFF,
                ),
                (
                    boardLeft + boardWidth,
                    boardTop + boardHeight,
                    26.0 / 64.0,
                    14.0 / 32.0,
                    0xFFFF_FFFF,
                ),
                (
                    boardLeft + boardWidth,
                    boardTop,
                    26.0 / 64.0,
                    2.0 / 32.0,
                    0xFFFF_FFFF,
                ),
                (boardLeft, boardTop, 2.0 / 64.0, 2.0 / 32.0, 0xFFFF_FFFF),
            ],
        );

        if self.blockId == 63 {
            // ModelSign stick front: texture offset (0,14), 2x14x2 box.
            let stickWidth = 11.71875_f32;
            let stickHeight = 82.03125_f32;
            let stickLeft = centerX - stickWidth * 0.5;
            drawList.push_textured_quad(
                texture,
                [
                    (
                        stickLeft,
                        boardTop + boardHeight + stickHeight,
                        2.0 / 64.0,
                        30.0 / 32.0,
                        0xFFFF_FFFF,
                    ),
                    (
                        stickLeft + stickWidth,
                        boardTop + boardHeight + stickHeight,
                        4.0 / 64.0,
                        30.0 / 32.0,
                        0xFFFF_FFFF,
                    ),
                    (
                        stickLeft + stickWidth,
                        boardTop + boardHeight,
                        4.0 / 64.0,
                        16.0 / 32.0,
                        0xFFFF_FFFF,
                    ),
                    (
                        stickLeft,
                        boardTop + boardHeight,
                        2.0 / 64.0,
                        16.0 / 32.0,
                        0xFFFF_FFFF,
                    ),
                ],
            );
        }

        // TileEntitySignRenderer draws the four baselines at -20,-10,0,10
        // after the GuiEditSign transform. At the GUI scale the font remains
        // effectively one screen pixel per font pixel.
        let firstBaseline = boardTop as i32 + 16;
        for (index, line) in self.lines.iter().enumerate() {
            let rendered = if index == self.editLine && self.updateCounter / 6 % 2 == 0 {
                format!("> {line} <")
            } else {
                line.clone()
            };
            let x = self.GuiScreen.width / 2 - fontRendererObj.get_string_width(&rendered) / 2;
            fontRendererObj.draw_string(
                drawList,
                &rendered,
                x as f32,
                (firstBaseline + index as i32 * 10) as f32,
                0,
                false,
            );
        }
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
    ) -> Option<GuiSoundCommand> {
        if mouseButton != 0 {
            return None;
        }
        let button = self
            .GuiScreen
            .buttonList
            .iter()
            .find(|button| button.mousePressed(mouseX, mouseY))?;
        if button.id == 0 {
            self.doneRequested = true;
            return Some(button.playPressSound());
        }
        None
    }

    pub fn keyPressed(&mut self, key: SignEditKey) -> bool {
        match key {
            SignEditKey::Up => self.editLine = self.editLine.wrapping_sub(1) & 3,
            SignEditKey::Down | SignEditKey::Enter => self.editLine = (self.editLine + 1) & 3,
            SignEditKey::Backspace => {
                self.lines[self.editLine].pop();
            }
            SignEditKey::Escape => self.doneRequested = true,
        }
        true
    }

    pub fn typedText(&mut self, text: &str, fontRendererObj: &FontRenderer) -> bool {
        let mut changed = false;
        for character in text.chars() {
            if !isAllowedCharacter(character) {
                continue;
            }
            let mut candidate = self.lines[self.editLine].clone();
            candidate.push(character);
            if fontRendererObj.get_string_width(&candidate) <= 90 {
                self.lines[self.editLine] = candidate;
                changed = true;
            }
        }
        changed
    }

    pub fn applyToTileEntity(&self, sign: &mut TileEntitySign) {
        for index in 0..4 {
            sign.signText[index] =
                crate::net::minecraft::util::text::ITextComponent::ITextComponent::fromPlainText(
                    self.lines[index].clone(),
                );
        }
        sign.lineBeingEdited = if self.updateCounter / 6 % 2 == 0 {
            self.editLine as i32
        } else {
            -1
        };
        sign.setEditable(false);
    }

    pub fn finishTileEntity(&self, sign: &mut TileEntitySign) {
        self.applyToTileEntity(sign);
        sign.lineBeingEdited = -1;
        sign.setEditable(true);
    }

    pub const fn getPosition(&self) -> BlockPos {
        self.position
    }
    pub const fn getBlockId(&self) -> i32 {
        self.blockId
    }
    pub const fn getMetadata(&self) -> i32 {
        self.metadata
    }
    pub const fn getLines(&self) -> &[String; 4] {
        &self.lines
    }
    pub const fn isDoneRequested(&self) -> bool {
        self.doneRequested
    }
    pub fn clearDoneRequest(&mut self) {
        self.doneRequested = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignEditKey {
    Up,
    Down,
    Enter,
    Backspace,
    Escape,
}

fn isAllowedCharacter(character: char) -> bool {
    character != '§' && character >= ' ' && character != '\u{7f}'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_edit_line_like_mcp_bitmask() {
        let sign = TileEntitySign::new(BlockPos::ORIGIN);
        let mut gui = GuiEditSign::new(&sign, 63, 0);
        gui.keyPressed(SignEditKey::Up);
        assert_eq!(gui.editLine, 3);
        gui.keyPressed(SignEditKey::Down);
        assert_eq!(gui.editLine, 0);
        gui.keyPressed(SignEditKey::Enter);
        assert_eq!(gui.editLine, 1);
    }

    #[test]
    fn rejects_section_sign_and_control_characters() {
        assert!(!isAllowedCharacter('§'));
        assert!(!isAllowedCharacter('\u{7f}'));
        assert!(isAllowedCharacter('中'));
    }
}
