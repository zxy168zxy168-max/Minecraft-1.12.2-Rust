use crate::net::minecraft::client::model::ModelVehicleBox::{
    append_box, VehicleBox, VehicleModelMesh,
};

/// Exact box definitions owned by MCP 1.12.2 `ModelEnderCrystal`.
pub struct ModelEnderCrystal;

impl ModelEnderCrystal {
    pub const TEXTURE_SIZE: [f32; 2] = [64.0, 32.0];
    pub const ROOT_SCALE: f32 = 2.0;
    pub const ROOT_TRANSLATE_Y: f32 = -0.5;
    pub const FLOAT_TRANSLATE_Y: f32 = 0.8;
    pub const DIAGONAL_ROTATION_DEGREES: f32 = 60.0;
    pub const DIAGONAL_ROTATION_AXIS: [f32; 3] = [0.7071, 0.0, 0.7071];
    pub const NESTED_SCALE: f32 = 0.875;

    pub fn glassMesh() -> VehicleModelMesh {
        Self::boxMesh([0, 0], [-4.0, -4.0, -4.0], [8, 8, 8])
    }

    pub fn cubeMesh() -> VehicleModelMesh {
        Self::boxMesh([32, 0], [-4.0, -4.0, -4.0], [8, 8, 8])
    }

    pub fn baseMesh() -> VehicleModelMesh {
        Self::boxMesh([0, 16], [-6.0, 0.0, -6.0], [12, 4, 12])
    }

    fn boxMesh(texture: [i32; 2], origin: [f32; 3], size: [i32; 3]) -> VehicleModelMesh {
        let mut mesh = VehicleModelMesh::default();
        append_box(
            &mut mesh,
            VehicleBox {
                texture,
                textureSize: Self::TEXTURE_SIZE,
                origin,
                size,
                pivot: [0.0; 3],
                rotation: [0.0; 3],
            },
        );
        mesh
    }
}
