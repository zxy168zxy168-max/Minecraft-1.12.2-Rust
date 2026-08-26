use crate::net::minecraft::client::model::ModelVehicleBox::{
    append_box, VehicleBox, VehicleModelMesh,
};

/// Exact single-box model from MCP 1.12.2 `ModelLeashKnot`.
pub struct ModelLeashKnot;

impl ModelLeashKnot {
    pub const TEXTURE_SIZE: [f32; 2] = [32.0, 32.0];
    pub const MODEL_SCALE: f32 = 0.0625;

    pub fn buildMesh(netHeadYaw: f32, headPitch: f32) -> VehicleModelMesh {
        let mut mesh = VehicleModelMesh::default();
        append_box(
            &mut mesh,
            VehicleBox {
                texture: [0, 0],
                textureSize: Self::TEXTURE_SIZE,
                origin: [-3.0, -6.0, -3.0],
                size: [6, 8, 6],
                pivot: [0.0, 0.0, 0.0],
                rotation: [headPitch.to_radians(), netHeadYaw.to_radians(), 0.0],
            },
        );
        mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knot_is_one_six_face_model_box() {
        let mesh = ModelLeashKnot::buildMesh(0.0, 0.0);
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
    }
}
