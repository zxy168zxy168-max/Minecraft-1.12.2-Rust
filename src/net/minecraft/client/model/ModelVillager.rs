use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup, LivingRenderInput,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VillagerPose {
    pub head: PartPose,
    pub nose: PartPose,
    pub body: PartPose,
    pub arms: PartPose,
    pub rightLeg: PartPose,
    pub leftLeg: PartPose,
}

/// MCP 1.12.2 `ModelVillager` geometry and pose state.
pub struct ModelVillager;

impl ModelVillager {
    pub fn pose(input: LivingRenderInput) -> VillagerPose {
        VillagerPose {
            head: PartPose {
                pivot: [0.0, 0.0, 0.0],
                rotation: [
                    input.headPitch.to_radians(),
                    (input.headYaw - input.bodyYaw).to_radians(),
                    0.0,
                ],
            },
            nose: PartPose {
                pivot: [0.0, -2.0, 0.0],
                rotation: [0.0; 3],
            },
            body: PartPose {
                pivot: [0.0, 0.0, 0.0],
                rotation: [0.0; 3],
            },
            arms: PartPose {
                pivot: [0.0, 3.0, -1.0],
                rotation: [-0.75, 0.0, 0.0],
            },
            rightLeg: PartPose {
                pivot: [-2.0, 12.0, 0.0],
                rotation: [
                    (input.limbSwing * 0.6662).cos() * 1.4 * input.limbSwingAmount * 0.5,
                    0.0,
                    0.0,
                ],
            },
            leftLeg: PartPose {
                pivot: [2.0, 12.0, 0.0],
                rotation: [
                    (input.limbSwing * 0.6662 + std::f32::consts::PI).cos()
                        * 1.4
                        * input.limbSwingAmount
                        * 0.5,
                    0.0,
                    0.0,
                ],
            },
        }
    }

    pub fn boxes(pose: VillagerPose, delta: f32) -> Vec<LivingModelBox> {
        let mut nose = model_box(
            [24, 0],
            [-1.0, -1.0, -6.0],
            [2, 4, 2],
            delta,
            false,
            pose.nose,
            LivingModelGroup::Head,
        );
        nose.parentPose = Some(pose.head);
        vec![
            model_box(
                [0, 0],
                [-4.0, -10.0, -4.0],
                [8, 10, 8],
                delta,
                false,
                pose.head,
                LivingModelGroup::Head,
            ),
            nose,
            model_box(
                [16, 20],
                [-4.0, 0.0, -3.0],
                [8, 12, 6],
                delta,
                false,
                pose.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 38],
                [-4.0, 0.0, -3.0],
                [8, 18, 6],
                delta + 0.5,
                false,
                pose.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [44, 22],
                [-8.0, -2.0, -2.0],
                [4, 8, 4],
                delta,
                false,
                pose.arms,
                LivingModelGroup::Body,
            ),
            model_box(
                [44, 22],
                [4.0, -2.0, -2.0],
                [4, 8, 4],
                delta,
                true,
                pose.arms,
                LivingModelGroup::Body,
            ),
            model_box(
                [40, 38],
                [-4.0, 2.0, -2.0],
                [8, 4, 4],
                delta,
                false,
                pose.arms,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 22],
                [-2.0, 0.0, -2.0],
                [4, 12, 4],
                delta,
                false,
                pose.rightLeg,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 22],
                [-2.0, 0.0, -2.0],
                [4, 12, 4],
                delta,
                true,
                pose.leftLeg,
                LivingModelGroup::Body,
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
        LivingChildLayout, LivingRenderInput,
    };

    fn input() -> LivingRenderInput {
        LivingRenderInput {
            position: [0.0; 3],
            bodyYaw: 0.0,
            headYaw: 30.0,
            headPitch: 10.0,
            limbSwing: 1.0,
            limbSwingAmount: 0.5,
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
    fn villager_arms_remain_crossed_at_mcp_angle() {
        let pose = ModelVillager::pose(input());
        assert_eq!(pose.arms.pivot, [0.0, 3.0, -1.0]);
        assert!((pose.arms.rotation[0] + 0.75).abs() < 1.0e-6);
    }
}
