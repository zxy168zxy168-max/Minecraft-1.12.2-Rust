use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::Gui::Gui;
use crate::net::minecraft::client::gui::GuiButton::GuiButton;
use crate::vulkan::GuiDrawList::GuiDrawList;

/// Initial MCP-compatible base for `GuiScreen`. Item rendering, clipboard,
/// chat-component events, and tooltip/NBT handling remain unmigrated and are
/// explicitly tracked as later methods of this same class rather than placed
/// in a replacement UI framework.
#[derive(Debug, Clone)]
pub struct GuiScreen {
    pub Gui: Gui,
    pub width: i32,
    pub height: i32,
    pub buttonList: Vec<GuiButton>,
    pub allowUserInput: bool,
    selectedButton: Option<usize>,
    eventButton: i32,
    lastMouseEvent: i64,
    touchValue: i32,
    focused: bool,
}

impl Default for GuiScreen {
    fn default() -> Self {
        Self {
            Gui: Gui::default(),
            width: 0,
            height: 0,
            buttonList: Vec::new(),
            allowUserInput: false,
            selectedButton: None,
            eventButton: -1,
            lastMouseEvent: 0,
            touchValue: 0,
            focused: false,
        }
    }
}

impl GuiScreen {
    pub fn drawScreen(
        &mut self,
        drawList: &mut GuiDrawList,
        fontRendererObj: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        for button in &mut self.buttonList {
            button.drawButton(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
        }
    }

    /// MCP `GuiScreen.drawDefaultBackground` for a client with no loaded world.
    /// The dirt texture and 32-pixel UV scale match `drawWorldBackground(0)`.
    pub fn drawDefaultBackground(&self, drawList: &mut GuiDrawList) {
        self.drawWorldBackground(drawList, 0);
    }

    /// MCP `GuiScreen.drawDefaultBackground` when a world is already loaded.
    /// Vanilla keeps the world visible and overlays the same vertical black
    /// gradient used by pause/options screens instead of drawing dirt.
    pub fn drawDefaultBackgroundInWorld(&self, drawList: &mut GuiDrawList) {
        drawList.draw_gradient_rect(
            0,
            0,
            self.width.max(1),
            self.height.max(1),
            0xC010_1010_u32 as i32,
            0xD010_1010_u32 as i32,
        );
    }

    pub fn drawWorldBackground(&self, drawList: &mut GuiDrawList, tint: i32) {
        let shade = (64 + tint).clamp(0, 255) as u32;
        let color = 0xFF00_0000 | (shade << 16) | (shade << 8) | shade;
        let width = self.width.max(1) as f32;
        let height = self.height.max(1) as f32;
        let texture = crate::net::minecraft::client::gui::Gui::OPTIONS_BACKGROUND.clone();
        drawList.push_textured_quad(
            texture,
            [
                (0.0, height, 0.0, height / 32.0, color),
                (width, height, width / 32.0, height / 32.0, color),
                (width, 0.0, width / 32.0, 0.0, color),
                (0.0, 0.0, 0.0, 0.0, color),
            ],
        );
    }

    pub fn addButton(&mut self, buttonIn: GuiButton) -> &mut GuiButton {
        self.buttonList.push(buttonIn);
        self.buttonList
            .last_mut()
            .expect("button was just inserted")
    }

    pub fn setWorldAndResolution(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
        self.buttonList.clear();
        self.initGui();
    }

    pub fn initGui(&mut self) {}

    pub fn onGuiClosed(&mut self) {}

    pub const fn doesGuiPauseGame(&self) -> bool {
        true
    }

    pub fn setFocused(&mut self, hasFocusedControlIn: bool) {
        self.focused = hasFocusedControlIn;
    }

    pub const fn isFocused(&self) -> bool {
        self.focused
    }

    pub fn selectedButton(&self) -> Option<&GuiButton> {
        self.selectedButton
            .and_then(|index| self.buttonList.get(index))
    }

    pub const fn eventButton(&self) -> i32 {
        self.eventButton
    }

    pub const fn lastMouseEvent(&self) -> i64 {
        self.lastMouseEvent
    }

    pub const fn touchValue(&self) -> i32 {
        self.touchValue
    }
}
