use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingChildLayout, LivingModelBox, LivingModelGroup, LivingRenderInput,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RabbitPose {
    pub leftFoot: PartPose,
    pub rightFoot: PartPose,
    pub leftThigh: PartPose,
    pub rightThigh: PartPose,
    pub body: PartPose,
    pub leftArm: PartPose,
    pub rightArm: PartPose,
    pub head: PartPose,
    pub rightEar: PartPose,
    pub leftEar: PartPose,
    pub tail: PartPose,
    pub nose: PartPose,
}

pub struct ModelRabbit;
impl ModelRabbit {
    pub const CHILD_LAYOUT: LivingChildLayout = LivingChildLayout {
        headScale: 0.56666666,
        headTranslation: [0.0, 22.0 * 0.0625, 2.0 * 0.0625],
        bodyScale: 0.4,
        bodyTranslation: [0.0, 36.0 * 0.0625, 0.0],
    };

    pub fn pose(input: LivingRenderInput, jumpCompletion: f32) -> RabbitPose {
        let jump = (jumpCompletion * std::f32::consts::PI).sin();
        let pitch = input.headPitch.to_radians();
        let yaw = (input.headYaw - input.bodyYaw).to_radians();
        RabbitPose {
            leftFoot: PartPose {
                pivot: [3.0, 17.5, 3.7],
                rotation: [jump * 50.0_f32.to_radians(), 0.0, 0.0],
            },
            rightFoot: PartPose {
                pivot: [-3.0, 17.5, 3.7],
                rotation: [jump * 50.0_f32.to_radians(), 0.0, 0.0],
            },
            leftThigh: PartPose {
                pivot: [3.0, 17.5, 3.7],
                rotation: [(jump * 50.0 - 21.0).to_radians(), 0.0, 0.0],
            },
            rightThigh: PartPose {
                pivot: [-3.0, 17.5, 3.7],
                rotation: [(jump * 50.0 - 21.0).to_radians(), 0.0, 0.0],
            },
            body: PartPose {
                pivot: [0.0, 19.0, 8.0],
                rotation: [-0.34906584, 0.0, 0.0],
            },
            leftArm: PartPose {
                pivot: [3.0, 17.0, -1.0],
                rotation: [(jump * -40.0 - 11.0).to_radians(), 0.0, 0.0],
            },
            rightArm: PartPose {
                pivot: [-3.0, 17.0, -1.0],
                rotation: [(jump * -40.0 - 11.0).to_radians(), 0.0, 0.0],
            },
            head: PartPose {
                pivot: [0.0, 16.0, -1.0],
                rotation: [pitch, yaw, 0.0],
            },
            rightEar: PartPose {
                pivot: [0.0, 16.0, -1.0],
                rotation: [pitch, yaw - 0.2617994, 0.0],
            },
            leftEar: PartPose {
                pivot: [0.0, 16.0, -1.0],
                rotation: [pitch, yaw + 0.2617994, 0.0],
            },
            tail: PartPose {
                pivot: [0.0, 20.0, 7.0],
                rotation: [-0.3490659, 0.0, 0.0],
            },
            nose: PartPose {
                pivot: [0.0, 16.0, -1.0],
                rotation: [pitch, yaw, 0.0],
            },
        }
    }

    pub fn boxes(p: RabbitPose) -> Vec<LivingModelBox> {
        vec![
            model_box(
                [26, 24],
                [-1.0, 5.5, -3.7],
                [2, 1, 7],
                0.0,
                true,
                p.leftFoot,
                LivingModelGroup::Body,
            ),
            model_box(
                [8, 24],
                [-1.0, 5.5, -3.7],
                [2, 1, 7],
                0.0,
                true,
                p.rightFoot,
                LivingModelGroup::Body,
            ),
            model_box(
                [30, 15],
                [-1.0, 0.0, 0.0],
                [2, 4, 5],
                0.0,
                true,
                p.leftThigh,
                LivingModelGroup::Body,
            ),
            model_box(
                [16, 15],
                [-1.0, 0.0, 0.0],
                [2, 4, 5],
                0.0,
                true,
                p.rightThigh,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 0],
                [-3.0, -2.0, -10.0],
                [6, 5, 10],
                0.0,
                true,
                p.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [8, 15],
                [-1.0, 0.0, -1.0],
                [2, 7, 2],
                0.0,
                true,
                p.leftArm,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 15],
                [-1.0, 0.0, -1.0],
                [2, 7, 2],
                0.0,
                true,
                p.rightArm,
                LivingModelGroup::Body,
            ),
            model_box(
                [32, 0],
                [-2.5, -4.0, -5.0],
                [5, 4, 5],
                0.0,
                true,
                p.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [52, 0],
                [-2.5, -9.0, -1.0],
                [2, 5, 1],
                0.0,
                true,
                p.rightEar,
                LivingModelGroup::Head,
            ),
            model_box(
                [58, 0],
                [0.5, -9.0, -1.0],
                [2, 5, 1],
                0.0,
                true,
                p.leftEar,
                LivingModelGroup::Head,
            ),
            model_box(
                [52, 6],
                [-1.5, -1.5, 0.0],
                [3, 3, 2],
                0.0,
                true,
                p.tail,
                LivingModelGroup::Body,
            ),
            model_box(
                [32, 9],
                [-0.5, -2.5, -5.5],
                [1, 1, 1],
                0.0,
                true,
                p.nose,
                LivingModelGroup::Head,
            ),
        ]
    }
}
