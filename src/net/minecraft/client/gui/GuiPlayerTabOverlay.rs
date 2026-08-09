use std::collections::HashMap;

use uuid::Uuid;

use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiIngame::{HudSolidRect, HudText, HudTexturedQuad, HudTexture};
use crate::net::minecraft::client::gui::GuiNewChat::split_formatted_text;
use crate::net::minecraft::client::network::NetworkPlayerInfo::NetworkPlayerInfo;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;
use crate::net::minecraft::world::GameType::GameType;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::scoreboard::Scoreboard::Scoreboard;
use crate::net::minecraft::scoreboard::ScorePlayerTeam::ScorePlayerTeam;
use crate::net::minecraft::scoreboard::IScoreCriteria::EnumRenderType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerTabHead {
    pub location: ResourceLocation,
    pub x: i32,
    pub y: i32,
    pub upsideDown: bool,
    pub renderHat: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlayerTabFrame {
    pub rectangles: Vec<HudSolidRect>,
    pub heads: Vec<PlayerTabHead>,
    pub icons: Vec<HudTexturedQuad>,
    pub texts: Vec<HudText>,
}

/// Backend-neutral port of the layout and ordering owned by MCP 1.12.2
/// `GuiPlayerTabOverlay`. Integer score objectives are rendered from the
/// synchronized Scoreboard; downloaded player skins and the animated HEARTS
/// objective branch remain separate dependencies and are never fabricated.
#[derive(Debug, Clone, Default)]
pub struct GuiPlayerTabOverlay {
    isBeingRendered: bool,
    lastTimeOpened: u64,
}

impl GuiPlayerTabOverlay {
    pub fn new() -> Self { Self::default() }

    pub fn updatePlayerList(&mut self, willBeRendered: bool, systemTimeMillis: u64) {
        if willBeRendered && !self.isBeingRendered {
            self.lastTimeOpened = systemTimeMillis;
        }
        self.isBeingRendered = willBeRendered;
    }

    pub fn buildFrameWithFont(
        &mut self,
        width: i32,
        entries: &[NetworkPlayerInfo],
        header: Option<&ITextComponent>,
        footer: Option<&ITextComponent>,
        scoreboard: Option<&Scoreboard>,
        systemTimeMillis: u64,
        fontRenderer: &FontRenderer,
        showHeads: bool,
        playerSkinParts: &HashMap<Uuid, u8>,
    ) -> PlayerTabFrame {
        self.updatePlayerList(true, systemTimeMillis);
        let width = width.max(1);
        let mut list = entries.to_vec();
        list.sort_by(|left, right| {
            let left_normal = left.getGameType() != GameType::Spectator;
            let right_normal = right.getGameType() != GameType::Spectator;
            let left_team = scoreboard
                .and_then(|board| board.getPlayersTeam(left.getGameProfile().getName()))
                .map_or("", ScorePlayerTeam::getRegisteredName);
            let right_team = scoreboard
                .and_then(|board| board.getPlayersTeam(right.getGameProfile().getName()))
                .map_or("", ScorePlayerTeam::getRegisteredName);
            right_normal.cmp(&left_normal)
                .then_with(|| left_team.cmp(right_team))
                .then_with(|| left.getGameProfile().getName().cmp(right.getGameProfile().getName()))
        });
        if list.is_empty() && header.is_none() && footer.is_none() {
            return PlayerTabFrame::default();
        }

        let max_name_width = list.iter().map(|entry| fontRenderer.get_string_width(&player_name(entry, scoreboard))).max().unwrap_or(0);
        // GuiIngame supplies display slot 0 to the tab overlay. Integer
        // objectives use the exact right-aligned score branch. HEARTS keeps
        // its dedicated animated icon branch separate until NetworkPlayerInfo
        // health-history fields are migrated; it is never replaced by fake
        // static hearts or a numeric approximation.
        let tabObjective = scoreboard
            .and_then(|board| board.getObjectiveInDisplaySlot(0))
            .filter(|objective| objective.getRenderType() == EnumRenderType::Integer);
        let scoreWidth = tabObjective.map_or(0, |objective| {
            list.iter()
                .filter(|entry| entry.getGameType() != GameType::Spectator)
                .map(|entry| {
                    let points = scoreboard
                        .map_or(0, |board| board.getScorePoints(entry.getGameProfile().getName(), objective.getName()));
                    fontRenderer.get_string_width(&format!(" {points}"))
                })
                .max()
                .unwrap_or(0)
        });
        list.truncate(80);
        let count = list.len() as i32;
        let mut rows = count;
        let mut columns = 1;
        while rows > 20 {
            columns += 1;
            rows = (count + columns - 1) / columns;
        }

        // MCP reserves nine pixels only when the integrated/encrypted
        // player-head branch is active. The face and hat quads use each
        // NetworkPlayerInfo's downloaded skin rather than a GUI atlas icon.
        let headWidth = if showHeads { 9 } else { 0 };
        let column_width = (columns * (max_name_width + scoreWidth + headWidth + 13))
            .min((width - 50).max(0)) / columns;
        let table_width = column_width * columns + (columns - 1) * 5;
        let table_left = width / 2 - table_width / 2;
        let mut y = 10;
        let header_lines = wrap_component_with_font(header, width - 50, fontRenderer);
        let footer_lines = wrap_component_with_font(footer, width - 50, fontRenderer);
        let widest_aux = header_lines.iter().chain(footer_lines.iter()).map(|line| fontRenderer.get_string_width(line)).max().unwrap_or(0);
        let overall_width = table_width.max(widest_aux);
        let mut frame = PlayerTabFrame::default();

        if !header_lines.is_empty() {
            frame.rectangles.push(HudSolidRect::new(width / 2 - overall_width / 2 - 1, y - 1, overall_width + 2, header_lines.len() as i32 * 9 + 1, 0x8000_0000));
            for line in header_lines {
                let line_width = fontRenderer.get_string_width(&line);
                frame.texts.push(HudText { text: line, x: width / 2 - line_width / 2, y, color: 0x00FF_FFFF, outline: true });
                y += 9;
            }
            y += 1;
        }

        frame.rectangles.push(HudSolidRect::new(width / 2 - overall_width / 2 - 1, y - 1, overall_width + 2, rows * 9 + 1, 0x8000_0000));
        for (index, entry) in list.iter().enumerate() {
            let index = index as i32;
            let column = index / rows.max(1);
            let row = index % rows.max(1);
            let x = table_left + column * (column_width + 5);
            let row_y = y + row * 9;
            frame.rectangles.push(HudSolidRect::new(x, row_y, column_width, 8, 0x20FF_FFFF));
            let mut nameX = x;
            if showHeads {
                let entityParts = entry
                    .getGameProfile()
                    .getId()
                    .and_then(|uuid| playerSkinParts.get(&uuid).copied());
                let upsideDown = entityParts.is_some()
                    && (entityParts.unwrap_or(0) & 0x01) != 0
                    && matches!(entry.getGameProfile().getName(), "Dinnerbone" | "Grumm");
                frame.heads.push(PlayerTabHead {
                    location: entry.getLocationSkin(),
                    x,
                    y: row_y,
                    upsideDown,
                    renderHat: entityParts.is_some_and(|parts| (parts & 0x40) != 0),
                });
                nameX += 9;
            }
            let name = player_name(entry, scoreboard);
            let spectator = entry.getGameType() == GameType::Spectator;
            frame.texts.push(HudText {
                text: if spectator { format!("§o{name}") } else { name },
                x: nameX,
                y: row_y,
                color: if spectator { 0x90FF_FFFF } else { 0xFFFF_FFFF },
                outline: true,
            });
            if !spectator {
                if let (Some(board), Some(objective)) = (scoreboard, tabObjective) {
                    let score = format!("§e{}", board.getScorePoints(entry.getGameProfile().getName(), objective.getName()));
                    let scoreLeft = nameX + max_name_width + 1;
                    let scoreRight = scoreLeft + scoreWidth;
                    if scoreRight - scoreLeft > 5 {
                        frame.texts.push(HudText {
                            text: score.clone(),
                            x: scoreRight - fontRenderer.get_string_width(&score),
                            y: row_y,
                            color: 0xFFFF_FFFF,
                            outline: true,
                        });
                    }
                }
            }
            let ping_index = ping_icon(entry.getResponseTime());
            frame.icons.push(HudTexturedQuad {
                texture: HudTexture::Icons,
                x: x + column_width - 11,
                y: row_y,
                width: 10,
                height: 8,
                textureX: 0,
                textureY: 176 + ping_index * 8,
                textureWidth: 10,
                textureHeight: 8,
                alpha: 1.0,
            });
        }

        if !footer_lines.is_empty() {
            y += rows * 9 + 1;
            frame.rectangles.push(HudSolidRect::new(width / 2 - overall_width / 2 - 1, y - 1, overall_width + 2, footer_lines.len() as i32 * 9 + 1, 0x8000_0000));
            for line in footer_lines {
                let line_width = fontRenderer.get_string_width(&line);
                frame.texts.push(HudText { text: line, x: width / 2 - line_width / 2, y, color: 0x00FF_FFFF, outline: true });
                y += 9;
            }
        }
        frame
    }

    pub fn buildFrame(
        &mut self,
        width: i32,
        entries: &[NetworkPlayerInfo],
        header: Option<&ITextComponent>,
        footer: Option<&ITextComponent>,
        scoreboard: Option<&Scoreboard>,
        systemTimeMillis: u64,
    ) -> PlayerTabFrame {
        let fontRenderer = FontRenderer::test_metric_renderer();
        self.buildFrameWithFont(
            width,
            entries,
            header,
            footer,
            scoreboard,
            systemTimeMillis,
            &fontRenderer,
            false,
            &HashMap::new(),
        )
    }

    pub fn hide(&mut self) { self.isBeingRendered = false; }
    pub const fn lastTimeOpened(&self) -> u64 { self.lastTimeOpened }
}

fn player_name(entry: &NetworkPlayerInfo, scoreboard: Option<&Scoreboard>) -> String {
    entry.getDisplayName()
        .map(|name| name.getFormattedText().to_owned())
        .unwrap_or_else(|| {
            ScorePlayerTeam::formatPlayerName(
                scoreboard.and_then(|board| board.getPlayersTeam(entry.getGameProfile().getName())),
                entry.getGameProfile().getName(),
            )
        })
}

fn visible_width(text: &str) -> i32 {
    let mut width = 0;
    let mut formatting = false;
    for character in text.chars() {
        if formatting { formatting = false; continue; }
        if character == '§' { formatting = true; } else { width += 6; }
    }
    width
}

fn wrap_component_with_font(
    component: Option<&ITextComponent>,
    width: i32,
    fontRenderer: &FontRenderer,
) -> Vec<String> {
    let Some(component) = component else { return Vec::new(); };
    let text = component.getFormattedText();
    if text.is_empty() { return Vec::new(); }
    fontRenderer.list_formatted_string_to_width(text, width.max(1))
}

fn wrap_component(component: Option<&ITextComponent>, width: i32) -> Vec<String> {
    let Some(component) = component else { return Vec::new(); };
    let text = component.getFormattedText();
    if text.is_empty() { return Vec::new(); }
    split_formatted_text(text, width.max(6))
}

const fn ping_icon(response_time: i32) -> i32 {
    if response_time < 0 { 5 }
    else if response_time < 150 { 0 }
    else if response_time < 300 { 1 }
    else if response_time < 600 { 2 }
    else if response_time < 1000 { 3 }
    else { 4 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::com::mojang::authlib::GameProfile::GameProfile;

    fn player(name: &str, game_type: GameType, ping: i32) -> NetworkPlayerInfo {
        NetworkPlayerInfo::new(GameProfile::new(None, name), game_type, ping, None)
    }

    #[test]
    fn caps_to_eighty_and_uses_twenty_rows_per_column() {
        let entries = (0..81).map(|index| player(&format!("P{index:02}"), GameType::Survival, 20)).collect::<Vec<_>>();
        let frame = GuiPlayerTabOverlay::new().buildFrame(500, &entries, None, None, None, 100);
        assert_eq!(frame.texts.len(), 80);
        assert_eq!(frame.icons.len(), 80);
        assert_eq!(frame.rectangles.len(), 81);
    }

    #[test]
    fn spectator_players_sort_after_normal_players() {
        let entries = vec![player("A", GameType::Spectator, 20), player("B", GameType::Survival, 20)];
        let frame = GuiPlayerTabOverlay::new().buildFrame(320, &entries, None, None, None, 100);
        assert_eq!(frame.texts[0].text, "B");
        assert_eq!(frame.texts[1].text, "§oA");
    }

    #[test]
    fn ping_thresholds_match_mcp() {
        assert_eq!(ping_icon(-1), 5);
        assert_eq!(ping_icon(149), 0);
        assert_eq!(ping_icon(150), 1);
        assert_eq!(ping_icon(300), 2);
        assert_eq!(ping_icon(600), 3);
        assert_eq!(ping_icon(1000), 4);
    }
    #[test]
    fn authenticated_head_branch_reserves_space_and_uses_downloaded_skin() {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let entry = NetworkPlayerInfo::new(
            GameProfile::new(Some(id), "Alex"),
            GameType::Survival,
            20,
            None,
        );
        let parts = HashMap::from([(id, 0x40)]);
        let font = FontRenderer::test_metric_renderer();
        let frame = GuiPlayerTabOverlay::new().buildFrameWithFont(
            320,
            &[entry],
            None,
            None,
            None,
            100,
            &font,
            true,
            &parts,
        );
        assert_eq!(frame.heads.len(), 1);
        assert!(frame.heads[0].renderHat);
        assert_eq!(frame.texts[0].x, frame.heads[0].x + 9);
    }

}
