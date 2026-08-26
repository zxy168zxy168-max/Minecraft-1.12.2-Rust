use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup, LivingRenderInput,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpiderPose {
    pub head: PartPose,
    pub neck: PartPose,
    pub body: PartPose,
    pub legs: [PartPose; 8],
}

pub struct ModelSpider;

impl ModelSpider {
    pub fn pose(input: LivingRenderInput) -> SpiderPose {
        let mut legs = [PartPose::default(); 8];
        let pivots = [
            [-4.0, 15.0, 2.0],
            [4.0, 15.0, 2.0],
            [-4.0, 15.0, 1.0],
            [4.0, 15.0, 1.0],
            [-4.0, 15.0, 0.0],
            [4.0, 15.0, 0.0],
            [-4.0, 15.0, -1.0],
            [4.0, 15.0, -1.0],
        ];
        let baseZ = [
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_4,
            -0.58119464,
            0.58119464,
            -0.58119464,
            0.58119464,
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_4,
        ];
        let baseY = [
            std::f32::consts::FRAC_PI_4,
            -std::f32::consts::FRAC_PI_4,
            0.3926991,
            -0.3926991,
            -0.3926991,
            0.3926991,
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_4,
        ];
        let swing = input.limbSwing;
        let amount = input.limbSwingAmount;
        let ys = [
            -(swing * 1.3324).cos() * 0.4 * amount,
            -((swing * 1.3324 + std::f32::consts::PI).cos()) * 0.4 * amount,
            -((swing * 1.3324 + std::f32::consts::FRAC_PI_2).cos()) * 0.4 * amount,
            -((swing * 1.3324 + std::f32::consts::PI * 1.5).cos()) * 0.4 * amount,
        ];
        let zs = [
            ((swing * 0.6662).sin() * 0.4).abs() * amount,
            ((swing * 0.6662 + std::f32::consts::PI).sin() * 0.4).abs() * amount,
            ((swing * 0.6662 + std::f32::consts::FRAC_PI_2).sin() * 0.4).abs() * amount,
            ((swing * 0.6662 + std::f32::consts::PI * 1.5).sin() * 0.4).abs() * amount,
        ];
        for i in 0..8 {
            let pair = i / 2;
            let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            legs[i] = PartPose {
                pivot: pivots[i],
                rotation: [0.0, baseY[i] + ys[pair] * sign, baseZ[i] + zs[pair] * sign],
            };
        }
        SpiderPose {
            head: PartPose {
                pivot: [0.0, 15.0, -3.0],
                rotation: [
                    input.headPitch.to_radians(),
                    (input.headYaw - input.bodyYaw).to_radians(),
                    0.0,
                ],
            },
            neck: PartPose {
                pivot: [0.0, 15.0, 0.0],
                rotation: [0.0; 3],
            },
            body: PartPose {
                pivot: [0.0, 15.0, 9.0],
                rotation: [0.0; 3],
            },
            legs,
        }
    }

    pub fn boxes(pose: SpiderPose) -> Vec<LivingModelBox> {
        let mut boxes = vec![
            model_box(
                [32, 4],
                [-4.0, -4.0, -8.0],
                [8, 8, 8],
                0.0,
                false,
                pose.head,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 0],
                [-3.0, -3.0, -3.0],
                [6, 6, 6],
                0.0,
                false,
                pose.neck,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 12],
                [-5.0, -4.0, -6.0],
                [10, 8, 12],
                0.0,
                false,
                pose.body,
                LivingModelGroup::Body,
            ),
        ];
        for i in 0..8 {
            let (origin, mirror) = if i % 2 == 0 {
                ([-15.0, -1.0, -1.0], false)
            } else {
                ([-1.0, -1.0, -1.0], false)
            };
            boxes.push(model_box(
                [18, 0],
                origin,
                [16, 2, 2],
                0.0,
                mirror,
                pose.legs[i],
                LivingModelGroup::Body,
            ));
        }
        boxes
    }
}
