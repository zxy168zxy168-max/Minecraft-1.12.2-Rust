use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiGameOverAction {
    Respawn,
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiGameOverInteraction {
    pub action: GuiGameOverAction,
    pub sound: GuiSoundCommand,
}

/// Direct MCP 1.12.2 `GuiGameOver` port for the multiplayer world path.
#[derive(Debug, Clone)]
pub struct GuiGameOver {
    pub GuiScreen: GuiScreen,
    enableButtonsTimer: i32,
    causeOfDeath: Option<ITextComponent>,
    hardcore: bool,
    score: i32,
    title: String,
    scorePrefix: String,
}

impl GuiGameOver {
    pub fn new(causeOfDeath: Option<ITextComponent>, hardcore: bool, score: i32) -> Self {
        Self {
            GuiScreen: GuiScreen::default(),
            enableButtonsTimer: 0,
            causeOfDeath,
            hardcore,
            score,
            title: String::new(),
            scorePrefix: String::new(),
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32, locale: &Locale) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.enableButtonsTimer = 0;
        self.title = locale
            .translate_key(if self.hardcore {
                "deathScreen.title.hardcore"
            } else {
                "deathScreen.title"
            })
            .to_owned();
        self.scorePrefix = locale.translate_key("deathScreen.score").to_owned();

        let respawnLabel = locale.translate_key(if self.hardcore {
            "deathScreen.spectate"
        } else {
            "deathScreen.respawn"
        });
        let quitLabel = locale.translate_key(if self.hardcore {
            "deathScreen.leaveServer"
        } else {
            "deathScreen.titleScreen"
        });
        let mut respawn = GuiButton::new(0, width / 2 - 100, height / 4 + 72, respawnLabel);
        let mut quit = GuiButton::new(1, width / 2 - 100, height / 4 + 96, quitLabel);
        respawn.enabled = false;
        quit.enabled = false;
        self.GuiScreen.buttonList.push(respawn);
        self.GuiScreen.buttonList.push(quit);
    }

    pub fn updateScreen(&mut self) {
        self.enableButtonsTimer = self.enableButtonsTimer.saturating_add(1);
        if self.enableButtonsTimer == 20 {
            for button in &mut self.GuiScreen.buttonList {
                button.enabled = true;
            }
        }
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
        drawList.draw_gradient_rect(
            0,
            0,
            self.GuiScreen.width,
            self.GuiScreen.height,
            1_615_855_616,
            -1_602_211_792,
        );
        drawList.push_matrix();
        drawList.scale(2.0, 2.0);
        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            &self.title,
            self.GuiScreen.width / 4,
            30,
            0x00FF_FFFF,
        );
        drawList.pop_matrix();

        if let Some(cause) = &self.causeOfDeath {
            let cause = cause.resolveWithLocale(locale);
            self.GuiScreen.Gui.drawCenteredString(
                font,
                drawList,
                cause.getFormattedText(),
                self.GuiScreen.width / 2,
                85,
                0x00FF_FFFF,
            );
        }
        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            &format!("{}: §e{}", self.scorePrefix, self.score),
            self.GuiScreen.width / 2,
            100,
            0x00FF_FFFF,
        );
        self.GuiScreen
            .drawScreen(drawList, font, mouseX, mouseY, partialTicks);
    }

    pub fn mouseClicked(
        &self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
    ) -> Option<GuiGameOverInteraction> {
        if mouseButton != 0 {
            return None;
        }
        let button = self
            .GuiScreen
            .buttonList
            .iter()
            .find(|button| button.mousePressed(mouseX, mouseY))?;
        let action = match button.id {
            0 => GuiGameOverAction::Respawn,
            1 => GuiGameOverAction::Quit,
            _ => return None,
        };
        Some(GuiGameOverInteraction {
            action,
            sound: button.playPressSound(),
        })
    }

    pub const fn isHardcore(&self) -> bool {
        self.hardcore
    }
    pub const fn doesGuiPauseGame(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locale() -> Locale {
        let mut locale = Locale::default();
        locale.load_bytes(
            concat!(
                "deathScreen.title=You died!\n",
                "deathScreen.title.hardcore=Game over!\n",
                "deathScreen.score=Score\n",
                "deathScreen.respawn=Respawn\n",
                "deathScreen.spectate=Spectate world\n",
                "deathScreen.titleScreen=Title screen\n",
                "deathScreen.leaveServer=Leave server\n",
            )
            .as_bytes(),
        );
        locale
    }

    #[test]
    fn buttons_match_vanilla_positions_and_unlock_after_twenty_ticks() {
        let mut screen = GuiGameOver::new(None, false, 7);
        screen.initGui(854, 480, &locale());
        assert_eq!(
            (
                screen.GuiScreen.buttonList[0].x,
                screen.GuiScreen.buttonList[0].y
            ),
            (327, 192)
        );
        assert!(!screen.GuiScreen.buttonList[0].enabled);
        for _ in 0..20 {
            screen.updateScreen();
        }
        assert!(screen
            .GuiScreen
            .buttonList
            .iter()
            .all(|button| button.enabled));
    }
}
