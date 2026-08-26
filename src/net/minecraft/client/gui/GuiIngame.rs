use crate::compat::Java::JavaRandom;
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiOverlayDebug::{DebugOverlayData, GuiOverlayDebug};
use crate::net::minecraft::client::gui::GuiSubtitleOverlay::GuiSubtitleOverlay;
use crate::net::minecraft::scoreboard::ScorePlayerTeam::ScorePlayerTeam;
use crate::net::minecraft::scoreboard::Scoreboard::Scoreboard;
use crate::net::minecraft::util::EnumHandSide::EnumHandSide;
use crate::net::minecraft::world::GameType::GameType;

const REGENERATION_POTION_ID: u8 = 10;
const HUNGER_POTION_ID: u8 = 17;
const POISON_POTION_ID: u8 = 19;
const WITHER_POTION_ID: u8 = 20;

/// Source texture used by a `GuiIngame` textured rectangle. The Vulkan
/// backend resolves these logical MCP resources through the shared atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudTexture {
    Widgets,
    Icons,
    BossBars,
    /// `GuiContainer.INVENTORY_BACKGROUND` (`container/inventory.png`).
    Inventory,
}

/// Backend-neutral equivalent of `Gui.drawTexturedModalRect` arguments.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HudTexturedQuad {
    pub texture: HudTexture,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub textureX: i32,
    pub textureY: i32,
    pub textureWidth: i32,
    pub textureHeight: i32,
    /// Alpha multiplier used by potion-effect fading.
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudSolidRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub color: u32,
}

