use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::vulkan::GuiDrawList::GuiDrawList;
#[derive(Debug, Clone)]
pub struct GuiDownloadTerrain {
    pub GuiScreen: GuiScreen,
    text: String,
}
impl GuiDownloadTerrain {
    pub fn new(text: String) -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            text,
        }
    }
    pub fn initGui(&mut self, width: i32, height: i32) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
    }
    pub fn drawScreen(
        &mut self,
        drawList: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.GuiScreen.drawWorldBackground(drawList, 0);
        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            &self.text,
            self.GuiScreen.width / 2,
            self.GuiScreen.height / 2 - 50,
            0x00FF_FFFF,
        );
        self.GuiScreen
            .drawScreen(drawList, font, mouseX, mouseY, partialTicks);
    }
}
