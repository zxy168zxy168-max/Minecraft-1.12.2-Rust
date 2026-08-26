use crate::net::minecraft::client::model::ModelBoxGeometry::{
    model_box_geometry, ModelBoxRotation, MODEL_BOX_FACE_INDICES,
};

/// A model-space vertex emitted by MCP 1.12.2 `ModelShulkerBullet` before
/// `RenderShulkerBullet` applies its entity and decorative rotations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerBulletModelVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShulkerBulletModelMesh {
    pub vertices: Vec<ShulkerBulletModelVertex>,
    pub indices: Vec<u32>,
}

/// MCP 1.12.2 `ModelShulkerBullet`.
pub struct ModelShulkerBullet;

impl ModelShulkerBullet {
    pub const TEXTURE_WIDTH: f32 = 64.0;
    pub const TEXTURE_HEIGHT: f32 = 32.0;
    pub const SCALE: f32 = 0.03125;

    pub fn buildMesh(netHeadYaw: f32, headPitch: f32) -> ShulkerBulletModelMesh {
        let mut mesh = ShulkerBulletModelMesh {
            vertices: Vec::with_capacity(72),
            indices: Vec::with_capacity(108),
        };
        append_box(
            &mut mesh,
            [0, 0],
            [-4.0, -4.0, -1.0],
            [8, 8, 2],
            netHeadYaw,
            headPitch,
        );
        append_box(
            &mut mesh,
            [0, 10],
            [-1.0, -4.0, -4.0],
            [2, 8, 8],
            netHeadYaw,
            headPitch,
        );
        append_box(
            &mut mesh,
            [20, 0],
            [-4.0, -1.0, -4.0],
            [8, 2, 8],
            netHeadYaw,
            headPitch,
        );
        mesh
    }
}

fn append_box(
    mesh: &mut ShulkerBulletModelMesh,
    texture: [i32; 2],
    origin: [f32; 3],
    size: [i32; 3],
    yawDegrees: f32,
    pitchDegrees: f32,
) {
    let geometry = model_box_geometry(
        texture,
        origin,
        size,
        0.0,
        false,
        ModelShulkerBullet::TEXTURE_WIDTH,
        ModelShulkerBullet::TEXTURE_HEIGHT,
    );
    let rotation = ModelBoxRotation::new([pitchDegrees.to_radians(), yawDegrees.to_radians(), 0.0]);
    let base = mesh.vertices.len() as u32;
    mesh.vertices.reserve(geometry.len());
    for vertex in geometry.iter() {
        let point = rotation.apply(vertex.position);
        mesh.vertices.push(ShulkerBulletModelVertex {
            position: [
                point[0] * ModelShulkerBullet::SCALE,
                point[1] * ModelShulkerBullet::SCALE,
                point[2] * ModelShulkerBullet::SCALE,
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
    fn three_crossed_boxes_emit_eighteen_quads() {
        let mesh = ModelShulkerBullet::buildMesh(0.0, 0.0);
        assert_eq!(mesh.vertices.len(), 72);
        assert_eq!(mesh.indices.len(), 108);
    }
}
