use crate::net::minecraft::client::model::ModelVehicleBox::{
    append_box, VehicleBox, VehicleModelMesh,
};

/// MCP 1.12.2 `ModelBoat`.
pub struct ModelBoat;

impl ModelBoat {
    pub const TEXTURE_SIZE: [f32; 2] = [128.0, 64.0];
    pub const MODEL_ROTATION_Y: f32 = 90.0;

    pub fn buildMesh(rowingTime: [f32; 2]) -> VehicleModelMesh {
        let mut mesh = VehicleModelMesh::default();
        for spec in Self::sideBoxes() {
            append_box(&mut mesh, spec);
        }
        for paddle in 0..2 {
            let f = rowingTime[paddle];
            let rotationX = clamped_lerp(
                -1.0471975803375244,
                -0.2617993950843811,
                ((-f).sin() + 1.0) * 0.5,
            );
            let mut rotationY = clamped_lerp(
                -std::f32::consts::FRAC_PI_4,
                std::f32::consts::FRAC_PI_4,
                ((-f + 1.0).sin() + 1.0) * 0.5,
            );
            if paddle == 1 {
                rotationY = std::f32::consts::PI - rotationY;
            }
            let texture = if paddle == 0 { [62, 0] } else { [62, 20] };
            let pivot = if paddle == 0 {
                [3.0, -5.0, 9.0]
            } else {
                [3.0, -5.0, -9.0]
            };
            let bladeX = if paddle == 0 { -1.001 } else { 0.001 };
            for (origin, size) in [
                ([-1.0, 0.0, -5.0], [2, 2, 18]),
                ([bladeX, -3.0, 8.0], [1, 6, 7]),
            ] {
                append_box(
                    &mut mesh,
                    VehicleBox {
                        texture,
                        textureSize: Self::TEXTURE_SIZE,
                        origin,
                        size,
                        pivot,
                        rotation: [rotationX, rotationY, 0.19634955],
                    },
                );
            }
        }
        mesh
    }

    /// Geometry submitted by `ModelBoat#renderMultipass`. Vulkan needs a
    /// depth-only/color-mask pass to consume this mesh without drawing color.
    pub fn buildNoWaterMesh() -> VehicleModelMesh {
        let mut mesh = VehicleModelMesh::default();
        append_box(
            &mut mesh,
            VehicleBox {
                texture: [0, 0],
                textureSize: Self::TEXTURE_SIZE,
                origin: [-14.0, -9.0, -3.0],
                size: [28, 16, 3],
                pivot: [0.0, -3.0, 1.0],
                rotation: [std::f32::consts::FRAC_PI_2, 0.0, 0.0],
            },
        );
        mesh
    }

    fn sideBoxes() -> [VehicleBox; 5] {
        [
            VehicleBox {
                texture: [0, 0],
                textureSize: Self::TEXTURE_SIZE,
                origin: [-14.0, -9.0, -3.0],
                size: [28, 16, 3],
                pivot: [0.0, 3.0, 1.0],
                rotation: [std::f32::consts::FRAC_PI_2, 0.0, 0.0],
            },
            VehicleBox {
                texture: [0, 19],
                textureSize: Self::TEXTURE_SIZE,
                origin: [-13.0, -7.0, -1.0],
                size: [18, 6, 2],
                pivot: [-15.0, 4.0, 4.0],
                rotation: [0.0, std::f32::consts::PI * 1.5, 0.0],
            },
            VehicleBox {
                texture: [0, 27],
                textureSize: Self::TEXTURE_SIZE,
                origin: [-8.0, -7.0, -1.0],
                size: [16, 6, 2],
                pivot: [15.0, 4.0, 0.0],
                rotation: [0.0, std::f32::consts::FRAC_PI_2, 0.0],
            },
            VehicleBox {
                texture: [0, 35],
                textureSize: Self::TEXTURE_SIZE,
                origin: [-14.0, -7.0, -1.0],
                size: [28, 6, 2],
                pivot: [0.0, 4.0, -9.0],
                rotation: [0.0, std::f32::consts::PI, 0.0],
            },
            VehicleBox {
                texture: [0, 43],
                textureSize: Self::TEXTURE_SIZE,
                origin: [-14.0, -7.0, -1.0],
                size: [28, 6, 2],
                pivot: [0.0, 4.0, 9.0],
                rotation: [0.0, 0.0, 0.0],
            },
        ]
    }
}

fn clamped_lerp(start: f32, end: f32, value: f32) -> f32 {
    start + (end - start) * value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_hull_parts_and_four_paddle_boxes_emit_fifty_four_quads() {
        let mesh = ModelBoat::buildMesh([0.0, 0.0]);
        assert_eq!(mesh.vertices.len(), 54 * 4);
        assert_eq!(mesh.indices.len(), 54 * 6);
    }

    #[test]
    fn no_water_is_one_six_face_box() {
        let mesh = ModelBoat::buildNoWaterMesh();
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
    }
}
