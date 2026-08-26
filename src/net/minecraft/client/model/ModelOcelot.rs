use crate::net::minecraft::client::entity::EntityOtherClient::EntityOtherClient;
use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingChildLayout, LivingModelBox, LivingModelGroup, LivingRenderInput,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcelotAnimationState {
    Sneaking,
    Normal,
    Sprinting,
    Sitting,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OcelotPose {
    pub head: PartPose,
    pub body: PartPose,
    pub tail: PartPose,
    pub tail2: PartPose,
    pub backLeft: PartPose,
    pub backRight: PartPose,
    pub frontLeft: PartPose,
    pub frontRight: PartPose,
    pub state: OcelotAnimationState,
}

pub struct ModelOcelot;
impl ModelOcelot {
    pub const CHILD_LAYOUT: LivingChildLayout = LivingChildLayout {
        headScale: 0.75,
        headTranslation: [0.0, 10.0 * 0.0625, 4.0 * 0.0625],
        bodyScale: 0.5,
        bodyTranslation: [0.0, 1.5, 0.0],
    };

    pub fn pose(input: LivingRenderInput, entity: &EntityOtherClient) -> OcelotPose {
        let state = if entity.entity.sneaking {
            OcelotAnimationState::Sneaking
        } else if entity.entitySprinting() {
            OcelotAnimationState::Sprinting
        } else if entity.tameableSitting() {
            OcelotAnimationState::Sitting
        } else {
            OcelotAnimationState::Normal
        };
        let mut body = PartPose {
            pivot: [0.0, 12.0, -10.0],
            rotation: [std::f32::consts::FRAC_PI_2, 0.0, 0.0],
        };
        let mut head = PartPose {
            pivot: [0.0, 15.0, -9.0],
            rotation: [
                input.headPitch.to_radians(),
                (input.headYaw - input.bodyYaw).to_radians(),
                0.0,
            ],
        };
        let mut tail = PartPose {
            pivot: [0.0, 15.0, 8.0],
            rotation: [0.9, 0.0, 0.0],
        };
        let mut tail2 = PartPose {
            pivot: [0.0, 20.0, 14.0],
            rotation: [0.0, 0.0, 0.0],
        };
        let mut frontLeft = PartPose {
            pivot: [1.2, 13.8, -5.0],
            rotation: [0.0; 3],
        };
        let mut frontRight = PartPose {
            pivot: [-1.2, 13.8, -5.0],
            rotation: [0.0; 3],
        };
        let mut backLeft = PartPose {
            pivot: [1.1, 18.0, 5.0],
            rotation: [0.0; 3],
        };
        let mut backRight = PartPose {
            pivot: [-1.1, 18.0, 5.0],
            rotation: [0.0; 3],
        };
        match state {
            OcelotAnimationState::Sneaking => {
                body.pivot[1] += 1.0;
                head.pivot[1] += 2.0;
                tail.pivot[1] += 1.0;
                tail2.pivot[1] -= 4.0;
                tail2.pivot[2] += 2.0;
                tail.rotation[0] = std::f32::consts::FRAC_PI_2;
                tail2.rotation[0] = std::f32::consts::FRAC_PI_2;
            }
            OcelotAnimationState::Sprinting => {
                tail2.pivot[1] = tail.pivot[1];
                tail2.pivot[2] += 2.0;
                tail.rotation[0] = std::f32::consts::FRAC_PI_2;
                tail2.rotation[0] = std::f32::consts::FRAC_PI_2;
            }
            OcelotAnimationState::Sitting => {
                body.rotation[0] = std::f32::consts::FRAC_PI_4;
                body.pivot[1] -= 4.0;
                body.pivot[2] += 5.0;
                head.pivot[1] -= 3.3;
                head.pivot[2] += 1.0;
                tail.pivot[1] += 8.0;
                tail.pivot[2] -= 2.0;
                tail2.pivot[1] += 2.0;
                tail2.pivot[2] -= 0.8;
                tail.rotation[0] = 1.7278761;
                tail2.rotation[0] = 2.670354;
                frontLeft.rotation[0] = -0.15707964;
                frontLeft.pivot[1] = 15.8;
                frontLeft.pivot[2] = -7.0;
                frontRight.rotation[0] = -0.15707964;
                frontRight.pivot[1] = 15.8;
                frontRight.pivot[2] = -7.0;
                backLeft.rotation[0] = -std::f32::consts::FRAC_PI_2;
                backLeft.pivot[1] = 21.0;
                backLeft.pivot[2] = 1.0;
                backRight.rotation[0] = -std::f32::consts::FRAC_PI_2;
                backRight.pivot[1] = 21.0;
                backRight.pivot[2] = 1.0;
            }
            OcelotAnimationState::Normal => {}
        }
        if state != OcelotAnimationState::Sitting {
            let swing = input.limbSwing;
            let amount = input.limbSwingAmount;
            if state == OcelotAnimationState::Sprinting {
                backLeft.rotation[0] = (swing * 0.6662).cos() * amount;
                backRight.rotation[0] = (swing * 0.6662 + 0.3).cos() * amount;
                frontLeft.rotation[0] =
                    (swing * 0.6662 + std::f32::consts::PI + 0.3).cos() * amount;
                frontRight.rotation[0] = (swing * 0.6662 + std::f32::consts::PI).cos() * amount;
                tail2.rotation[0] = 1.7278761 + std::f32::consts::PI / 10.0 * swing.cos() * amount;
            } else {
                backLeft.rotation[0] = (swing * 0.6662).cos() * amount;
                backRight.rotation[0] = (swing * 0.6662 + std::f32::consts::PI).cos() * amount;
                frontLeft.rotation[0] = (swing * 0.6662 + std::f32::consts::PI).cos() * amount;
                frontRight.rotation[0] = (swing * 0.6662).cos() * amount;
                let scale = if state == OcelotAnimationState::Normal {
                    std::f32::consts::FRAC_PI_4
                } else {
                    0.47123894
                };
                tail2.rotation[0] = 1.7278761 + scale * swing.cos() * amount;
            }
        }
        OcelotPose {
            head,
            body,
            tail,
            tail2,
            backLeft,
            backRight,
            frontLeft,
            frontRight,
            state,
        }
    }

    pub fn boxes(p: OcelotPose) -> Vec<LivingModelBox> {
        vec![
            model_box(
                [0, 0],
                [-2.5, -2.0, -3.0],
                [5, 4, 5],
                0.0,
                false,
                p.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [0, 24],
                [-1.5, 0.0, -4.0],
                [3, 2, 2],
                0.0,
                false,
                p.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [0, 10],
                [-2.0, -3.0, 0.0],
                [1, 1, 2],
                0.0,
                false,
                p.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [6, 10],
                [1.0, -3.0, 0.0],
                [1, 1, 2],
                0.0,
                false,
                p.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [20, 0],
                [-2.0, 3.0, -8.0],
                [4, 16, 6],
                0.0,
                false,
                p.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 15],
                [-0.5, 0.0, 0.0],
                [1, 8, 1],
                0.0,
                false,
                p.tail,
                LivingModelGroup::Body,
            ),
            model_box(
                [4, 15],
                [-0.5, 0.0, 0.0],
                [1, 8, 1],
                0.0,
                false,
                p.tail2,
                LivingModelGroup::Body,
            ),
            model_box(
                [8, 13],
                [-1.0, 0.0, 1.0],
                [2, 6, 2],
                0.0,
                false,
                p.backLeft,
                LivingModelGroup::Body,
            ),
            model_box(
                [8, 13],
                [-1.0, 0.0, 1.0],
                [2, 6, 2],
                0.0,
                false,
                p.backRight,
                LivingModelGroup::Body,
            ),
            model_box(
                [40, 0],
                [-1.0, 0.0, 0.0],
                [2, 10, 2],
                0.0,
                false,
                p.frontLeft,
                LivingModelGroup::Body,
            ),
            model_box(
                [40, 0],
                [-1.0, 0.0, 0.0],
                [2, 10, 2],
                0.0,
                false,
                p.frontRight,
                LivingModelGroup::Body,
            ),
        ]
    }
}
