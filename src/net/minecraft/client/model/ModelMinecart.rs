use crate::net::minecraft::client::model::ModelVehicleBox::{
    append_box, VehicleBox, VehicleModelMesh,
};

/// MCP 1.12.2 `ModelMinecart`.
pub struct ModelMinecart;

impl ModelMinecart {
    pub const TEXTURE_SIZE: [f32; 2] = [64.0, 32.0];

    /// RenderMinecart passes ageInTicks=-0.1, moving the inner floor from 4 to 4.1.
    pub fn buildMesh(ageInTicks: f32) -> VehicleModelMesh {
        let mut mesh = VehicleModelMesh::default();
        for spec in [
            VehicleBox {
                texture: [0, 10],
                textureSize: Self::TEXTURE_SIZE,
                origin: [-10.0, -8.0, -1.0],
                size: [20, 16, 2],
                pivot: [0.0, 4.0, 0.0],
                rotation: [std::f32::consts::FRAC_PI_2, 0.0, 0.0],
            },
            VehicleBox {
                texture: [0, 0],
                textureSize: Self::TEXTURE_SIZE,
                origin: [-8.0, -9.0, -1.0],
                size: [16, 8, 2],
                pivot: [-9.0, 4.0, 0.0],
                rotation: [0.0, std::f32::consts::PI * 1.5, 0.0],
            },
            VehicleBox {
                texture: [0, 0],
                textureSize: Self::TEXTURE_SIZE,
                origin: [-8.0, -9.0, -1.0],
                size: [16, 8, 2],
                pivot: [9.0, 4.0, 0.0],
                rotation: [0.0, std::f32::consts::FRAC_PI_2, 0.0],
            },
            VehicleBox {
                texture: [0, 0],
                textureSize: Self::TEXTURE_SIZE,
                origin: [-8.0, -9.0, -1.0],
                size: [16, 8, 2],
                pivot: [0.0, 4.0, -7.0],
                rotation: [0.0, std::f32::consts::PI, 0.0],
            },
            VehicleBox {
                texture: [0, 0],
                textureSize: Self::TEXTURE_SIZE,
                origin: [-8.0, -9.0, -1.0],
                size: [16, 8, 2],
                pivot: [0.0, 4.0, 7.0],
                rotation: [0.0, 0.0, 0.0],
            },
            VehicleBox {
                texture: [44, 10],
                textureSize: Self::TEXTURE_SIZE,
                origin: [-9.0, -7.0, -1.0],
                size: [18, 14, 1],
                pivot: [0.0, 4.0 - ageInTicks, 0.0],
                rotation: [-std::f32::consts::FRAC_PI_2, 0.0, 0.0],
            },
        ] {
            append_box(&mut mesh, spec);
        }
        mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_parts_emit_thirty_six_quads() {
        let mesh = ModelMinecart::buildMesh(-0.1);
        assert_eq!(mesh.vertices.len(), 36 * 4);
        assert_eq!(mesh.indices.len(), 36 * 6);
    }
}
