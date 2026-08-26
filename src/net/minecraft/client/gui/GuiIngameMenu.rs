use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiIngameMenuAction {
    Disconnect,
    ReturnToGame,
    Options,
    Advancements,
    Statistics,
    ShareToLan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiIngameMenuInteraction {
    pub action: GuiIngameMenuAction,
    pub sound: GuiSoundCommand,
}

/// MCP 1.12.2 `GuiIngameMenu` for a remote multiplayer world.
///
/// The exact button IDs, coordinates and labels are retained. Buttons whose
/// backing screens are not yet represented by real client state remain visibly
/// disabled rather than pretending to work.
#[derive(Debug, Clone)]
pub struct GuiIngameMenu {
    pub GuiScreen: GuiScreen,
    pub title: String,
    pub saveStep: i32,
    pub visibleTime: i32,
}

impl Default for GuiIngameMenu {
    fn default() -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            title: "Game menu".to_owned(),
            saveStep: 0,
            visibleTime: 0,
        }
    }
}

impl GuiIngameMenu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initGui(&mut self, width: i32, height: i32, locale: &Locale) {
        self.saveStep = 0;
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.title = locale.translate_key("menu.game").to_owned();

        self.GuiScreen.buttonList.push(GuiButton::new(
            1,
            width / 2 - 100,
            height / 4 + 104,
            locale.translate_key("menu.disconnect"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::new(
            4,
            width / 2 - 100,
            height / 4 + 8,
            locale.translate_key("menu.returnToGame"),
        ));
        self.GuiScreen.buttonList.push(GuiButton::newWithSize(
            0,
            width / 2 - 100,
            height / 4 + 80,
            98,
            20,
            locale.translate_key("menu.options"),
        ));
        let mut share = GuiButton::newWithSize(
            7,
            width / 2 + 2,
            height / 4 + 80,
            98,
            20,
            locale.translate_key("menu.shareToLan"),
        );
        // This runtime currently represents a remote multiplayer connection.
        share.enabled = false;
        self.GuiScreen.buttonList.push(share);

        let mut advancements = GuiButton::newWithSize(
            5,
            width / 2 - 100,
            height / 4 + 32,
            98,
            20,
            locale.translate_key("gui.advancements"),
        );
        advancements.enabled = false;
        self.GuiScreen.buttonList.push(advancements);

        let mut statistics = GuiButton::newWithSize(
            6,
            width / 2 + 2,
            height / 4 + 32,
            98,
            20,
            locale.translate_key("gui.stats"),
        );
        statistics.enabled = false;
        self.GuiScreen.buttonList.push(statistics);
    }

    pub fn updateScreen(&mut self) {
        self.visibleTime = self.visibleTime.saturating_add(1);
    }

    pub fn drawScreen(
        &mut self,
        drawList: &mut GuiDrawList,
        fontRendererObj: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.GuiScreen.drawDefaultBackgroundInWorld(drawList);
        self.GuiScreen.Gui.drawCenteredString(
            fontRendererObj,
            drawList,
            &self.title,
            self.GuiScreen.width / 2,
            40,
            0x00FF_FFFF,
        );
        self.GuiScreen
            .drawScreen(drawList, fontRendererObj, mouseX, mouseY, partialTicks);
    }

    pub fn mouseClicked(
        &self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
    ) -> Option<GuiIngameMenuInteraction> {
        if mouseButton != 0 {
            return None;
        }
        self.GuiScreen.buttonList.iter().find_map(|button| {
            if !button.mousePressed(mouseX, mouseY) {
                return None;
            }
            let action = match button.id {
                1 => GuiIngameMenuAction::Disconnect,
                4 => GuiIngameMenuAction::ReturnToGame,
                0 => GuiIngameMenuAction::Options,
                5 => GuiIngameMenuAction::Advancements,
                6 => GuiIngameMenuAction::Statistics,
                7 => GuiIngameMenuAction::ShareToLan,
                _ => return None,
            };
            Some(GuiIngameMenuInteraction {
                action,
                sound: button.playPressSound(),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_pause_menu_keeps_vanilla_ids_and_coordinates() {
        let mut locale = Locale::default();
        locale.load_bytes(
            concat!(
                "menu.game=Game menu\n",
                "menu.disconnect=Disconnect\n",
                "menu.returnToGame=Back to Game\n",
                "menu.options=Options...\n",
                "menu.shareToLan=Open to LAN\n",
                "gui.advancements=Advancements\n",
                "gui.stats=Statistics\n",
            )
            .as_bytes(),
        );
        let mut menu = GuiIngameMenu::new();
        menu.initGui(854, 480, &locale);
        let return_to_game = menu
            .GuiScreen
            .buttonList
            .iter()
            .find(|button| button.id == 4)
            .unwrap();
        assert_eq!((return_to_game.x, return_to_game.y), (327, 128));
        let disconnect = menu
            .GuiScreen
            .buttonList
            .iter()
            .find(|button| button.id == 1)
            .unwrap();
        assert_eq!((disconnect.x, disconnect.y), (327, 224));
        assert!(
            !menu
                .GuiScreen
                .buttonList
                .iter()
                .find(|button| button.id == 7)
                .unwrap()
                .enabled
        );
    }
}
