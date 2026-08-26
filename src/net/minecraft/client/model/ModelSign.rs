use crate::net::minecraft::client::model::ModelVehicleBox::{
    append_box, VehicleBox, VehicleModelMesh,
};

/// Exact box geometry from MCP 1.12.2 `ModelSign`.
pub struct ModelSign;

impl ModelSign {
    pub fn buildMesh(showStick: bool) -> VehicleModelMesh {
        let mut mesh = VehicleModelMesh::default();
        append_box(
            &mut mesh,
            VehicleBox {
                texture: [0, 0],
                textureSize: [64.0, 32.0],
                origin: [-12.0, -14.0, -1.0],
                size: [24, 12, 2],
                pivot: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
            },
        );
        if showStick {
            append_box(
                &mut mesh,
                VehicleBox {
                    texture: [0, 14],
                    textureSize: [64.0, 32.0],
                    origin: [-1.0, -2.0, -1.0],
                    size: [2, 14, 2],
                    pivot: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                },
            );
        }
        mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standing_sign_adds_stick_but_wall_sign_does_not() {
        let standing = ModelSign::buildMesh(true);
        let wall = ModelSign::buildMesh(false);
        assert_eq!(standing.indices.len(), 72);
        assert_eq!(wall.indices.len(), 36);
    }
}