impl HudSolidRect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32, color: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            color,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudText {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub color: u32,
    pub outline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudScaledText {
    pub text: HudText,
    pub scale: i32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct IngameHudFrame {
    pub hotbar: Vec<HudTexturedQuad>,
    pub crosshair: Vec<HudTexturedQuad>,
    pub playerStats: Vec<HudTexturedQuad>,
    pub potionEffects: Vec<HudTexturedQuad>,
    pub experienceBar: Vec<HudTexturedQuad>,
    pub experienceLevel: Option<HudText>,
    pub scoreboardRectangles: Vec<HudSolidRect>,
    pub scoreboardTexts: Vec<HudText>,
    pub actionBar: Option<HudText>,
    pub subtitleRectangles: Vec<HudSolidRect>,
    pub subtitleTexts: Vec<HudText>,
    pub titleTexts: Vec<HudScaledText>,
    pub debugRectangles: Vec<HudSolidRect>,
    pub debugTexts: Vec<HudText>,
}

/// MCP-facing owner for the ported `GuiIngame` overlay branches.
///
/// The health history fields are stateful in vanilla: damage and healing flash
/// the previous value for 20 or 10 GUI ticks. Keeping them here rather than in
/// Vulkan preserves the original class responsibility and update semantics.
pub struct GuiIngame {
    updateCounter: i32,
    playerHealth: i32,
    lastPlayerHealth: i32,
    lastSystemTime: u64,
    healthUpdateCounter: i64,
    rand: JavaRandom,
    titleFadeIn: i32,
    titleDisplayTime: i32,
    titleFadeOut: i32,
    titlesTimer: i32,
    displayedTitle: String,
    displayedSubTitle: String,
    overlayMessage: String,
    overlayMessageTime: i32,
    subtitleOverlay: GuiSubtitleOverlay,
    overlayDebug: GuiOverlayDebug,
}

impl Default for GuiIngame {
    fn default() -> Self {
        Self {
            updateCounter: 0,
            playerHealth: 0,
            lastPlayerHealth: 0,
            lastSystemTime: 0,
            healthUpdateCounter: 0,
            rand: JavaRandom::new(0),
            titleFadeIn: 10,
            titleDisplayTime: 70,
            titleFadeOut: 20,
            titlesTimer: 0,
            displayedTitle: String::new(),
            displayedSubTitle: String::new(),
            overlayMessage: String::new(),
            overlayMessageTime: 0,
            subtitleOverlay: GuiSubtitleOverlay::new(),
            overlayDebug: GuiOverlayDebug::new(),
        }
    }
}

impl GuiIngame {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ports the normal-player branches of `GuiIngame.renderHotbar`,
    /// `renderAttackIndicator`, and `renderPlayerStats` from 1.12.2.
    ///
    /// Player health/food variants, armor, air and mount health follow the
    /// source HUD coordinates and ordering; synchronized gameplay state is
    /// supplied by the renderer capture rather than reconstructed here.
    #[allow(clippy::too_many_arguments)]
    pub fn buildFrameWithFont(
        &mut self,
        guiWidth: i32,
        guiHeight: i32,
        currentHotbarSlot: i32,
        offhandNonEmpty: bool,
        primaryHand: EnumHandSide,
        gameType: GameType,
        playerHealth: f32,
        playerMaxHealth: f32,
        absorptionAmount: f32,
        foodLevel: i32,
        saturationLevel: f32,
        armorValue: i32,
        air: i32,
        inWater: bool,
        hardcoreMode: bool,
        activePotionEffects: &[crate::net::minecraft::potion::PotionEffect::PotionEffect],
        ridingLivingEntity: bool,
        mountHealth: Option<(f32, f32)>,
        horseJumpPower: Option<f32>,
        experience: f32,
        experienceLevel: i32,
        xpBarCap: i32,
        hurtResistantTime: i32,
        playerTicksExisted: i32,
        systemTimeMillis: u64,
        scoreboard: Option<&Scoreboard>,
        localPlayerName: &str,
        actionBarText: Option<&str>,
        actionBarAge: i32,
        partialTicks: f32,
        showSubtitles: bool,
        listenerPosition: [f32; 3],
        listenerYaw: f32,
        listenerPitch: f32,
        debugData: Option<&DebugOverlayData>,
        fontRenderer: &FontRenderer,
    ) -> IngameHudFrame {
        let guiWidth = guiWidth.max(1);
        let guiHeight = guiHeight.max(1);
        self.updateCounter = playerTicksExisted;
        let mut frame = IngameHudFrame::default();

        if gameType != GameType::Spectator {
            let center = guiWidth / 2;
            frame.hotbar.push(HudTexturedQuad {
                texture: HudTexture::Widgets,
                x: center - 91,
                y: guiHeight - 22,
                width: 182,
                height: 22,
                textureX: 0,
                textureY: 0,
                textureWidth: 182,
                textureHeight: 22,
                alpha: 1.0,
            });
            frame.hotbar.push(HudTexturedQuad {
                texture: HudTexture::Widgets,
                x: center - 92 + currentHotbarSlot.clamp(0, 8) * 20,
                y: guiHeight - 23,
                width: 24,
                height: 22,
                textureX: 0,
                textureY: 22,
                textureWidth: 24,
                textureHeight: 22,
                alpha: 1.0,
            });

            if offhandNonEmpty {
                // `EntityPlayer.getPrimaryHand().opposite()` selects the side
                // occupied by the offhand frame in the original method.
                let offhandSide = match primaryHand {
                    EnumHandSide::Right => EnumHandSide::Left,
                    EnumHandSide::Left => EnumHandSide::Right,
                };
                let (x, textureX) = if offhandSide == EnumHandSide::Left {
                    (center - 120, 24)
                } else {
                    (center + 91, 53)
                };
                frame.hotbar.push(HudTexturedQuad {
                    texture: HudTexture::Widgets,
                    x,
                    y: guiHeight - 23,
                    width: 29,
                    height: 24,
                    textureX,
                    textureY: 22,
                    textureWidth: 29,
                    textureHeight: 24,
                    alpha: 1.0,
                });
            }

            frame.crosshair.push(HudTexturedQuad {
                texture: HudTexture::Icons,
                x: guiWidth / 2 - 7,
                y: guiHeight / 2 - 7,
                width: 16,
                height: 16,
                textureX: 0,
                textureY: 0,
                textureWidth: 16,
                textureHeight: 16,
                alpha: 1.0,
            });
        }

        if gameType.isSurvivalOrAdventure() {
            self.appendPlayerStats(
                &mut frame.playerStats,
                guiWidth,
                guiHeight,
                playerHealth,
                playerMaxHealth,
                absorptionAmount,
                foodLevel,
                saturationLevel,
                armorValue,
                air,
                inWater,
                hardcoreMode,
                activePotionEffects,
                !ridingLivingEntity,
                hurtResistantTime,
                systemTimeMillis,
            );
        }

        // MCP GuiIngame chooses the horse jump bar before the ordinary XP
        // bar: a riding IJumpingMount replaces XP even outside the normal
        // survival/adventure branch.
        if let Some(power) = horseJumpPower {
            self.appendHorseJumpBar(&mut frame.experienceBar, guiWidth, guiHeight, power);
        } else if gameType.isSurvivalOrAdventure() {
            self.appendExperience(
                &mut frame,
                guiWidth,
                guiHeight,
                experience,
                experienceLevel,
                xpBarCap,
            );
        }

        // MCP `GuiIngame#renderGameOverlay` calls `renderMountHealth` after
        // `renderPlayerStats` and outside the shouldDrawHUD gate. Food is
        // suppressed above whenever the ridden entity is an EntityLivingBase.
        if let Some((mountHealth, mountMaxHealth)) = mountHealth {
            self.appendMountHealth(
                &mut frame.playerStats,
                guiWidth,
                guiHeight,
                mountHealth,
                mountMaxHealth,
            );
        }

        // MCP `GuiIngame#renderGameOverlay` invokes renderPotionEffects outside
        // the survival/adventure player-stat gate.
        self.appendPotionEffects(&mut frame.potionEffects, guiWidth, activePotionEffects);

        if let Some(scoreboard) = scoreboard {
            self.appendScoreboard(
                &mut frame,
                guiWidth,
                guiHeight,
                scoreboard,
                localPlayerName,
                fontRenderer,
            );
        }
        let (overlayText, overlayRemaining) = if self.overlayMessageTime > 0 {
            (Some(self.overlayMessage.as_str()), self.overlayMessageTime)
        } else {
            (actionBarText, 60 - actionBarAge.max(0))
        };
        if let Some(text) = overlayText {
            let alpha = ((overlayRemaining * 255 / 20).min(255)).max(0);
            if alpha > 8 {
                frame.actionBar = Some(HudText {
                    text: text.to_owned(),
                    x: guiWidth / 2 - fontRenderer.get_string_width(text) / 2,
                    y: guiHeight - 68 - 4,
                    color: ((alpha as u32) << 24) | 0x00FF_FFFF,
                    outline: false,
                });
            }
        }

        let subtitles = self.subtitleOverlay.buildFrame(
            showSubtitles,
            guiWidth,
            guiHeight,
            listenerPosition,
            listenerYaw,
            listenerPitch,
            systemTimeMillis,
            fontRenderer,
        );
        frame.subtitleRectangles = subtitles.rectangles;
        frame.subtitleTexts = subtitles.texts;

        if self.titlesTimer > 0 {
            let remaining = self.titlesTimer as f32 - partialTicks.clamp(0.0, 1.0);
            let mut alpha = 255_i32;
            if self.titlesTimer > self.titleFadeOut + self.titleDisplayTime && self.titleFadeIn > 0
            {
                let elapsed = (self.titleFadeIn + self.titleDisplayTime + self.titleFadeOut) as f32
                    - remaining;
                alpha = (elapsed * 255.0 / self.titleFadeIn as f32) as i32;
            }
            if self.titlesTimer <= self.titleFadeOut && self.titleFadeOut > 0 {
                alpha = (remaining * 255.0 / self.titleFadeOut as f32) as i32;
            }
            alpha = alpha.clamp(0, 255);
            if alpha > 8 {
                let color = ((alpha as u32) << 24) | 0x00FF_FFFF;
                frame.titleTexts.push(HudScaledText {
                    scale: 4,
                    text: HudText {
                        text: self.displayedTitle.clone(),
                        x: guiWidth / 2 - fontRenderer.get_string_width(&self.displayedTitle) * 2,
                        y: guiHeight / 2 - 40,
                        color,
                        outline: true,
                    },
                });
                frame.titleTexts.push(HudScaledText {
                    scale: 2,
                    text: HudText {
                        text: self.displayedSubTitle.clone(),
                        x: guiWidth / 2 - fontRenderer.get_string_width(&self.displayedSubTitle),
                        y: guiHeight / 2 + 10,
                        color,
                        outline: true,
                    },
                });
            }
        }

        if let Some(debugData) = debugData {
            let debug = self
                .overlayDebug
                .buildFrame(guiWidth, debugData, fontRenderer);
            frame.debugRectangles = debug.rectangles;
            frame.debugTexts = debug.texts;
        }

        frame
    }

    #[allow(clippy::too_many_arguments)]
    pub fn buildFrame(
        &mut self,
        guiWidth: i32,
        guiHeight: i32,
        currentHotbarSlot: i32,
        offhandNonEmpty: bool,
        primaryHand: EnumHandSide,
        gameType: GameType,
        playerHealth: f32,
        absorptionAmount: f32,
        foodLevel: i32,
        saturationLevel: f32,
        experience: f32,
        experienceLevel: i32,
        xpBarCap: i32,
        hurtResistantTime: i32,
        playerTicksExisted: i32,
        systemTimeMillis: u64,
        scoreboard: Option<&Scoreboard>,
        localPlayerName: &str,
        actionBarText: Option<&str>,
        actionBarAge: i32,
    ) -> IngameHudFrame {
        let fontRenderer = FontRenderer::test_metric_renderer();
        self.buildFrameWithFont(
            guiWidth,
            guiHeight,
            currentHotbarSlot,
            offhandNonEmpty,
            primaryHand,
            gameType,
            playerHealth,
            20.0,
            absorptionAmount,
            foodLevel,
            saturationLevel,
            0,
            300,
            false,
            false,
            &[],
            false,
            None,
            None,
            experience,
            experienceLevel,
            xpBarCap,
            hurtResistantTime,
            playerTicksExisted,
            systemTimeMillis,
            scoreboard,
            localPlayerName,
            actionBarText,
            actionBarAge,
            0.0,
            false,
            [0.0; 3],
            0.0,
            0.0,
            None,
            &fontRenderer,
        )
    }

    pub fn soundPlay(
        &mut self,
        subtitle: impl Into<String>,
        location: [f32; 3],
        systemTimeMillis: u64,
    ) {
        self.subtitleOverlay
            .soundPlay(subtitle, location, systemTimeMillis);
    }

    pub fn setDefaultTitlesTimes(&mut self) {
        self.titleFadeIn = 10;
        self.titleDisplayTime = 70;
        self.titleFadeOut = 20;
    }

    pub fn updateTick(&mut self) {
        if self.overlayMessageTime > 0 {
            self.overlayMessageTime -= 1;
        }
        if self.titlesTimer > 0 {
            self.titlesTimer -= 1;
            if self.titlesTimer <= 0 {
                self.displayedTitle.clear();
                self.displayedSubTitle.clear();
            }
        }
        self.updateCounter = self.updateCounter.wrapping_add(1);
    }

    pub fn setOverlayMessage(&mut self, message: impl Into<String>) {
        self.overlayMessage = message.into();
        self.overlayMessageTime = 60;
    }

    pub fn displayTitle(
        &mut self,
        title: Option<&str>,
        subTitle: Option<&str>,
        timeFadeIn: i32,
        displayTime: i32,
        timeFadeOut: i32,
    ) {
        if title.is_none()
            && subTitle.is_none()
            && timeFadeIn < 0
            && displayTime < 0
            && timeFadeOut < 0
        {
            self.displayedTitle.clear();
            self.displayedSubTitle.clear();
            self.titlesTimer = 0;
        } else if let Some(title) = title {
            self.displayedTitle = title.to_owned();
            self.titlesTimer = self.titleFadeIn + self.titleDisplayTime + self.titleFadeOut;
        } else if let Some(subTitle) = subTitle {
            self.displayedSubTitle = subTitle.to_owned();
        } else {
            if timeFadeIn >= 0 {
                self.titleFadeIn = timeFadeIn;
            }
            if displayTime >= 0 {
                self.titleDisplayTime = displayTime;
            }
            if timeFadeOut >= 0 {
                self.titleFadeOut = timeFadeOut;
            }
            if self.titlesTimer > 0 {
                self.titlesTimer = self.titleFadeIn + self.titleDisplayTime + self.titleFadeOut;
            }
        }
    }

    fn appendScoreboard(
        &self,
        frame: &mut IngameHudFrame,
        guiWidth: i32,
        guiHeight: i32,
        scoreboard: &Scoreboard,
        localPlayerName: &str,
        fontRenderer: &FontRenderer,
    ) {
        let Some(objective) = scoreboard.getSidebarObjective(localPlayerName) else {
            return;
        };
        let mut scores = scoreboard
            .getSortedScores(objective)
            .into_iter()
            .filter(|score| !score.getPlayerName().starts_with('#'))
            .collect::<Vec<_>>();
        if scores.len() > 15 {
            scores.drain(0..scores.len() - 15);
        }
        let mut maximumWidth = fontRenderer.get_string_width(objective.getDisplayName());
        for score in &scores {
            let name = ScorePlayerTeam::formatPlayerName(
                scoreboard.getPlayersTeam(score.getPlayerName()),
                score.getPlayerName(),
            );
            let combined = format!("{name}: §c{}", score.getScorePoints());
            maximumWidth = maximumWidth.max(fontRenderer.get_string_width(&combined));
        }
        let totalHeight = scores.len() as i32 * 9;
        let bottom = guiHeight / 2 + totalHeight / 3;
        let left = guiWidth - maximumWidth - 3;
        let right = guiWidth - 1;
        for (index, score) in scores.iter().enumerate() {
            let row = index as i32 + 1;
            let y = bottom - row * 9;
            let name = ScorePlayerTeam::formatPlayerName(
                scoreboard.getPlayersTeam(score.getPlayerName()),
                score.getPlayerName(),
            );
            let points = format!("§c{}", score.getScorePoints());
            frame.scoreboardRectangles.push(HudSolidRect::new(
                left - 2,
                y,
                right - (left - 2),
                9,
                0x5000_0000,
            ));
            frame.scoreboardTexts.push(HudText {
                text: name,
                x: left,
                y,
                color: 0x20FF_FFFF,
                outline: false,
            });
            frame.scoreboardTexts.push(HudText {
                text: points.clone(),
                x: right - fontRenderer.get_string_width(&points),
                y,
                color: 0x20FF_FFFF,
                outline: false,
            });
            if row == scores.len() as i32 {
                frame.scoreboardRectangles.push(HudSolidRect::new(
                    left - 2,
                    y - 10,
                    right - (left - 2),
                    9,
                    0x6000_0000,
                ));
                frame.scoreboardRectangles.push(HudSolidRect::new(
                    left - 2,
                    y - 1,
                    right - (left - 2),
                    1,
                    0x5000_0000,
                ));
                let title = objective.getDisplayName();
                frame.scoreboardTexts.push(HudText {
                    text: title.to_owned(),
                    x: left + maximumWidth / 2 - fontRenderer.get_string_width(title) / 2,
                    y: y - 9,
                    color: 0x20FF_FFFF,
                    outline: false,
                });
            }
        }
    }

    fn appendExperience(
        &self,
        frame: &mut IngameHudFrame,
        guiWidth: i32,
        guiHeight: i32,
        experience: f32,
        experienceLevel: i32,
        xpBarCap: i32,
    ) {
        if xpBarCap > 0 {
            let x = guiWidth / 2 - 91;
            let y = guiHeight - 32 + 3;
            frame.experienceBar.push(HudTexturedQuad {
                texture: HudTexture::Icons,
                x,
                y,
                width: 182,
                height: 5,
                textureX: 0,
                textureY: 64,
                textureWidth: 182,
                textureHeight: 5,
                alpha: 1.0,
            });
            let filled = (experience * 183.0) as i32;
            if filled > 0 {
                frame.experienceBar.push(HudTexturedQuad {
                    texture: HudTexture::Icons,
                    x,
                    y,
                    width: filled,
                    height: 5,
                    textureX: 0,
                    textureY: 69,
                    textureWidth: filled,
                    textureHeight: 5,
                    alpha: 1.0,
                });
            }
        }

        if experienceLevel > 0 {
            let text = experienceLevel.to_string();
            let width = text.chars().count() as i32 * 6;
            frame.experienceLevel = Some(HudText {
                text,
                x: (guiWidth - width) / 2,
                y: guiHeight - 31 - 4,
                color: 8_453_920,
                outline: true,
            });
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn appendPlayerStats(
        &mut self,
        output: &mut Vec<HudTexturedQuad>,
        guiWidth: i32,
        guiHeight: i32,
        health: f32,
        maxHealth: f32,
        absorptionAmount: f32,
        foodLevel: i32,
        saturationLevel: f32,
        armorValue: i32,
        air: i32,
        inWater: bool,
        hardcore: bool,
        effects: &[crate::net::minecraft::potion::PotionEffect::PotionEffect],
        showFood: bool,
        hurtResistantTime: i32,
        systemTimeMillis: u64,
    ) {
        let currentHealth = health.ceil().max(0.0) as i32;
        let flash = self.healthUpdateCounter > self.updateCounter as i64
            && (self.healthUpdateCounter - self.updateCounter as i64) / 3 % 2 == 1;

        if currentHealth < self.playerHealth && hurtResistantTime > 0 {
            self.lastSystemTime = systemTimeMillis;
            self.healthUpdateCounter = (self.updateCounter + 20) as i64;
        } else if currentHealth > self.playerHealth && hurtResistantTime > 0 {
            self.lastSystemTime = systemTimeMillis;
            self.healthUpdateCounter = (self.updateCounter + 10) as i64;
        }

        if systemTimeMillis.saturating_sub(self.lastSystemTime) > 1_000 {
            self.playerHealth = currentHealth;
            self.lastPlayerHealth = currentHealth;
            self.lastSystemTime = systemTimeMillis;
        }

        self.playerHealth = currentHealth;
        let previousHealth = self.lastPlayerHealth;
        self.rand
            .set_seed((self.updateCounter.wrapping_mul(312_871)) as i64);

        let foodLevel = foodLevel.clamp(0, 20);
        let left = guiWidth / 2 - 91;
        let right = guiWidth / 2 + 91;
        let baseline = guiHeight - 39;

        // MCP `EntityLivingBase#getMaxHealth`: use the synchronized
        // generic.maxHealth attribute captured from the local player.
        let maxHealth = maxHealth.max(1.0);
        let absorption = absorptionAmount.ceil().max(0.0) as i32;
        let heartRows = (((maxHealth + absorption as f32) / 2.0).ceil() as i32 + 9) / 10;
        let rowHeight = (10 - (heartRows - 2)).max(3);
        let totalHeartSlots = ((maxHealth + absorption as f32) / 2.0).ceil() as i32;
        let mut absorptionRemaining = absorption;

        let regenFlashHeart = if effects
            .iter()
            .any(|effect| effect.getPotionId() == REGENERATION_POTION_ID)
        {
            self.updateCounter
                .rem_euclid((maxHealth + 5.0).ceil() as i32)
        } else {
            -1
        };
        let heartTextureY = if hardcore { 45 } else { 0 };
        let heartVariantOffset = if effects
            .iter()
            .any(|effect| effect.getPotionId() == POISON_POTION_ID)
        {
            36
        } else if effects
            .iter()
            .any(|effect| effect.getPotionId() == WITHER_POTION_ID)
        {
            72
        } else {
            0
        };

        let armorY = baseline - (heartRows - 1) * rowHeight - 10;
        for armorIndex in 0..10 {
            if armorValue > 0 {
                let x = left + armorIndex * 8;
                let (textureX, textureY) = if armorIndex * 2 + 1 < armorValue {
                    (34, 9)
                } else if armorIndex * 2 + 1 == armorValue {
                    (25, 9)
                } else {
                    (16, 9)
                };
                output.push(icon_quad(x, armorY, textureX, textureY));
            }
        }

        for heartIndex in (0..totalHeartSlots).rev() {
            let row = (heartIndex + 1 + 9) / 10 - 1;
            let mut y = baseline - row * rowHeight;
            if currentHealth <= 4 {
                y += self.rand.next_i32_bound(2);
            }
            if absorptionRemaining <= 0 && heartIndex == regenFlashHeart {
                y -= 2;
            }
            let x = left + heartIndex.rem_euclid(10) * 8;
            output.push(icon_quad(
                x,
                y,
                16 + if flash { 9 } else { 0 },
                heartTextureY,
            ));

            if flash {
                if heartIndex * 2 + 1 < previousHealth {
                    output.push(icon_quad(x, y, 16 + heartVariantOffset + 54, heartTextureY));
                } else if heartIndex * 2 + 1 == previousHealth {
                    output.push(icon_quad(x, y, 16 + heartVariantOffset + 63, heartTextureY));
                }
            }

            if absorptionRemaining > 0 {
                if absorptionRemaining == absorption && absorption % 2 == 1 {
                    output.push(icon_quad(
                        x,
                        y,
                        16 + heartVariantOffset + 153,
                        heartTextureY,
                    ));
                    absorptionRemaining -= 1;
                } else {
                    output.push(icon_quad(
                        x,
                        y,
                        16 + heartVariantOffset + 144,
                        heartTextureY,
                    ));
                    absorptionRemaining -= 2;
                }
            } else if heartIndex * 2 + 1 < currentHealth {
                output.push(icon_quad(x, y, 16 + heartVariantOffset + 36, heartTextureY));
            } else if heartIndex * 2 + 1 == currentHealth {
                output.push(icon_quad(x, y, 16 + heartVariantOffset + 45, heartTextureY));
            }
        }

        if showFood {
            let (foodBackgroundX, foodFullX, foodHalfX) = if effects
                .iter()
                .any(|effect| effect.getPotionId() == HUNGER_POTION_ID)
            {
                (133, 88, 97)
            } else {
                (16, 52, 61)
            };
            for foodIndex in 0..10 {
                let mut y = baseline;
                if saturationLevel <= 0.0 && self.updateCounter.rem_euclid(foodLevel * 3 + 1) == 0 {
                    y += self.rand.next_i32_bound(3) - 1;
                }
                let x = right - foodIndex * 8 - 9;
                output.push(icon_quad(x, y, foodBackgroundX, 27));
                if foodIndex * 2 + 1 < foodLevel {
                    output.push(icon_quad(x, y, foodFullX, 27));
                } else if foodIndex * 2 + 1 == foodLevel {
                    output.push(icon_quad(x, y, foodHalfX, 27));
                }
            }
        }

        if inWater {
            let consumed = ((air - 2) as f64 * 10.0 / 300.0).ceil() as i32;
            let remaining = (air as f64 * 10.0 / 300.0).ceil() as i32 - consumed;
            for bubble in 0..(consumed + remaining) {
                let x = right - bubble * 8 - 9;
                let (textureX, textureY) = if bubble < consumed {
                    (16, 18)
                } else {
                    (25, 18)
                };
                output.push(icon_quad(x, baseline - 10, textureX, textureY));
            }
        }
    }

    /// MCP `GuiIngame#renderHorseJumpBar`: the riding IJumpingMount charge
    /// replaces the XP bar, using ICONS rows 84/89 and 182px background.
    fn appendHorseJumpBar(
        &self,
        output: &mut Vec<HudTexturedQuad>,
        guiWidth: i32,
        guiHeight: i32,
        horseJumpPower: f32,
    ) {
        let x = guiWidth / 2 - 91;
        let y = guiHeight - 32 + 3;
        output.push(HudTexturedQuad {
            texture: HudTexture::Icons,
            x,
            y,
            width: 182,
            height: 5,
            textureX: 0,
            textureY: 84,
            textureWidth: 182,
            textureHeight: 5,
            alpha: 1.0,
        });
        let filled = (horseJumpPower * 183.0) as i32;
        if filled > 0 {
            output.push(HudTexturedQuad {
                texture: HudTexture::Icons,
                x,
                y,
                width: filled,
                height: 5,
                textureX: 0,
                textureY: 89,
                textureWidth: filled,
                textureHeight: 5,
                alpha: 1.0,
            });
        }
    }

    /// MCP `GuiIngame#renderMountHealth`: up to 30 half-hearts, ten per
    /// row, right-aligned above the hotbar. The caller supplies only
    /// synchronized EntityLivingBase health/max-health state.
    fn appendMountHealth(
        &self,
        output: &mut Vec<HudTexturedQuad>,
        guiWidth: i32,
        guiHeight: i32,
        health: f32,
        maxHealth: f32,
    ) {
        let currentHealth = health.ceil() as i32;
        let mut halfHearts = ((maxHealth + 0.5) as i32) / 2;
        halfHearts = halfHearts.min(30);
        let right = guiWidth / 2 + 91;
        let mut y = guiHeight - 39;
        let mut rowOffset = 0;

        while halfHearts > 0 {
            let rowCount = halfHearts.min(10);
            halfHearts -= rowCount;
            for index in 0..rowCount {
                let x = right - index * 8 - 9;
                output.push(icon_quad(x, y, 52, 9));
                let healthIndex = index * 2 + 1 + rowOffset;
                if healthIndex < currentHealth {
                    output.push(icon_quad(x, y, 88, 9));
                } else if healthIndex == currentHealth {
                    output.push(icon_quad(x, y, 97, 9));
                }
            }
            y -= 10;
            rowOffset += 20;
        }
    }

    /// MCP `GuiIngame#renderPotionEffects`.
    fn appendPotionEffects(
        &self,
        output: &mut Vec<HudTexturedQuad>,
        guiWidth: i32,
        effects: &[crate::net::minecraft::potion::PotionEffect::PotionEffect],
    ) {
        if effects.is_empty() {
            return;
        }
        let mut beneficialCount = 0;
        let mut harmfulCount = 0;
        let mut sorted = effects.to_vec();
        // `Ordering.natural().reverse().sortedCopy(collection)` uses
        // PotionEffect#compareTo, not duration alone. Preserve the ambient,
        // duration-threshold and potion liquid-colour tie breakers.
        sorted.sort_by(|a, b| {
            let a_color = crate::net::minecraft::potion::Potion::potion_meta(a.getPotionId())
                .map_or(0, |meta| meta.liquidColor);
            let b_color = crate::net::minecraft::potion::Potion::potion_meta(b.getPotionId())
                .map_or(0, |meta| meta.liquidColor);
            let natural = if (a.getDuration() <= 32_147 || b.getDuration() <= 32_147)
                && (!a.getIsAmbient() || !b.getIsAmbient())
            {
                a.getIsAmbient()
                    .cmp(&b.getIsAmbient())
                    .then_with(|| a.getDuration().cmp(&b.getDuration()))
                    .then_with(|| a_color.cmp(&b_color))
            } else {
                a.getIsAmbient()
                    .cmp(&b.getIsAmbient())
                    .then_with(|| a_color.cmp(&b_color))
            };
            natural.reverse()
        });
        for effect in &sorted {
            let Some(meta) =
                crate::net::minecraft::potion::Potion::potion_meta(effect.getPotionId())
            else {
                continue;
            };
            if !meta.hasStatusIcon() || !effect.doesShowParticles() {
                continue;
            }
            let mut x = guiWidth;
            let mut y = 1;
            if meta.beneficial {
                beneficialCount += 1;
                x -= 25 * beneficialCount;
            } else {
                harmfulCount += 1;
                x -= 25 * harmfulCount;
                y += 26;
            }
            let mut alpha = 1.0_f32;
            if !effect.getIsAmbient() && effect.getDuration() <= 200 {
                let flash = 10 - effect.getDuration() / 20;
                alpha = (effect.getDuration() as f32 / 10.0 / 5.0 * 0.5).clamp(0.0, 0.5)
                    + (effect.getDuration() as f32 * std::f32::consts::PI / 5.0).cos()
                        * (flash as f32 / 10.0 * 0.25).clamp(0.0, 0.25);
            }
            let (frameX, frameY) = if effect.getIsAmbient() {
                (165, 166)
            } else {
                (141, 166)
            };
            output.push(HudTexturedQuad {
                texture: HudTexture::Inventory,
                x,
                y,
                width: 24,
                height: 24,
                textureX: frameX,
                textureY: frameY,
                textureWidth: 24,
                textureHeight: 24,
                alpha: 1.0,
            });
            let (iconX, iconY) = meta.iconRect();
            output.push(HudTexturedQuad {
                texture: HudTexture::Inventory,
                x: x + 3,
                y: y + 3,
                width: 18,
                height: 18,
                textureX: iconX,
                textureY: iconY,
                textureWidth: 18,
                textureHeight: 18,
                alpha,
            });
        }
    }
}

const fn icon_quad(x: i32, y: i32, textureX: i32, textureY: i32) -> HudTexturedQuad {
    HudTexturedQuad {
        texture: HudTexture::Icons,
        x,
        y,
        width: 9,
        height: 9,
        textureX,
        textureY,
        textureWidth: 9,
        textureHeight: 9,
        alpha: 1.0,
    }
}

fn visible_width(text: &str) -> i32 {
    let mut width = 0;
    let mut formatting = false;
    for character in text.chars() {
        if formatting {
            formatting = false;
            continue;
        }
        if character == '§' {
            formatting = true;
        } else {
            width += 6;
        }
    }
    width
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(gui: &mut GuiIngame, gameType: GameType) -> IngameHudFrame {
        gui.buildFrame(
            320,
            180,
            4,
            false,
            EnumHandSide::Right,
            gameType,
            20.0,
            0.0,
            20,
            5.0,
            0.0,
            0,
            7,
            0,
            1,
            2_000,
            None,
            "",
            None,
            0,
        )
    }

    #[test]
    fn hotbar_coordinates_match_mcp_render_hotbar() {
        let mut gui = GuiIngame::new();
        let frame = frame(&mut gui, GameType::Survival);
        assert_eq!(
            frame.hotbar[0],
            HudTexturedQuad {
                texture: HudTexture::Widgets,
                x: 69,
                y: 158,
                width: 182,
                height: 22,
                textureX: 0,
                textureY: 0,
                textureWidth: 182,
                textureHeight: 22,
                alpha: 1.0,
            }
        );
        assert_eq!(frame.hotbar[1].x, 148);
        assert_eq!(frame.hotbar[1].y, 157);
        assert_eq!(frame.crosshair[0].x, 153);
        assert_eq!(frame.crosshair[0].y, 83);
    }

    #[test]
    fn normal_health_and_food_coordinates_match_render_player_stats() {
        let mut gui = GuiIngame::new();
        let frame = frame(&mut gui, GameType::Survival);
        assert!(frame.playerStats.contains(&icon_quad(69, 141, 16, 0)));
        assert!(frame.playerStats.contains(&icon_quad(69, 141, 52, 0)));
        assert!(frame.playerStats.contains(&icon_quad(242, 141, 16, 27)));
        assert!(frame.playerStats.contains(&icon_quad(242, 141, 52, 27)));
    }

    #[test]
    fn absorption_uses_vanilla_yellow_heart_uvs() {
        let mut gui = GuiIngame::new();
        let frame = gui.buildFrame(
            320,
            180,
            0,
            false,
            EnumHandSide::Right,
            GameType::Survival,
            20.0,
            4.0,
            20,
            5.0,
            0.0,
            0,
            7,
            0,
            1,
            2_000,
            None,
            "",
            None,
            0,
        );
        assert!(frame
            .playerStats
            .iter()
            .any(|quad| quad.textureX == 160 && quad.textureY == 0));
    }

    #[test]
    fn offhand_frame_uses_primary_hand_opposite() {
        let mut gui = GuiIngame::new();
        let rightHanded = gui.buildFrame(
            320,
            180,
            0,
            true,
            EnumHandSide::Right,
            GameType::Survival,
            20.0,
            0.0,
            20,
            5.0,
            0.0,
            0,
            7,
            0,
            1,
            2_000,
            None,
            "",
            None,
            0,
        );
        assert_eq!(rightHanded.hotbar[2].x, 40);
        assert_eq!(rightHanded.hotbar[2].textureX, 24);

        let leftHanded = gui.buildFrame(
            320,
            180,
            0,
            true,
            EnumHandSide::Left,
            GameType::Survival,
            20.0,
            0.0,
            20,
            5.0,
            0.0,
            0,
            7,
            0,
            2,
            2_050,
            None,
            "",
            None,
            0,
        );
        assert_eq!(leftHanded.hotbar[2].x, 251);
        assert_eq!(leftHanded.hotbar[2].textureX, 53);
    }

    #[test]
    fn spectator_branch_does_not_emit_normal_player_hud() {
        let mut gui = GuiIngame::new();
        let frame = frame(&mut gui, GameType::Spectator);
        assert!(frame.hotbar.is_empty());
        assert!(frame.crosshair.is_empty());
        assert!(frame.playerStats.is_empty());
    }

    #[test]
    fn experience_bar_and_level_match_mcp_geometry() {
        let mut gui = GuiIngame::new();
        let frame = gui.buildFrame(
            320,
            180,
            0,
            false,
            EnumHandSide::Right,
            GameType::Survival,
            20.0,
            0.0,
            20,
            5.0,
            0.5,
            12,
            31,
            0,
            1,
            2_000,
            None,
            "",
            None,
            0,
        );
        assert_eq!(
            frame.experienceBar[0],
            HudTexturedQuad {
                texture: HudTexture::Icons,
                x: 69,
                y: 151,
                width: 182,
                height: 5,
                textureX: 0,
                textureY: 64,
                textureWidth: 182,
                textureHeight: 5,
                alpha: 1.0,
            }
        );
        assert_eq!(frame.experienceBar[1].width, 91);
        assert_eq!(frame.experienceBar[1].textureY, 69);
        assert_eq!(
            frame.experienceLevel,
            Some(HudText {
                text: "12".to_owned(),
                x: 154,
                y: 145,
                color: 8_453_920,
                outline: true,
            })
        );
    }

    #[test]
    fn synchronized_max_health_expands_player_heart_rows() {
        let mut gui = GuiIngame::new();
        let mut quads = Vec::new();
        gui.appendPlayerStats(
            &mut quads,
            320,
            180,
            40.0,
            40.0,
            0.0,
            20,
            5.0,
            0,
            300,
            false,
            false,
            &[],
            true,
            0,
            2_000,
        );
        // 40 max health = 20 heart slots, so a second row is present.
        assert!(quads.iter().any(|quad| quad.textureY == 0 && quad.y < 141));
    }

    #[test]
    fn horse_jump_bar_replaces_experience_bar_geometry() {
        let gui = GuiIngame::new();
        let mut bars = Vec::new();
        gui.appendHorseJumpBar(&mut bars, 320, 180, 0.5);
        assert_eq!(bars[0].x, 69);
        assert_eq!(bars[0].y, 151);
        assert_eq!(bars[0].textureY, 84);
        assert_eq!(bars[0].width, 182);
        assert_eq!(bars[1].textureY, 89);
        assert_eq!(bars[1].width, (0.5_f32 * 183.0) as i32);
    }

    #[test]
    fn living_mount_suppresses_food_and_uses_vanilla_mount_hearts() {
        let mut gui = GuiIngame::new();
        let mut stats = Vec::new();
        gui.appendPlayerStats(
            &mut stats,
            320,
            180,
            20.0,
            20.0,
            0.0,
            20,
            5.0,
            0,
            300,
            false,
            false,
            &[],
            false,
            0,
            2_000,
        );
        assert!(!stats.iter().any(|quad| quad.textureY == 27));

        gui.appendMountHealth(&mut stats, 320, 180, 15.0, 20.0);
        // MCP renderMountHealth: first slot is x=242/y=141, empty UV 52/9
        // followed by a full mount-heart UV 88/9 while health exceeds 1.
        assert!(stats.contains(&icon_quad(242, 141, 52, 9)));
        assert!(stats.contains(&icon_quad(242, 141, 88, 9)));
    }

    #[test]
    fn creative_keeps_hotbar_but_hides_survival_stats() {
        let mut gui = GuiIngame::new();
        let frame = frame(&mut gui, GameType::Creative);
        assert!(!frame.hotbar.is_empty());
        assert!(frame.playerStats.is_empty());
        assert!(frame.experienceBar.is_empty());
        assert!(frame.experienceLevel.is_none());
    }
}
