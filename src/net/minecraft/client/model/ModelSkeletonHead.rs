use crate::net::minecraft::client::model::ModelVehicleBox::{
    append_box, VehicleBox, VehicleModelMesh,
};

/// Exact one-box MCP 1.12.2 `ModelSkeletonHead` used by `RenderWitherSkull`.
pub struct ModelSkeletonHead;

impl ModelSkeletonHead {
    pub const TEXTURE_SIZE: [f32; 2] = [64.0, 64.0];

    pub fn buildMesh(netHeadYaw: f32, headPitch: f32) -> VehicleModelMesh {
        let mut mesh = VehicleModelMesh::default();
        append_box(
            &mut mesh,
            VehicleBox {
                texture: [0, 35],
                textureSize: Self::TEXTURE_SIZE,
                origin: [-4.0, -8.0, -4.0],
                size: [8, 8, 8],
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
    fn skull_head_is_one_eight_pixel_cube() {
        let mesh = ModelSkeletonHead::buildMesh(0.0, 0.0);
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
    }
}
