use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingChildLayout, LivingModelBox, LivingModelGroup, LivingRenderInput, RenderLivingBase,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChickenPose {
    pub head: PartPose,
    pub bill: PartPose,
    pub chin: PartPose,
    pub body: PartPose,
    pub rightLeg: PartPose,
    pub leftLeg: PartPose,
    pub rightWing: PartPose,
    pub leftWing: PartPose,
}

pub struct ModelChicken;

impl ModelChicken {
    pub fn input(input: LivingRenderInput) -> LivingRenderInput {
        RenderLivingBase::withChildLayout(input, LivingChildLayout::CHICKEN)
    }

    pub fn pose(input: LivingRenderInput, flap: f32) -> ChickenPose {
        let head = PartPose {
            pivot: [0.0, 15.0, -4.0],
            rotation: [
                input.headPitch.to_radians(),
                (input.headYaw - input.bodyYaw).to_radians(),
                0.0,
            ],
        };
        ChickenPose {
            head,
            bill: head,
            chin: head,
            body: PartPose {
                pivot: [0.0, 16.0, 0.0],
                rotation: [std::f32::consts::FRAC_PI_2, 0.0, 0.0],
            },
            rightLeg: PartPose {
                pivot: [-2.0, 19.0, 1.0],
                rotation: [
                    (input.limbSwing * 0.6662).cos() * 1.4 * input.limbSwingAmount,
                    0.0,
                    0.0,
                ],
            },
            leftLeg: PartPose {
                pivot: [1.0, 19.0, 1.0],
                rotation: [
                    (input.limbSwing * 0.6662 + std::f32::consts::PI).cos()
                        * 1.4
                        * input.limbSwingAmount,
                    0.0,
                    0.0,
                ],
            },
            rightWing: PartPose {
                pivot: [-4.0, 13.0, 0.0],
                rotation: [0.0, 0.0, flap],
            },
            leftWing: PartPose {
                pivot: [4.0, 13.0, 0.0],
                rotation: [0.0, 0.0, -flap],
            },
        }
    }

    pub fn boxes(pose: ChickenPose) -> Vec<LivingModelBox> {
        vec![
            model_box(
                [0, 0],
                [-2.0, -6.0, -2.0],
                [4, 6, 3],
                0.0,
                false,
                pose.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [14, 0],
                [-2.0, -4.0, -4.0],
                [4, 2, 2],
                0.0,
                false,
                pose.bill,
                LivingModelGroup::Head,
            ),
            model_box(
                [14, 4],
                [-1.0, -2.0, -3.0],
                [2, 2, 2],
                0.0,
                false,
                pose.chin,
                LivingModelGroup::Head,
            ),
            model_box(
                [0, 9],
                [-3.0, -4.0, -3.0],
                [6, 8, 6],
                0.0,
                false,
                pose.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [26, 0],
                [-1.0, 0.0, -3.0],
                [3, 5, 3],
                0.0,
                false,
                pose.rightLeg,
                LivingModelGroup::Body,
            ),
            model_box(
                [26, 0],
                [-1.0, 0.0, -3.0],
                [3, 5, 3],
                0.0,
                false,
                pose.leftLeg,
                LivingModelGroup::Body,
            ),
            model_box(
                [24, 13],
                [0.0, 0.0, -3.0],
                [1, 4, 6],
                0.0,
                false,
                pose.rightWing,
                LivingModelGroup::Body,
            ),
            model_box(
                [24, 13],
                [-1.0, 0.0, -3.0],
                [1, 4, 6],
                0.0,
                false,
                pose.leftWing,
                LivingModelGroup::Body,
            ),
        ]
    }
}
