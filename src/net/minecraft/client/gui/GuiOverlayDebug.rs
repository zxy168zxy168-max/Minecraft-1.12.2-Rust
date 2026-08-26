use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::Block::Block;
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiIngame::{HudSolidRect, HudText};
use crate::net::minecraft::util::math::BlockPos::BlockPos;

#[derive(Debug, Clone)]
pub struct DebugOverlayData {
    pub version: String,
    pub versionType: String,
    pub debugFps: i32,
    pub reducedDebugInfo: bool,
    pub showDebugProfilerChart: bool,
    pub showLagometer: bool,
    pub playerPosition: [f64; 3],
    pub rotationYaw: f32,
    pub rotationPitch: f32,
    pub renderDistanceChunks: i32,
    pub loadedRenderChunks: usize,
    pub queuedRenderChunks: usize,
    pub remotePlayerCount: usize,
    pub nonPlayerEntityCount: usize,
    pub particleCount: usize,
    pub dimension: i32,
    pub biomeName: String,
    pub skyLight: u8,
    pub blockLight: u8,
    pub worldTime: i64,
    pub targetBlock: Option<(BlockPos, IBlockState)>,
    pub outputWidth: u32,
    pub outputHeight: u32,
    pub vulkanDevice: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DebugOverlayFrame {
    pub rectangles: Vec<HudSolidRect>,
    pub texts: Vec<HudText>,
}

/// Backend-neutral port of MCP 1.12.2 `GuiOverlayDebug`.
#[derive(Debug, Clone, Default)]
pub struct GuiOverlayDebug;

impl GuiOverlayDebug {
    pub fn new() -> Self {
        Self
    }

    pub fn buildFrame(
        &self,
        guiWidth: i32,
        data: &DebugOverlayData,
        fontRenderer: &FontRenderer,
    ) -> DebugOverlayFrame {
        let left = self.leftLines(data);
        let right = self.rightLines(data);
        let mut frame = DebugOverlayFrame::default();
        append_lines(&mut frame, &left, false, guiWidth, fontRenderer);
        append_lines(&mut frame, &right, true, guiWidth, fontRenderer);
        frame
    }

    fn leftLines(&self, data: &DebugOverlayData) -> Vec<String> {
        let position = BlockPos::new(
            data.playerPosition[0].floor() as i32,
            data.playerPosition[1].floor() as i32,
            data.playerPosition[2].floor() as i32,
        );
        let mut lines = vec![
            format!(
                "Minecraft 1.12.2 ({}/rust-vulkan{})",
                data.version,
                if data.versionType.eq_ignore_ascii_case("release") {
                    String::new()
                } else {
                    format!("/{}", data.versionType)
                },
            ),
            format!("{} fps", data.debugFps.max(0)),
            format!(
                "C: {} / {} (queued {})",
                data.loadedRenderChunks,
                render_chunk_budget(data.renderDistanceChunks),
                data.queuedRenderChunks,
            ),
            format!(
                "E: {}+{}, P: {}",
                data.remotePlayerCount, data.nonPlayerEntityCount, data.particleCount,
            ),
            dimension_name(data.dimension).to_owned(),
            String::new(),
        ];

        if data.reducedDebugInfo {
            lines.push(format!(
                "Chunk-relative: {} {} {}",
                position.x & 15,
                position.y & 15,
                position.z & 15,
            ));
        } else {
            lines.extend([
                format!(
                    "XYZ: {:.3} / {:.5} / {:.3}",
                    data.playerPosition[0], data.playerPosition[1], data.playerPosition[2],
                ),
                format!("Block: {} {} {}", position.x, position.y, position.z),
                format!(
                    "Chunk: {} {} {} in {} {} {}",
                    position.x & 15,
                    position.y & 15,
                    position.z & 15,
                    position.x >> 4,
                    position.y >> 4,
                    position.z >> 4,
                ),
                format!(
                    "Facing: {} ({}) ({:.1} / {:.1})",
                    horizontal_facing(data.rotationYaw),
                    facing_description(data.rotationYaw),
                    wrap_degrees(data.rotationYaw),
                    wrap_degrees(data.rotationPitch),
                ),
                format!("Biome: {}", data.biomeName),
                format!(
                    "Light: {} ({} sky, {} block)",
                    data.skyLight.max(data.blockLight),
                    data.skyLight,
                    data.blockLight,
                ),
                format!("Day: {}", data.worldTime.div_euclid(24_000)),
            ]);
            if let Some((target, _)) = data.targetBlock {
                lines.push(format!(
                    "Looking at: {} {} {}",
                    target.x, target.y, target.z
                ));
            }
        }
        lines.push(String::new());
        lines.push(format!(
            "Debug: Pie [shift]: {} FPS [alt]: {}",
            visible(data.showDebugProfilerChart),
            visible(data.showLagometer),
        ));
        lines.push("For help: press F3 + Q".to_owned());
        lines
    }

