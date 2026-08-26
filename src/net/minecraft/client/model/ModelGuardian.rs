use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup, LivingRenderInput,
};

/// Render-only inputs read by MCP 1.12.2 `ModelGuardian#setRotationAngles`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuardianModelState {
    pub spikesAnimation: f32,
    pub tailAnimation: f32,
    /// `EntityGuardian#getPositionEyes(0.0F)`.
    pub guardianEyes: [f64; 3],
    /// Targeted entity eyes, or the render-view entity eyes when no target exists.
    pub focusEyes: Option<[f64; 3]>,
    /// `EntityGuardian#getLook(0.0F)` before its Y component is discarded.
    pub guardianLook: [f64; 3],
}

/// MCP 1.12.2 `ModelGuardian`.
pub struct ModelGuardian;

impl ModelGuardian {
    const SPINE_ROT_X: [f32; 12] = [
        1.75, 0.25, 0.0, 0.0, 0.5, 0.5, 0.5, 0.5, 1.25, 0.75, 0.0, 0.0,
    ];
    const SPINE_ROT_Y: [f32; 12] = [
        0.0, 0.0, 0.0, 0.0, 0.25, 1.75, 1.25, 0.75, 0.0, 0.0, 0.0, 0.0,
    ];
    const SPINE_ROT_Z: [f32; 12] = [
        0.0, 0.0, 0.25, 1.75, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.75, 1.25,
    ];
    const SPINE_X: [f32; 12] = [
        0.0, 0.0, 8.0, -8.0, -8.0, 8.0, 8.0, -8.0, 0.0, 0.0, 8.0, -8.0,
    ];
    const SPINE_Y: [f32; 12] = [
        -8.0, -8.0, -8.0, -8.0, 0.0, 0.0, 0.0, 0.0, 8.0, 8.0, 8.0, 8.0,
    ];
    const SPINE_Z: [f32; 12] = [
        8.0, -8.0, 0.0, 0.0, -8.0, -8.0, 8.0, 8.0, 8.0, -8.0, 0.0, 0.0,
    ];

    pub fn boxes(input: LivingRenderInput, state: GuardianModelState) -> Vec<LivingModelBox> {
        let bodyPose = PartPose {
            pivot: [0.0; 3],
            rotation: [
                input.headPitch.to_radians(),
                (input.headYaw - input.bodyYaw).to_radians(),
                0.0,
            ],
        };
        let mut boxes = Vec::with_capacity(22);
        boxes.push(model_box(
            [0, 0],
            [-6.0, 10.0, -8.0],
            [12, 12, 16],
            0.0,
            false,
            bodyPose,
            LivingModelGroup::Body,
        ));
        boxes.push(model_box(
            [0, 28],
            [-8.0, 10.0, -6.0],
            [2, 12, 12],
            0.0,
            false,
            bodyPose,
            LivingModelGroup::Body,
        ));
        boxes.push(model_box(
            [0, 28],
            [6.0, 10.0, -6.0],
            [2, 12, 12],
            0.0,
            true,
            bodyPose,
            LivingModelGroup::Body,
        ));
        boxes.push(model_box(
            [16, 40],
            [-6.0, 8.0, -6.0],
            [12, 2, 12],
            0.0,
            false,
            bodyPose,
            LivingModelGroup::Body,
        ));
        boxes.push(model_box(
            [16, 40],
            [-6.0, 22.0, -6.0],
            [12, 2, 12],
            0.0,
            false,
            bodyPose,
            LivingModelGroup::Body,
        ));

        let extension = (1.0 - state.spikesAnimation) * 0.55;
        for i in 0..12 {
            let scale = 1.0 + (input.ageInTicks * 1.5 + i as f32).cos() * 0.01 - extension;
            let mut spine = model_box(
                [0, 0],
                [-1.0, -4.5, -1.0],
                [2, 9, 2],
                0.0,
                false,
                PartPose {
                    pivot: [
                        Self::SPINE_X[i] * scale,
                        16.0 + Self::SPINE_Y[i] * scale,
                        Self::SPINE_Z[i] * scale,
                    ],
                    rotation: [
                        std::f32::consts::PI * Self::SPINE_ROT_X[i],
                        std::f32::consts::PI * Self::SPINE_ROT_Y[i],
                        std::f32::consts::PI * Self::SPINE_ROT_Z[i],
                    ],
                },
                LivingModelGroup::Body,
            );
            spine.parentPose = Some(bodyPose);
            boxes.push(spine);
        }

        let (eyeX, eyeY) = Self::eyeOffsets(state);
        let mut eye = model_box(
            [8, 0],
            [-1.0, 15.0, 0.0],
            [2, 2, 1],
            0.0,
            false,
            PartPose {
                pivot: [eyeX, eyeY, -8.25],
                rotation: [0.0; 3],
            },
            LivingModelGroup::Body,
        );
        eye.parentPose = Some(bodyPose);
        boxes.push(eye);

        let tail0Pose = PartPose {
            pivot: [0.0; 3],
            rotation: [
                0.0,
                state.tailAnimation.sin() * std::f32::consts::PI * 0.05,
                0.0,
            ],
        };
        let tail1Pose = PartPose {
            pivot: [-1.5, 0.5, 14.0],
            rotation: [
                0.0,
                state.tailAnimation.sin() * std::f32::consts::PI * 0.1,
                0.0,
            ],
        };
        let tail2Pose = PartPose {
            pivot: [0.5, 0.5, 6.0],
            rotation: [
                0.0,
                state.tailAnimation.sin() * std::f32::consts::PI * 0.15,
                0.0,
            ],
        };

        let mut tail0 = model_box(
            [40, 0],
            [-2.0, 14.0, 7.0],
            [4, 4, 8],
            0.0,
            false,
            tail0Pose,
            LivingModelGroup::Body,
        );
        tail0.parentPose = Some(bodyPose);
        boxes.push(tail0);

        let mut tail1 = model_box(
            [0, 54],
            [0.0, 14.0, 0.0],
            [3, 3, 7],
            0.0,
            false,
            tail1Pose,
            LivingModelGroup::Body,
        );
        tail1.parentPose = Some(tail0Pose);
        tail1.parentPose2 = Some(bodyPose);
        boxes.push(tail1);

        for (texture, origin, size) in [
            ([41, 32], [0.0, 14.0, 0.0], [2, 2, 6]),
            ([25, 19], [1.0, 10.5, 3.0], [1, 9, 9]),
        ] {
            let mut tail2 = model_box(
                texture,
                origin,
                size,
                0.0,
                false,
                tail2Pose,
                LivingModelGroup::Body,
            );
            tail2.parentPose = Some(tail1Pose);
            tail2.parentPose2 = Some(tail0Pose);
            tail2.parentPose3 = Some(bodyPose);
            boxes.push(tail2);
        }
        boxes
    }

