use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup, LivingRenderInput,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreeperPose {
    pub head: PartPose,
    pub body: PartPose,
    pub leg1: PartPose,
    pub leg2: PartPose,
    pub leg3: PartPose,
    pub leg4: PartPose,
}

pub struct ModelCreeper;

impl ModelCreeper {
    pub fn pose(input: LivingRenderInput) -> CreeperPose {
        let phase = input.limbSwing * 0.6662;
        CreeperPose {
            head: PartPose {
                pivot: [0.0, 6.0, 0.0],
                rotation: [
                    input.headPitch.to_radians(),
                    (input.headYaw - input.bodyYaw).to_radians(),
                    0.0,
                ],
            },
            body: PartPose {
                pivot: [0.0, 6.0, 0.0],
                rotation: [0.0; 3],
            },
            leg1: PartPose {
                pivot: [-2.0, 18.0, 4.0],
                rotation: [phase.cos() * 1.4 * input.limbSwingAmount, 0.0, 0.0],
            },
            leg2: PartPose {
                pivot: [2.0, 18.0, 4.0],
                rotation: [
                    (phase + std::f32::consts::PI).cos() * 1.4 * input.limbSwingAmount,
                    0.0,
                    0.0,
                ],
            },
            leg3: PartPose {
                pivot: [-2.0, 18.0, -4.0],
                rotation: [
                    (phase + std::f32::consts::PI).cos() * 1.4 * input.limbSwingAmount,
                    0.0,
                    0.0,
                ],
            },
            leg4: PartPose {
                pivot: [2.0, 18.0, -4.0],
                rotation: [phase.cos() * 1.4 * input.limbSwingAmount, 0.0, 0.0],
            },
        }
    }

    pub fn boxes(pose: CreeperPose, delta: f32) -> Vec<LivingModelBox> {
        vec![
            model_box(
                [0, 0],
                [-4.0, -8.0, -4.0],
                [8, 8, 8],
                delta,
                false,
                pose.head,
                LivingModelGroup::Body,
            ),
            model_box(
                [16, 16],
                [-4.0, 0.0, -2.0],
                [8, 12, 4],
                delta,
                false,
                pose.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, 6, 4],
                delta,
                false,
                pose.leg1,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, 6, 4],
                delta,
                false,
                pose.leg2,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, 6, 4],
                delta,
                false,
                pose.leg3,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, 6, 4],
                delta,
                false,
                pose.leg4,
                LivingModelGroup::Body,
            ),
        ]
    }
}
