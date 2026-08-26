use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiIngame::{HudSolidRect, HudText};

#[derive(Debug, Clone, PartialEq)]
struct Subtitle {
    subtitle: String,
    startTime: u64,
    location: [f32; 3],
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SubtitleFrame {
    pub rectangles: Vec<HudSolidRect>,
    pub texts: Vec<HudText>,
}

/// Backend-neutral port of MCP 1.12.2 `GuiSubtitleOverlay`.
#[derive(Debug, Clone, Default)]
pub struct GuiSubtitleOverlay {
    subtitles: Vec<Subtitle>,
    enabled: bool,
}

impl GuiSubtitleOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn soundPlay(
        &mut self,
        subtitle: impl Into<String>,
        location: [f32; 3],
        systemTimeMillis: u64,
    ) {
        let subtitle = subtitle.into();
        if let Some(existing) = self
            .subtitles
            .iter_mut()
            .find(|entry| entry.subtitle == subtitle)
        {
            existing.location = location;
            existing.startTime = systemTimeMillis;
            return;
        }
        self.subtitles.push(Subtitle {
            subtitle,
            startTime: systemTimeMillis,
            location,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn buildFrame(
        &mut self,
        showSubtitles: bool,
        guiWidth: i32,
        guiHeight: i32,
        listenerPosition: [f32; 3],
        listenerYaw: f32,
        listenerPitch: f32,
        systemTimeMillis: u64,
        fontRenderer: &FontRenderer,
    ) -> SubtitleFrame {
        self.enabled = showSubtitles;
        if !self.enabled {
            return SubtitleFrame::default();
        }

        self.subtitles
            .retain(|subtitle| subtitle.startTime.saturating_add(3_000) > systemTimeMillis);
        if self.subtitles.is_empty() {
            return SubtitleFrame::default();
        }

        let pitch = -listenerPitch.to_radians();
        let yaw = -listenerYaw.to_radians();
        let forward = rotate_yaw(rotate_pitch([0.0, 0.0, -1.0], pitch), yaw);
        let up = rotate_yaw(rotate_pitch([0.0, 1.0, 0.0], pitch), yaw);
        let side = cross(forward, up);

        let widest = self
            .subtitles
            .iter()
            .map(|subtitle| fontRenderer.get_string_width(&subtitle.subtitle))
            .max()
            .unwrap_or(0);
        let width = widest
            + fontRenderer.get_string_width("<")
            + fontRenderer.get_string_width(" ")
            + fontRenderer.get_string_width(">")
            + fontRenderer.get_string_width(" ");
        let halfWidth = width / 2;
        let fontHeight = 9;
        let halfFontHeight = fontHeight / 2;
        let mut frame = SubtitleFrame::default();

        for (index, subtitle) in self.subtitles.iter().enumerate() {
            let direction = normalize([
                subtitle.location[0] - listenerPosition[0],
                subtitle.location[1] - listenerPosition[1],
                subtitle.location[2] - listenerPosition[2],
            ]);
            let lateral = -dot(side, direction);
            let ahead = -dot(forward, direction);
            let visibleAhead = ahead > 0.5;
            let elapsed = systemTimeMillis.saturating_sub(subtitle.startTime) as f32;
            let fade = (elapsed / 3_000.0).clamp(0.0, 1.0);
            let gray = (255.0 + (75.0 - 255.0) * fade).floor() as u32;
            let color = 0xFF00_0000 | gray << 16 | gray << 8 | gray;
            let centerX = guiWidth - halfWidth - 2;
            let centerY = guiHeight - 30 - index as i32 * (fontHeight + 1);
            frame.rectangles.push(HudSolidRect::new(
                centerX - halfWidth - 1,
                centerY - halfFontHeight - 1,
                halfWidth * 2 + 2,
                halfFontHeight * 2 + 2,
                0xCC00_0000,
            ));
            if !visibleAhead {
                if lateral > 0.0 {
                    frame.texts.push(HudText {
                        text: ">".to_owned(),
                        x: centerX + halfWidth - fontRenderer.get_string_width(">"),
                        y: centerY - halfFontHeight,
                        color,
                        outline: false,
                    });
                } else if lateral < 0.0 {
                    frame.texts.push(HudText {
                        text: "<".to_owned(),
                        x: centerX - halfWidth,
                        y: centerY - halfFontHeight,
                        color,
                        outline: false,
                    });
                }
            }
            frame.texts.push(HudText {
                text: subtitle.subtitle.clone(),
                x: centerX - fontRenderer.get_string_width(&subtitle.subtitle) / 2,
                y: centerY - halfFontHeight,
                color,
                outline: false,
            });
        }
        frame
    }
}

fn rotate_pitch(value: [f32; 3], pitch: f32) -> [f32; 3] {
    let cosine = pitch.cos();
    let sine = pitch.sin();
    [
        value[0],
        value[1] * cosine + value[2] * sine,
        value[2] * cosine - value[1] * sine,
    ]
}

fn rotate_yaw(value: [f32; 3], yaw: f32) -> [f32; 3] {
    let cosine = yaw.cos();
    let sine = yaw.sin();
    [
        value[0] * cosine + value[2] * sine,
        value[1],
        value[2] * cosine - value[0] * sine,
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(value: [f32; 3]) -> [f32; 3] {
    let length = dot(value, value).sqrt();
    if length <= f32::EPSILON {
        [0.0; 3]
    } else {
        [value[0] / length, value[1] / length, value[2] / length]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_subtitle_refreshes_instead_of_duplicating() {
        let mut overlay = GuiSubtitleOverlay::new();
        overlay.soundPlay("Footsteps", [1.0, 0.0, 0.0], 100);
        overlay.soundPlay("Footsteps", [2.0, 0.0, 0.0], 200);
        assert_eq!(overlay.subtitles.len(), 1);
        assert_eq!(overlay.subtitles[0].location, [2.0, 0.0, 0.0]);
        assert_eq!(overlay.subtitles[0].startTime, 200);
    }
}
