use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::vulkan::GuiDrawList::{GuiDrawCommand, GuiDrawList, GuiTopology, GuiVertex};

use crate::vulkan::PanoramaRenderer::PanoramaPassPlan;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct VulkanGuiVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color_rgba: [f32; 4],
}

impl From<GuiVertex> for VulkanGuiVertex {
    fn from(vertex: GuiVertex) -> Self {
        Self {
            position: [vertex.x, vertex.y, vertex.z],
            uv: [vertex.u, vertex.v],
            color_rgba: argb_to_rgba(vertex.color),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiBatch {
    pub texture: Option<ResourceLocation>,
    pub topology: GuiTopology,
    pub vertices: Vec<VulkanGuiVertex>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompiledGuiStep {
    Draw(GuiBatch),
    Panorama(PanoramaPassPlan),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompiledGuiFrame {
    pub steps: Vec<CompiledGuiStep>,
}

impl CompiledGuiFrame {
    /// Compiles GUI commands while preserving their source order. Only
    /// immediately adjacent quads with identical texture and topology are
    /// merged; batching across a panorama command or texture transition would
    /// alter Minecraft's draw order and is prohibited.
    pub fn compile(draw_list: &GuiDrawList) -> Self {
        let mut frame = Self::default();
        for command in draw_list.commands() {
            match command {
                GuiDrawCommand::Panorama(command) => {
                    frame
                        .steps
                        .push(CompiledGuiStep::Panorama(PanoramaPassPlan::from_command(
                            command,
                        )));
                }
                GuiDrawCommand::Quad {
                    texture,
                    topology,
                    vertices,
                } => {
                    if let Some(CompiledGuiStep::Draw(batch)) = frame.steps.last_mut() {
                        if batch.texture.as_ref() == texture.as_ref() && batch.topology == *topology
                        {
                            append_quad(batch, *vertices);
                            continue;
                        }
                    }
                    let mut batch = GuiBatch {
                        texture: texture.clone(),
                        topology: *topology,
                        vertices: Vec::with_capacity(4),
                        indices: Vec::with_capacity(6),
                    };
                    append_quad(&mut batch, *vertices);
                    frame.steps.push(CompiledGuiStep::Draw(batch));
                }
            }
        }
        frame
    }
}

fn append_quad(batch: &mut GuiBatch, vertices: [GuiVertex; 4]) {
    let base = batch.vertices.len() as u32;
    batch.vertices.extend(vertices.map(VulkanGuiVertex::from));
    match batch.topology {
        GuiTopology::Quads => {
            // Equivalent decomposition of GL_QUADS vertex order.
            batch
                .indices
                .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        GuiTopology::TriangleStrip => {
            // GL_TRIANGLE_STRIP: (0,1,2), then winding-corrected (2,1,3).
            batch.indices.extend_from_slice(&[
                base,
                base + 1,
                base + 2,
                base + 2,
                base + 1,
                base + 3,
            ]);
        }
    }
}

pub fn argb_to_rgba(color: u32) -> [f32; 4] {
    const INV_255: f32 = 1.0 / 255.0;
    [
        ((color >> 16) & 0xFF) as f32 * INV_255,
        ((color >> 8) & 0xFF) as f32 * INV_255,
        (color & 0xFF) as f32 * INV_255,
        ((color >> 24) & 0xFF) as f32 * INV_255,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vulkan::GuiDrawList::GuiDrawList;

    #[test]
    fn argb_conversion_preserves_channel_order() {
        assert_eq!(
            argb_to_rgba(0x8040_20FF),
            [64.0 / 255.0, 32.0 / 255.0, 1.0, 128.0 / 255.0]
        );
    }

    #[test]
    fn only_adjacent_compatible_quads_are_batched() {
        let texture = ResourceLocation::parse("textures/gui/widgets.png");
        let mut list = GuiDrawList::new();
        list.draw_textured_modal_rect(texture.clone(), 0, 0, 0, 0, 10, 10);
        list.draw_textured_modal_rect(texture, 10, 0, 0, 0, 10, 10);
        list.draw_rect(0, 20, 10, 30, -1);
        let frame = CompiledGuiFrame::compile(&list);
        assert_eq!(frame.steps.len(), 2);
        let CompiledGuiStep::Draw(first) = &frame.steps[0] else {
            panic!("draw")
        };
        assert_eq!(first.vertices.len(), 8);
        assert_eq!(first.indices.len(), 12);
    }

    #[test]
    fn triangle_strip_uses_alternating_winding() {
        let texture = ResourceLocation::parse("textures/font/ascii.png");
        let mut list = GuiDrawList::new();
        list.push_triangle_strip(
            texture,
            [
                (0.0, 0.0, 0.0, 0.0, 0xFFFF_FFFF),
                (0.0, 1.0, 0.0, 1.0, 0xFFFF_FFFF),
                (1.0, 0.0, 1.0, 0.0, 0xFFFF_FFFF),
                (1.0, 1.0, 1.0, 1.0, 0xFFFF_FFFF),
            ],
        );
        let frame = CompiledGuiFrame::compile(&list);
        let CompiledGuiStep::Draw(batch) = &frame.steps[0] else {
            panic!("draw")
        };
        assert_eq!(batch.indices, [0, 1, 2, 2, 1, 3]);
    }
}
