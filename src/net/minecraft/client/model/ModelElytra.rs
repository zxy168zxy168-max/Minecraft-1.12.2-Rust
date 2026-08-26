use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelPlayer::ModelBoxSpec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElytraRotationState {
    pub rotateX: f32,
    pub rotateY: f32,
    pub rotateZ: f32,
}

impl Default for ElytraRotationState {
    fn default() -> Self {
        // AbstractClientPlayer fields are Java-zero-initialized. The first
        // ModelElytra render moves each value 10% toward its target.
        Self {
            rotateX: 0.0,
            rotateY: 0.0,
            rotateZ: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElytraPose {
    pub leftWing: PartPose,
    pub rightWing: PartPose,
    pub rotations: ElytraRotationState,
}

pub struct ModelElytra;

impl ModelElytra {
    /// Exact target angles and `AbstractClientPlayer.rotateElytra*` smoothing
    /// from MCP 1.12.2 `ModelElytra#setRotationAngles`.
    pub fn setRotationAngles(
        sneaking: bool,
        elytraFlying: bool,
        motion: [f64; 3],
        previous: ElytraRotationState,
    ) -> ElytraPose {
        let mut rotateX = 0.2617994_f32;
        let mut rotateZ = -0.2617994_f32;
        let mut pivotY = 0.0_f32;
        let mut rotateY = 0.0_f32;

        if elytraFlying {
            let mut factor = 1.0_f32;
            if motion[1] < 0.0 {
                let length =
                    (motion[0] * motion[0] + motion[1] * motion[1] + motion[2] * motion[2]).sqrt();
                if length >= 1.0e-4 {
                    let normalizedY = motion[1] / length;
                    factor = 1.0 - (-normalizedY).powf(1.5) as f32;
                }
            }
            rotateX = factor * 0.34906584 + (1.0 - factor) * rotateX;
            rotateZ = factor * -std::f32::consts::FRAC_PI_2 + (1.0 - factor) * rotateZ;
        } else if sneaking {
            rotateX = std::f32::consts::TAU / 9.0;
            rotateZ = -std::f32::consts::FRAC_PI_4;
            pivotY = 3.0;
            rotateY = 0.08726646;
        }

        let rotations = ElytraRotationState {
            rotateX: (f64::from(previous.rotateX) + f64::from(rotateX - previous.rotateX) * 0.1_f64)
                as f32,
            rotateY: (f64::from(previous.rotateY) + f64::from(rotateY - previous.rotateY) * 0.1_f64)
                as f32,
            rotateZ: (f64::from(previous.rotateZ) + f64::from(rotateZ - previous.rotateZ) * 0.1_f64)
                as f32,
        };
        let leftWing = PartPose {
            pivot: [5.0, pivotY, 0.0],
            rotation: [rotations.rotateX, rotations.rotateY, rotations.rotateZ],
        };
        let rightWing = PartPose {
            pivot: [-5.0, pivotY, 0.0],
            rotation: [rotations.rotateX, -rotations.rotateY, -rotations.rotateZ],
        };
        ElytraPose {
            leftWing,
            rightWing,
            rotations,
        }
    }

    /// Reconstructs the rendered wing pivots from the already-smoothed
    /// `AbstractClientPlayer.rotateElytra*` values. This avoids advancing the
    /// 10% interpolator a second time while the Vulkan mesh is baked.
    pub fn poseFromRotations(sneaking: bool, rotations: ElytraRotationState) -> ElytraPose {
        let pivotY = if sneaking { 3.0 } else { 0.0 };
        ElytraPose {
            leftWing: PartPose {
                pivot: [5.0, pivotY, 0.0],
                rotation: [rotations.rotateX, rotations.rotateY, rotations.rotateZ],
            },
            rightWing: PartPose {
                pivot: [-5.0, pivotY, 0.0],
                rotation: [rotations.rotateX, -rotations.rotateY, -rotations.rotateZ],
            },
            rotations,
        }
    }

    pub fn boxes(pose: ElytraPose) -> [ModelBoxSpec; 2] {
        [
            ModelBoxSpec {
                texture: [22, 0],
                origin: [-10.0, 0.0, 0.0],
                size: [10, 20, 2],
                delta: 1.0,
                mirror: false,
                pose: pose.leftWing,
            },
            ModelBoxSpec {
                texture: [22, 0],
                origin: [0.0, 0.0, 0.0],
                size: [10, 20, 2],
                delta: 1.0,
                mirror: true,
                pose: pose.rightWing,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_player_rotation_fields_start_at_zero_and_advance_ten_percent() {
        let initial = ElytraRotationState::default();
        assert_eq!(
            initial,
            ElytraRotationState {
                rotateX: 0.0,
                rotateY: 0.0,
                rotateZ: 0.0
            }
        );
        let pose = ModelElytra::setRotationAngles(false, false, [0.0; 3], initial);
        assert!((pose.rotations.rotateX - 0.02617994).abs() < 1.0e-7);
        assert!((pose.rotations.rotateZ + 0.02617994).abs() < 1.0e-7);
    }

    #[test]
    fn sneaking_uses_vanilla_pivot_and_angles() {
        let pose =
            ModelElytra::setRotationAngles(true, false, [0.0; 3], ElytraRotationState::default());
        assert!(pose.leftWing.pivot[1] > 0.0);
        assert!(pose.leftWing.rotation[1] > 0.0);
        assert_eq!(pose.rightWing.rotation[1], -pose.leftWing.rotation[1]);
    }

    #[test]
    fn wings_use_mirrored_ten_by_twenty_by_two_boxes() {
        let boxes = ModelElytra::boxes(ModelElytra::setRotationAngles(
            false,
            false,
            [0.0; 3],
            ElytraRotationState::default(),
        ));
        assert_eq!(boxes[0].size, [10, 20, 2]);
        assert!(!boxes[0].mirror);
        assert!(boxes[1].mirror);
    }
}
