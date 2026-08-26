use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup, LivingRenderInput,
};

/// Render-only interpolation owned by MCP 1.12.2 `EntityShulker` and consumed
/// by `ModelShulker#setRotationAngles`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerModelState {
    pub clientPeekAmount: f32,
}

/// MCP 1.12.2 `ModelShulker`.
pub struct ModelShulker;

impl ModelShulker {
    pub const TEXTURE_WIDTH: f32 = 64.0;
    pub const TEXTURE_HEIGHT: f32 = 64.0;

    /// `ModelShulker#render` deliberately renders only base and lid. The head
    /// is submitted by `RenderShulker.HeadLayer` after its attachment-facing
    /// correction matrix has been applied.
    pub fn shellBoxes(input: LivingRenderInput, state: ShulkerModelState) -> Vec<LivingModelBox> {
        let peek = state.clientPeekAmount;
        let phase = (0.5 + peek) * std::f32::consts::PI;
        let sinMinusOne = -1.0 + phase.sin();
        let bob = if phase > std::f32::consts::PI {
            (input.ageInTicks * 0.1).sin() * 0.7
        } else {
            0.0
        };
        let lidYaw = if peek > 0.3 {
            sinMinusOne.powi(4) * std::f32::consts::PI * 0.125
        } else {
            0.0
        };

        vec![
            model_box(
                [0, 28],
                [-8.0, -8.0, -8.0],
                [16, 8, 16],
                0.0,
                false,
                PartPose {
                    pivot: [0.0, 24.0, 0.0],
                    rotation: [0.0; 3],
                },
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 0],
                [-8.0, -16.0, -8.0],
                [16, 12, 16],
                0.0,
                false,
                PartPose {
                    pivot: [0.0, 16.0 + phase.sin() * 8.0 + bob, 0.0],
                    rotation: [0.0, lidYaw, 0.0],
                },
                LivingModelGroup::Body,
            ),
        ]
    }

    pub fn headBoxes(input: LivingRenderInput) -> Vec<LivingModelBox> {
        vec![model_box(
            [0, 52],
            [-3.0, 0.0, -3.0],
            [6, 6, 6],
            0.0,
            false,
            PartPose {
                pivot: [0.0, 12.0, 0.0],
                rotation: [
                    input.headPitch.to_radians(),
                    (input.headYaw - input.bodyYaw).to_radians(),
                    0.0,
                ],
            },
            LivingModelGroup::Head,
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::client::renderer::entity::RenderLivingBase::LivingChildLayout;

    fn input(age: f32) -> LivingRenderInput {
        LivingRenderInput {
            position: [0.0; 3],
            bodyYaw: 180.0,
            headYaw: 180.0,
            headPitch: 0.0,
            limbSwing: 0.0,
            limbSwingAmount: 0.0,
            ageInTicks: age,
            swingProgress: 0.0,
            sneaking: false,
            child: false,
            deathRotation: 0.0,
            preScale: 0.999,
            preScaleXYZ: [0.999; 3],
            childLayout: LivingChildLayout::BIPED,
            adultTranslation: [0.0; 3],
        }
    }

    #[test]
    fn closed_lid_uses_source_pivot_and_zero_yaw() {
        let boxes = ModelShulker::shellBoxes(
            input(20.0),
            ShulkerModelState {
                clientPeekAmount: 0.0,
            },
        );
        assert_eq!(boxes[1].pose.pivot, [0.0, 24.0, 0.0]);
        assert_eq!(boxes[1].pose.rotation[1], 0.0);
    }

    #[test]
    fn half_open_lid_uses_source_translation_and_fourth_power_rotation() {
        let boxes = ModelShulker::shellBoxes(
            input(20.0),
            ShulkerModelState {
                clientPeekAmount: 0.5,
            },
        );
        assert!((boxes[1].pose.pivot[1] - 16.0).abs() < 1.0e-5);
        assert!((boxes[1].pose.rotation[1] - std::f32::consts::PI * 0.125).abs() < 1.0e-5);
    }
}
