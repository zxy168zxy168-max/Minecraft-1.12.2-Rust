use crate::net::minecraft::client::entity::EntityOtherClient::EntityOtherClient;
use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingChildLayout, LivingModelBox, LivingModelGroup, LivingRenderInput,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WolfPose {
    pub head: PartPose,
    pub body: PartPose,
    pub mane: PartPose,
    pub legs: [PartPose; 4],
    pub tail: PartPose,
}

/// Exact box geometry and living-animation transforms from MCP 1.12.2
/// `ModelWolf`. Parent-less Rust boxes share the same source ModelRenderer
/// pose where Java adds ears and muzzle to `wolfHeadMain`.
pub struct ModelWolf;

impl ModelWolf {
    pub const CHILD_LAYOUT: LivingChildLayout = LivingChildLayout {
        headScale: 1.0,
        headTranslation: [0.0, 5.0 * 0.0625, 2.0 * 0.0625],
        bodyScale: 0.5,
        bodyTranslation: [0.0, 1.5, 0.0],
    };

    pub fn pose(
        input: LivingRenderInput,
        entity: &EntityOtherClient,
        partialTicks: f32,
    ) -> WolfPose {
        let headRoll =
            entity.wolfInterestedAngle(partialTicks) + entity.wolfShakeAngle(partialTicks, 0.0);
        let bodyRoll = entity.wolfShakeAngle(partialTicks, -0.16);
        let maneRoll = entity.wolfShakeAngle(partialTicks, -0.08);
        let tailRoll = entity.wolfShakeAngle(partialTicks, -0.2);
        let mut legs = [PartPose::default(); 4];
        let (body, mane, tail);
        if entity.tameableSitting() {
            mane = PartPose {
                pivot: [-1.0, 16.0, -3.0],
                rotation: [std::f32::consts::PI * 2.0 / 5.0, 0.0, maneRoll],
            };
            body = PartPose {
                pivot: [0.0, 18.0, 0.0],
                rotation: [std::f32::consts::FRAC_PI_4, 0.0, bodyRoll],
            };
            tail = PartPose {
                pivot: [-1.0, 21.0, 6.0],
                rotation: [entity.wolfTailRotation(), 0.0, tailRoll],
            };
            legs[0] = PartPose {
                pivot: [-2.5, 22.0, 2.0],
                rotation: [std::f32::consts::PI * 1.5, 0.0, 0.0],
            };
            legs[1] = PartPose {
                pivot: [0.5, 22.0, 2.0],
                rotation: [std::f32::consts::PI * 1.5, 0.0, 0.0],
            };
            legs[2] = PartPose {
                pivot: [-2.49, 17.0, -4.0],
                rotation: [5.811947, 0.0, 0.0],
            };
            legs[3] = PartPose {
                pivot: [0.51, 17.0, -4.0],
                rotation: [5.811947, 0.0, 0.0],
            };
        } else {
            body = PartPose {
                pivot: [0.0, 14.0, 2.0],
                rotation: [std::f32::consts::FRAC_PI_2, 0.0, bodyRoll],
            };
            mane = PartPose {
                pivot: [-1.0, 14.0, -3.0],
                rotation: [std::f32::consts::FRAC_PI_2, 0.0, maneRoll],
            };
            tail = PartPose {
                pivot: [-1.0, 12.0, 8.0],
                rotation: [
                    entity.wolfTailRotation(),
                    if entity.wolfAngry() {
                        0.0
                    } else {
                        (input.limbSwing * 0.6662).cos() * 1.4 * input.limbSwingAmount
                    },
                    tailRoll,
                ],
            };
            let phase = input.limbSwing * 0.6662;
            legs[0] = PartPose {
                pivot: [-2.5, 16.0, 7.0],
                rotation: [phase.cos() * 1.4 * input.limbSwingAmount, 0.0, 0.0],
            };
            legs[1] = PartPose {
                pivot: [0.5, 16.0, 7.0],
                rotation: [
                    (phase + std::f32::consts::PI).cos() * 1.4 * input.limbSwingAmount,
                    0.0,
                    0.0,
                ],
            };
            legs[2] = PartPose {
                pivot: [-2.5, 16.0, -4.0],
                rotation: [
                    (phase + std::f32::consts::PI).cos() * 1.4 * input.limbSwingAmount,
                    0.0,
                    0.0,
                ],
            };
            legs[3] = PartPose {
                pivot: [0.5, 16.0, -4.0],
                rotation: [phase.cos() * 1.4 * input.limbSwingAmount, 0.0, 0.0],
            };
        }
        WolfPose {
            head: PartPose {
                pivot: [-1.0, 13.5, -7.0],
                rotation: [
                    input.headPitch.to_radians(),
                    (input.headYaw - input.bodyYaw).to_radians(),
                    headRoll,
                ],
            },
            body,
            mane,
            legs,
            tail,
        }
    }

    pub fn boxes(pose: WolfPose, delta: f32) -> Vec<LivingModelBox> {
        let mut boxes = vec![
            model_box(
                [0, 0],
                [-2.0, -3.0, -2.0],
                [6, 6, 4],
                delta,
                false,
                pose.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [16, 14],
                [-2.0, -5.0, 0.0],
                [2, 2, 1],
                delta,
                false,
                pose.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [16, 14],
                [2.0, -5.0, 0.0],
                [2, 2, 1],
                delta,
                false,
                pose.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [0, 10],
                [-0.5, 0.0, -5.0],
                [3, 3, 4],
                delta,
                false,
                pose.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [18, 14],
                [-3.0, -2.0, -3.0],
                [6, 9, 6],
                delta,
                false,
                pose.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [21, 0],
                [-3.0, -3.0, -3.0],
                [8, 6, 7],
                delta,
                false,
                pose.mane,
                LivingModelGroup::Body,
            ),
            model_box(
                [9, 18],
                [0.0, 0.0, -1.0],
                [2, 8, 2],
                delta,
                false,
                pose.tail,
                LivingModelGroup::Body,
            ),
        ];
        for leg in pose.legs {
            boxes.push(model_box(
                [0, 18],
                [0.0, 0.0, -1.0],
                [2, 8, 2],
                delta,
                false,
                leg,
                LivingModelGroup::Body,
            ));
        }
        boxes
    }
}
