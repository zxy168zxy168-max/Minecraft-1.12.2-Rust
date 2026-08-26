use crate::net::minecraft::client::model::ModelBoxGeometry::{
    model_box_geometry, ModelBoxRotation, MODEL_BOX_FACE_INDICES,
};

/// CPU geometry port of MCP 1.12.2 `ModelBook`.
///
/// Coordinates are emitted in ModelRenderer units scaled by 1/16 after each
/// part's rotation point and Y rotation have been applied. The caller owns the
/// enclosing GuiEnchantment or TileEntityEnchantmentTable transform.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BookVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BookMesh {
    pub vertices: Vec<BookVertex>,
    pub indices: Vec<u32>,
}

pub struct ModelBook;

impl ModelBook {
    pub const TEXTURE_WIDTH: f32 = 64.0;
    pub const TEXTURE_HEIGHT: f32 = 32.0;

    /// Parameters are the exact six values passed by `GuiEnchantment` and
    /// `TileEntityEnchantmentTableRenderer` into `ModelBook#render`:
    /// `ticks`, right-page factor, left-page factor and open amount.
    pub fn buildMesh(ticks: f32, pageFlipRight: f32, pageFlipLeft: f32, open: f32) -> BookMesh {
        let mut mesh = BookMesh {
            vertices: Vec::with_capacity(7 * 24),
            indices: Vec::with_capacity(7 * 36),
        };
        let angle = ((ticks * 0.02).sin() * 0.1 + 1.25) * open;
        let page_x = angle.sin();

        add_box(
            &mut mesh,
            [0, 0],
            [-6.0, -5.0, 0.0],
            [6, 10, 0],
            [0.0, 0.0, -1.0],
            std::f32::consts::PI + angle,
        );
        add_box(
            &mut mesh,
            [16, 0],
            [0.0, -5.0, 0.0],
            [6, 10, 0],
            [0.0, 0.0, 1.0],
            -angle,
        );
        add_box(
            &mut mesh,
            [12, 0],
            [-1.0, -5.0, 0.0],
            [2, 10, 0],
            [0.0, 0.0, 0.0],
            std::f32::consts::FRAC_PI_2,
        );
        add_box(
            &mut mesh,
            [0, 10],
            [0.0, -4.0, -0.99],
            [5, 8, 1],
            [page_x, 0.0, 0.0],
            angle,
        );
        add_box(
            &mut mesh,
            [12, 10],
            [0.0, -4.0, -0.01],
            [5, 8, 1],
            [page_x, 0.0, 0.0],
            -angle,
        );
        add_box(
            &mut mesh,
            [24, 10],
            [0.0, -4.0, 0.0],
            [5, 8, 0],
            [page_x, 0.0, 0.0],
            angle - angle * 2.0 * pageFlipRight,
        );
        add_box(
            &mut mesh,
            [24, 10],
            [0.0, -4.0, 0.0],
            [5, 8, 0],
            [page_x, 0.0, 0.0],
            angle - angle * 2.0 * pageFlipLeft,
        );
        mesh
    }
}

fn add_box(
    mesh: &mut BookMesh,
    texture: [i32; 2],
    origin: [f32; 3],
    size: [i32; 3],
    pivot: [f32; 3],
    rotate_y: f32,
) {
    let geometry = model_box_geometry(
        texture,
        origin,
        size,
        0.0,
        false,
        ModelBook::TEXTURE_WIDTH,
        ModelBook::TEXTURE_HEIGHT,
    );
    let rotation = ModelBoxRotation::new([0.0, rotate_y, 0.0]);
    let base = mesh.vertices.len() as u32;
    mesh.vertices.reserve(geometry.len());
    for vertex in geometry.iter() {
        let point = rotation.apply(vertex.position);
        mesh.vertices.push(BookVertex {
            position: [
                (point[0] + pivot[0]) * 0.0625,
                (point[1] + pivot[1]) * 0.0625,
                (point[2] + pivot[2]) * 0.0625,
            ],
            uv: vertex.uv,
        });
    }
    mesh.indices.reserve(MODEL_BOX_FACE_INDICES.len());
    mesh.indices
        .extend(MODEL_BOX_FACE_INDICES.iter().map(|index| base + index));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_contains_all_seven_source_parts() {
        let mesh = ModelBook::buildMesh(0.0, 0.0, 1.0, 1.0);
        assert_eq!(mesh.vertices.len(), 7 * 6 * 4);
        assert_eq!(mesh.indices.len(), 7 * 6 * 6);
    }

    #[test]
    fn page_factors_move_the_two_flipping_pages_independently() {
        let left = ModelBook::buildMesh(10.0, 0.0, 1.0, 1.0);
        let right = ModelBook::buildMesh(10.0, 1.0, 0.0, 1.0);
        assert_ne!(left.vertices, right.vertices);
    }
}
