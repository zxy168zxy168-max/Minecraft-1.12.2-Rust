use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuiVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub u: f32,
    pub v: f32,
    /// Packed ARGB, matching Minecraft's integer color convention.
    pub color: u32,
}

impl GuiVertex {
    pub const fn new(x: f32, y: f32, z: f32, u: f32, v: f32, color: u32) -> Self {
        Self {
            x,
            y,
            z,
            u,
            v,
            color,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    pub m00: f32,
    pub m01: f32,
    pub m02: f32,
    pub m10: f32,
    pub m11: f32,
    pub m12: f32,
}

impl Transform2D {
    pub const IDENTITY: Self = Self {
        m00: 1.0,
        m01: 0.0,
        m02: 0.0,
        m10: 0.0,
        m11: 1.0,
        m12: 0.0,
    };

    pub fn translated(self, x: f32, y: f32) -> Self {
        self.then(Self {
            m02: x,
            m12: y,
            ..Self::IDENTITY
        })
    }

    pub fn scaled(self, x: f32, y: f32) -> Self {
        self.then(Self {
            m00: x,
            m11: y,
            ..Self::IDENTITY
        })
    }

    pub fn rotated_degrees(self, angle: f32) -> Self {
        let radians = angle.to_radians();
        let (sin, cos) = radians.sin_cos();
        self.then(Self {
            m00: cos,
            m01: -sin,
            m02: 0.0,
            m10: sin,
            m11: cos,
            m12: 0.0,
        })
    }

    /// Matrix composition equivalent to applying `next` after the current
    /// OpenGL model-view transform.
    pub fn then(self, next: Self) -> Self {
        Self {
            m00: self.m00 * next.m00 + self.m01 * next.m10,
            m01: self.m00 * next.m01 + self.m01 * next.m11,
            m02: self.m00 * next.m02 + self.m01 * next.m12 + self.m02,
            m10: self.m10 * next.m00 + self.m11 * next.m10,
            m11: self.m10 * next.m01 + self.m11 * next.m11,
            m12: self.m10 * next.m02 + self.m11 * next.m12 + self.m12,
        }
    }

    pub fn transform_point(self, x: f32, y: f32) -> (f32, f32) {
        (
            self.m00 * x + self.m01 * y + self.m02,
            self.m10 * x + self.m11 * y + self.m12,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PanoramaCommand {
    pub textures: [ResourceLocation; 6],
    pub panorama_timer: f32,
    pub first_blur_passes: i32,
    pub second_blur_passes: i32,
    pub final_blur_pairs: i32,
    pub screen_width: i32,
    pub screen_height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiTopology {
    Quads,
    TriangleStrip,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GuiDrawCommand {
    Quad {
        texture: Option<ResourceLocation>,
        topology: GuiTopology,
        vertices: [GuiVertex; 4],
    },
    Panorama(PanoramaCommand),
}

#[derive(Debug, Clone)]
pub struct GuiDrawList {
    commands: Vec<GuiDrawCommand>,
    transform_stack: Vec<Transform2D>,
    z_level: f32,
}

impl Default for GuiDrawList {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            transform_stack: vec![Transform2D::IDENTITY],
            z_level: 0.0,
        }
    }
}

impl GuiDrawList {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn commands(&self) -> &[GuiDrawCommand] {
        &self.commands
    }
    pub fn into_commands(self) -> Vec<GuiDrawCommand> {
        self.commands
    }
    pub fn clear(&mut self) {
        self.commands.clear();
    }
    pub fn set_z_level(&mut self, z_level: f32) {
        self.z_level = z_level;
    }
    pub fn z_level(&self) -> f32 {
        self.z_level
    }

    pub fn push_matrix(&mut self) {
        let current = self.current_transform();
        self.transform_stack.push(current);
    }

    pub fn pop_matrix(&mut self) {
        assert!(self.transform_stack.len() > 1, "GUI matrix stack underflow");
        self.transform_stack.pop();
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        let current = self.current_transform().translated(x, y);
        *self.transform_stack.last_mut().expect("matrix stack") = current;
    }

    pub fn rotate_degrees(&mut self, angle: f32) {
        let current = self.current_transform().rotated_degrees(angle);
        *self.transform_stack.last_mut().expect("matrix stack") = current;
    }

    pub fn scale(&mut self, x: f32, y: f32) {
        let current = self.current_transform().scaled(x, y);
        *self.transform_stack.last_mut().expect("matrix stack") = current;
    }

    pub fn panorama(&mut self, command: PanoramaCommand) {
        self.commands.push(GuiDrawCommand::Panorama(command));
    }

    /// Direct port of `Gui.drawRect`, including coordinate swapping.
    pub fn draw_rect(
        &mut self,
        mut left: i32,
        mut top: i32,
        mut right: i32,
        mut bottom: i32,
        color: i32,
    ) {
        if left < right {
            std::mem::swap(&mut left, &mut right);
        }
        if top < bottom {
            std::mem::swap(&mut top, &mut bottom);
        }
        self.push_quad(
            None,
            GuiTopology::Quads,
            [
                (left as f32, bottom as f32, 0.0, 0.0, color as u32),
                (right as f32, bottom as f32, 0.0, 0.0, color as u32),
                (right as f32, top as f32, 0.0, 0.0, color as u32),
                (left as f32, top as f32, 0.0, 0.0, color as u32),
            ],
        );
    }

    pub fn draw_horizontal_line(&mut self, mut start_x: i32, mut end_x: i32, y: i32, color: i32) {
        if end_x < start_x {
            std::mem::swap(&mut start_x, &mut end_x);
        }
        self.draw_rect(start_x, y, end_x + 1, y + 1, color);
    }

    pub fn draw_vertical_line(&mut self, x: i32, mut start_y: i32, mut end_y: i32, color: i32) {
        if end_y < start_y {
            std::mem::swap(&mut start_y, &mut end_y);
        }
        self.draw_rect(x, start_y + 1, x + 1, end_y, color);
    }

    /// Direct port of `Gui.drawGradientRect` vertex and color order.
    pub fn draw_gradient_rect(
        &mut self,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        start_color: i32,
        end_color: i32,
    ) {
        self.push_quad(
            None,
            GuiTopology::Quads,
            [
                (right as f32, top as f32, 0.0, 0.0, start_color as u32),
                (left as f32, top as f32, 0.0, 0.0, start_color as u32),
                (left as f32, bottom as f32, 0.0, 0.0, end_color as u32),
                (right as f32, bottom as f32, 0.0, 0.0, end_color as u32),
            ],
        );
    }

    /// Direct port of the 256x256 `Gui.drawTexturedModalRect` overload.
    pub fn draw_textured_modal_rect(
        &mut self,
        texture: ResourceLocation,
        x: i32,
        y: i32,
        texture_x: i32,
        texture_y: i32,
        width: i32,
        height: i32,
    ) {
        self.draw_modal_rect_with_custom_sized_texture(
            texture,
            x as f32,
            y as f32,
            texture_x as f32,
            texture_y as f32,
            width as f32,
            height as f32,
            256.0,
            256.0,
        );
    }

    /// Direct port of `Gui.drawModalRectWithCustomSizedTexture`.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_modal_rect_with_custom_sized_texture(
        &mut self,
        texture: ResourceLocation,
        x: f32,
        y: f32,
        u: f32,
        v: f32,
        width: f32,
        height: f32,
        texture_width: f32,
        texture_height: f32,
    ) {
        let inv_width = 1.0 / texture_width;
        let inv_height = 1.0 / texture_height;
        self.push_quad(
            Some(texture),
            GuiTopology::Quads,
            [
                (
                    x,
                    y + height,
                    u * inv_width,
                    (v + height) * inv_height,
                    0xFFFF_FFFF,
                ),
                (
                    x + width,
                    y + height,
                    (u + width) * inv_width,
                    (v + height) * inv_height,
                    0xFFFF_FFFF,
                ),
                (
                    x + width,
                    y,
                    (u + width) * inv_width,
                    v * inv_height,
                    0xFFFF_FFFF,
                ),
                (x, y, u * inv_width, v * inv_height, 0xFFFF_FFFF),
            ],
        );
    }

    pub fn push_textured_quad(
        &mut self,
        texture: ResourceLocation,
        vertices: [(f32, f32, f32, f32, u32); 4],
    ) {
        self.push_quad(Some(texture), GuiTopology::Quads, vertices);
    }

    pub fn push_solid_quad(&mut self, vertices: [(f32, f32, u32); 4]) {
        self.push_quad(
            None,
            GuiTopology::Quads,
            vertices.map(|(x, y, color)| (x, y, 0.0, 0.0, color)),
        );
    }

    fn current_transform(&self) -> Transform2D {
        *self
            .transform_stack
            .last()
            .expect("GUI matrix stack is never empty")
    }

    pub fn push_triangle_strip(
        &mut self,
        texture: ResourceLocation,
        vertices: [(f32, f32, f32, f32, u32); 4],
    ) {
        self.push_quad(Some(texture), GuiTopology::TriangleStrip, vertices);
    }

    fn push_quad(
        &mut self,
        texture: Option<ResourceLocation>,
        topology: GuiTopology,
        vertices: [(f32, f32, f32, f32, u32); 4],
    ) {
        let transform = self.current_transform();
        let vertices = vertices.map(|(x, y, u, v, color)| {
            let (x, y) = transform.transform_point(x, y);
            GuiVertex::new(x, y, self.z_level, u, v, color)
        });
        self.commands.push(GuiDrawCommand::Quad {
            texture,
            topology,
            vertices,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_rect_preserves_vanilla_reverse_vertex_order() {
        let mut list = GuiDrawList::new();
        list.draw_rect(1, 2, 5, 7, 0xAABB_CCDDu32 as i32);
        let GuiDrawCommand::Quad { vertices, .. } = &list.commands()[0] else {
            panic!("quad expected")
        };
        assert_eq!((vertices[0].x, vertices[0].y), (5.0, 2.0));
        assert_eq!((vertices[2].x, vertices[2].y), (1.0, 7.0));
        assert_eq!(vertices[0].color, 0xAABB_CCDD);
    }

    #[test]
    fn textured_rect_uses_256_uv_scale() {
        let mut list = GuiDrawList::new();
        list.draw_textured_modal_rect(
            ResourceLocation::new("minecraft", "textures/gui/widgets.png"),
            10,
            20,
            0,
            66,
            100,
            20,
        );
        let GuiDrawCommand::Quad { vertices, .. } = &list.commands()[0] else {
            panic!("quad expected")
        };
        assert_eq!(vertices[0].v, 86.0 / 256.0);
        assert_eq!(vertices[1].u, 100.0 / 256.0);
    }

    #[test]
    fn matrix_stack_applies_splash_transform_order() {
        let mut list = GuiDrawList::new();
        list.translate(100.0, 70.0);
        list.rotate_degrees(-20.0);
        list.scale(2.0, 2.0);
        list.push_solid_quad([(0.0, 0.0, 0), (1.0, 0.0, 0), (1.0, 1.0, 0), (0.0, 1.0, 0)]);
        let GuiDrawCommand::Quad { vertices, .. } = &list.commands()[0] else {
            panic!("quad expected")
        };
        assert!((vertices[0].x - 100.0).abs() < 0.0001);
        assert!((vertices[0].y - 70.0).abs() < 0.0001);
    }
}
