use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup, LivingRenderInput,
};

/// MCP 1.12.2 `ModelBlaze`.
pub struct ModelBlaze;

impl ModelBlaze {
    pub fn boxes(input: LivingRenderInput) -> Vec<LivingModelBox> {
        let mut boxes = Vec::with_capacity(13);
        boxes.push(model_box(
            [0, 0],
            [-4.0, -4.0, -4.0],
            [8, 8, 8],
            0.0,
            false,
            PartPose {
                pivot: [0.0; 3],
                rotation: [
                    input.headPitch.to_radians(),
                    (input.headYaw - input.bodyYaw).to_radians(),
                    0.0,
                ],
            },
            LivingModelGroup::Head,
        ));

        let mut phase = input.ageInTicks * std::f32::consts::PI * -0.1;
        for i in 0..4 {
            boxes.push(model_box(
                [0, 16],
                [0.0, 0.0, 0.0],
                [2, 8, 2],
                0.0,
                false,
                PartPose {
                    pivot: [
                        phase.cos() * 9.0,
                        -2.0 + (((i * 2) as f32 + input.ageInTicks) * 0.25).cos(),
                        phase.sin() * 9.0,
                    ],
                    rotation: [0.0; 3],
                },
                LivingModelGroup::Body,
            ));
            phase += 1.0;
        }

        phase = std::f32::consts::FRAC_PI_4 + input.ageInTicks * std::f32::consts::PI * 0.03;
        for i in 4..8 {
            boxes.push(model_box(
                [0, 16],
                [0.0, 0.0, 0.0],
                [2, 8, 2],
                0.0,
                false,
                PartPose {
                    pivot: [
                        phase.cos() * 7.0,
                        2.0 + (((i * 2) as f32 + input.ageInTicks) * 0.25).cos(),
                        phase.sin() * 7.0,
                    ],
                    rotation: [0.0; 3],
                },
                LivingModelGroup::Body,
            ));
            phase += 1.0;
        }

        phase = 0.471_238_94 + input.ageInTicks * std::f32::consts::PI * -0.05;
        for i in 8..12 {
            boxes.push(model_box(
                [0, 16],
                [0.0, 0.0, 0.0],
                [2, 8, 2],
                0.0,
                false,
                PartPose {
                    pivot: [
                        phase.cos() * 5.0,
                        11.0 + ((i as f32 * 1.5 + input.ageInTicks) * 0.5).cos(),
                        phase.sin() * 5.0,
                    ],
                    rotation: [0.0; 3],
                },
                LivingModelGroup::Body,
            ));
            phase += 1.0;
        }
        boxes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::client::renderer::entity::RenderLivingBase::LivingChildLayout;

    fn input(age: f32) -> LivingRenderInput {
        LivingRenderInput {
            position: [0.0; 3],
            bodyYaw: 10.0,
            headYaw: 40.0,
            headPitch: 20.0,
            limbSwing: 0.0,
            limbSwingAmount: 0.0,
            ageInTicks: age,
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
    fn creates_twelve_rods_and_one_head() {
        let boxes = ModelBlaze::boxes(input(0.0));
        assert_eq!(boxes.len(), 13);
        assert_eq!(boxes[0].size, [8, 8, 8]);
        assert_eq!(boxes[1].size, [2, 8, 2]);
    }

    #[test]
    fn first_ring_starts_at_mcp_phase() {
        let boxes = ModelBlaze::boxes(input(0.0));
        assert!((boxes[1].pose.pivot[0] - 9.0).abs() < 1.0e-6);
        assert!((boxes[1].pose.pivot[1] + 1.0).abs() < 1.0e-6);
        assert!(boxes[1].pose.pivot[2].abs() < 1.0e-6);
    }
}