    fn rightLines(&self, data: &DebugOverlayData) -> Vec<String> {
        let mut lines = vec![
            format!("Rust: {} {}bit", rust_runtime_label(), usize::BITS),
            format!("OS: {} {}", std::env::consts::OS, std::env::consts::ARCH),
            String::new(),
            format!("CPU: {}", cpu_name()),
            String::new(),
            format!(
                "Display: {}x{} (Vulkan)",
                data.outputWidth, data.outputHeight
            ),
            data.vulkanDevice.clone(),
        ];
        if !data.reducedDebugInfo {
            if let Some((_, state)) = data.targetBlock {
                let block = Block::getBlockById(state.getBlockId());
                lines.push(String::new());
                lines.push(block.getRegistryName().to_string());
                lines.push(format!("metadata: {}", state.getMetadata()));
            }
        }
        lines
    }
}

fn append_lines(
    frame: &mut DebugOverlayFrame,
    lines: &[String],
    rightAligned: bool,
    guiWidth: i32,
    fontRenderer: &FontRenderer,
) {
    let fontHeight = 9;
    for (index, line) in lines.iter().enumerate() {
        if line.is_empty() {
            continue;
        }
        let width = fontRenderer.get_string_width(line);
        let x = if rightAligned {
            guiWidth - 2 - width
        } else {
            2
        };
        let y = 2 + fontHeight * index as i32;
        frame.rectangles.push(HudSolidRect::new(
            x - 1,
            y - 1,
            width + 2,
            fontHeight,
            0x9050_5050,
        ));
        frame.texts.push(HudText {
            text: line.clone(),
            x,
            y,
            color: 0xFFE0_E0E0,
            outline: false,
        });
    }
}

fn render_chunk_budget(distance: i32) -> i32 {
    let diameter = distance.max(2) * 2 + 1;
    diameter * diameter * 16
}

fn visible(value: bool) -> &'static str {
    if value {
        "visible"
    } else {
        "hidden"
    }
}
fn dimension_name(dimension: i32) -> &'static str {
    match dimension {
        -1 => "The Nether",
        1 => "The End",
        _ => "Overworld",
    }
}

fn horizontal_facing(yaw: f32) -> &'static str {
    match ((yaw / 90.0).floor() as i32).rem_euclid(4) {
        0 => "south",
        1 => "west",
        2 => "north",
        _ => "east",
    }
}

fn facing_description(yaw: f32) -> &'static str {
    match horizontal_facing(yaw) {
        "north" => "Towards negative Z",
        "south" => "Towards positive Z",
        "west" => "Towards negative X",
        _ => "Towards positive X",
    }
}

fn wrap_degrees(value: f32) -> f32 {
    (value + 180.0).rem_euclid(360.0) - 180.0
}
fn rust_runtime_label() -> String {
    option_env!("RUSTC_VERSION").unwrap_or("native").to_owned()
}
fn cpu_name() -> String {
    std::env::var("PROCESSOR_IDENTIFIER")
        .or_else(|_| std::env::var("HOSTTYPE"))
        .unwrap_or_else(|_| std::env::consts::ARCH.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facing_matches_minecraft_horizontal_index() {
        assert_eq!(horizontal_facing(0.0), "south");
        assert_eq!(horizontal_facing(90.0), "west");
        assert_eq!(horizontal_facing(180.0), "north");
        assert_eq!(horizontal_facing(-90.0), "east");
    }
}
