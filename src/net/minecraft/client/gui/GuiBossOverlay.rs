use crate::net::minecraft::client::gui::BossInfoClient::BossInfoClient;
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiIngame::{HudText, HudTexture, HudTexturedQuad};
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::network::play::server::SPacketUpdateBossInfo::{
    Operation, SPacketUpdateBossInfo,
};
use crate::net::minecraft::world::BossInfo::Overlay;
use uuid::Uuid;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BossOverlayFrame {
    pub bars: Vec<HudTexturedQuad>,
    pub texts: Vec<HudText>,
}

/// MCP 1.12.2 `GuiBossOverlay`, retaining insertion order without using a
/// backend-specific container.
#[derive(Debug, Clone, Default)]
pub struct GuiBossOverlay {
    mapBossInfos: Vec<(Uuid, BossInfoClient)>,
}

impl GuiBossOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(
        &mut self,
        packetIn: &SPacketUpdateBossInfo,
        systemTimeMillis: u64,
        locale: &Locale,
    ) {
        let id = packetIn.getUniqueId();
        match packetIn.getOperation() {
            Operation::Add => {
                self.mapBossInfos.retain(|(existing, _)| *existing != id);
                if let Some(info) = BossInfoClient::new(packetIn, systemTimeMillis, locale) {
                    self.mapBossInfos.push((id, info));
                }
            }
            Operation::Remove => self.mapBossInfos.retain(|(existing, _)| *existing != id),
            _ => {
                if let Some((_, info)) = self
                    .mapBossInfos
                    .iter_mut()
                    .find(|(existing, _)| *existing == id)
                {
                    info.updateFromPacket(packetIn, systemTimeMillis, locale);
                }
            }
        }
    }

    pub fn buildFrame(
        &self,
        guiWidth: i32,
        guiHeight: i32,
        systemTimeMillis: u64,
        fontRenderer: &FontRenderer,
    ) -> BossOverlayFrame {
        let mut frame = BossOverlayFrame::default();
        let mut y = 12;
        for (_, client) in &self.mapBossInfos {
            let info = client.info();
            let x = guiWidth / 2 - 91;
            append_bar(
                &mut frame.bars,
                x,
                y,
                info.getColor().ordinal(),
                info.getOverlay(),
                182,
            );
            let filled = (client.getPercent(systemTimeMillis) * 183.0) as i32;
            if filled > 0 {
                append_bar(
                    &mut frame.bars,
                    x,
                    y,
                    info.getColor().ordinal(),
                    info.getOverlay(),
                    filled.min(183),
                );
                // `append_bar` distinguishes fill/background through this
                // marker by replacing the just-added Y coordinate below.
                let count = if info.getOverlay() == Overlay::Progress {
                    1
                } else {
                    2
                };
                for quad in frame.bars.iter_mut().rev().take(count) {
                    quad.textureY += 5;
                }
            }
            let name = info.getName().getFormattedText();
            frame.texts.push(HudText {
                x: guiWidth / 2 - fontRenderer.get_string_width(name) / 2,
                y: y - 9,
                text: name.to_owned(),
                color: 0x00FF_FFFF,
                outline: true,
            });
            y += 10 + 9;
            if y >= guiHeight / 3 {
                break;
            }
        }
        frame
    }

    pub fn clearBossInfos(&mut self) {
        self.mapBossInfos.clear();
    }
    pub fn shouldPlayEndBossMusic(&self) -> bool {
        self.mapBossInfos
            .iter()
            .any(|(_, info)| info.info().shouldPlayEndBossMusic())
    }
    pub fn shouldDarkenSky(&self) -> bool {
        self.mapBossInfos
            .iter()
            .any(|(_, info)| info.info().shouldDarkenSky())
    }
    pub fn shouldCreateFog(&self) -> bool {
        self.mapBossInfos
            .iter()
            .any(|(_, info)| info.info().shouldCreateFog())
    }
}

fn append_bar(
    bars: &mut Vec<HudTexturedQuad>,
    x: i32,
    y: i32,
    colorOrdinal: i32,
    overlay: Overlay,
    width: i32,
) {
    bars.push(HudTexturedQuad {
        texture: HudTexture::BossBars,
        x,
        y,
        width,
        height: 5,
        textureX: 0,
        textureY: colorOrdinal * 10,
        textureWidth: width,
        textureHeight: 5,
        alpha: 1.0,
    });
    if overlay != Overlay::Progress {
        bars.push(HudTexturedQuad {
            texture: HudTexture::BossBars,
            x,
            y,
            width,
            height: 5,
            textureX: 0,
            textureY: 80 + (overlay.ordinal() - 1) * 10,
            textureWidth: width,
            textureHeight: 5,
            alpha: 1.0,
        });
    }
}