    fn eyeOffsets(state: GuardianModelState) -> (f32, f32) {
        let Some(focus) = state.focusEyes else {
            return (0.0, 0.0);
        };
        let eyeY = if focus[1] - state.guardianEyes[1] > 0.0 {
            0.0
        } else {
            1.0
        };
        let dx = state.guardianEyes[0] - focus[0];
        let dz = state.guardianEyes[2] - focus[2];
        let length = (dx * dx + dz * dz).sqrt();
        if length <= 1.0e-12 {
            return (0.0, eyeY);
        }
        // Vec3d.normalize().rotateYaw(PI/2): (x,z) -> (z,-x).
        let rotatedX = dz / length;
        let rotatedZ = -dx / length;
        let dot = (state.guardianLook[0] * rotatedX + state.guardianLook[2] * rotatedZ) as f32;
        (dot.abs().sqrt() * 2.0 * dot.signum(), eyeY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::client::renderer::entity::RenderLivingBase::LivingChildLayout;

    fn input() -> LivingRenderInput {
        LivingRenderInput {
            position: [0.0; 3],
            bodyYaw: 0.0,
            headYaw: 0.0,
            headPitch: 0.0,
            limbSwing: 0.0,
            limbSwingAmount: 0.0,
            ageInTicks: 0.0,
            swingProgress: 0.0,
            sneaking: false,
            child: false,
            deathRotation: 0.0,
            preScale: 1.0,
            preScaleXYZ: [1.0; 3],
            childLayout: LivingChildLayout::BIPED,
            adultTranslation: [0.0; 3],
        }
    }

    #[test]
    fn model_contains_body_spines_eye_and_tail_boxes() {
        let boxes = ModelGuardian::boxes(
            input(),
            GuardianModelState {
                spikesAnimation: 1.0,
                tailAnimation: 0.0,
                guardianEyes: [0.0, 0.0, 0.0],
                focusEyes: None,
                guardianLook: [0.0, 0.0, 1.0],
            },
        );
        assert_eq!(boxes.len(), 22);
        assert_eq!(boxes[0].size, [12, 12, 16]);
        assert_eq!(boxes[5].size, [2, 9, 2]);
        assert_eq!(boxes[17].size, [2, 2, 1]);
    }

    #[test]
    fn fully_extended_first_spine_uses_mcp_position() {
        let boxes = ModelGuardian::boxes(
            input(),
            GuardianModelState {
                spikesAnimation: 1.0,
                tailAnimation: 0.0,
                guardianEyes: [0.0; 3],
                focusEyes: None,
                guardianLook: [0.0, 0.0, 1.0],
            },
        );
        assert!((boxes[5].pose.pivot[1] - 7.92).abs() < 1.0e-5);
        assert!((boxes[5].pose.pivot[2] - 8.08).abs() < 1.0e-5);
    }

    #[test]
    fn eye_tracks_focus_side_with_vanilla_square_root_curve() {
        let state = GuardianModelState {
            spikesAnimation: 1.0,
            tailAnimation: 0.0,
            guardianEyes: [0.0, 1.0, 0.0],
            focusEyes: Some([1.0, 2.0, 0.0]),
            guardianLook: [0.0, 0.0, 1.0],
        };
        let (x, y) = ModelGuardian::eyeOffsets(state);
        assert!((x - 2.0).abs() < 1.0e-6);
        assert_eq!(y, 0.0);
    }
}
