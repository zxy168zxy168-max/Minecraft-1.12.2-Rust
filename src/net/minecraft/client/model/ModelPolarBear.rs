use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelQuadruped::ModelQuadruped;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingChildLayout, LivingModelBox, LivingModelGroup, LivingRenderInput,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolarBearPose {
    pub head: PartPose,
    pub body: PartPose,
    pub legs: [PartPose; 4],
}
pub struct ModelPolarBear;
impl ModelPolarBear {
    pub const CHILD_LAYOUT: LivingChildLayout = LivingChildLayout {
        headScale: 0.6666667,
        headTranslation: [0.0, 16.0 * 0.0625, 4.0 * 0.0625],
        bodyScale: 0.5,
        bodyTranslation: [0.0, 1.5, 0.0],
    };
    pub fn pose(input: LivingRenderInput, standingScale: f32) -> PolarBearPose {
        let q = ModelQuadruped::pose(input, 12);
        let f1 = standingScale * standingScale;
        let f2 = 1.0 - f1;
        let body = PartPose {
            pivot: [-2.0, 9.0 * f2 + 11.0 * f1, 12.0],
            rotation: [
                std::f32::consts::FRAC_PI_2 - f1 * std::f32::consts::PI * 0.35,
                0.0,
                0.0,
            ],
        };
        let mut legs = [q.leg1, q.leg2, q.leg3, q.leg4];
        legs[0].pivot = [-4.5, 14.0, 6.0];
        legs[1].pivot = [4.5, 14.0, 6.0];
        legs[2].pivot = [-3.5, 14.0 * f2 - 6.0 * f1, -8.0 * f2 - 4.0 * f1];
        legs[3].pivot = [3.5, legs[2].pivot[1], legs[2].pivot[2]];
        legs[2].rotation[0] -= f1 * std::f32::consts::PI * 0.45;
        legs[3].rotation[0] -= f1 * std::f32::consts::PI * 0.45;
        let head = PartPose {
            pivot: [0.0, 10.0 * f2 - 12.0 * f1, -16.0 * f2 - 3.0 * f1],
            rotation: [
                input.headPitch.to_radians() + f1 * std::f32::consts::PI * 0.15,
                (input.headYaw - input.bodyYaw).to_radians(),
                0.0,
            ],
        };
        PolarBearPose { head, body, legs }
    }
    pub fn boxes(p: PolarBearPose) -> Vec<LivingModelBox> {
        vec![
            model_box(
                [0, 0],
                [-3.5, -3.0, -3.0],
                [7, 7, 7],
                0.0,
                false,
                p.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [0, 44],
                [-2.5, 1.0, -6.0],
                [5, 3, 3],
                0.0,
                false,
                p.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [26, 0],
                [-4.5, -4.0, -1.0],
                [2, 2, 1],
                0.0,
                false,
                p.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [26, 0],
                [2.5, -4.0, -1.0],
                [2, 2, 1],
                0.0,
                true,
                p.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [0, 19],
                [-5.0, -13.0, -7.0],
                [14, 14, 11],
                0.0,
                false,
                p.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [39, 0],
                [-4.0, -25.0, -7.0],
                [12, 12, 10],
                0.0,
                false,
                p.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [50, 22],
                [-2.0, 0.0, -2.0],
                [4, 10, 8],
                0.0,
                false,
                p.legs[0],
                LivingModelGroup::Body,
            ),
            model_box(
                [50, 22],
                [-2.0, 0.0, -2.0],
                [4, 10, 8],
                0.0,
                false,
                p.legs[1],
                LivingModelGroup::Body,
            ),
            model_box(
                [50, 40],
                [-2.0, 0.0, -2.0],
                [4, 10, 6],
                0.0,
                false,
                p.legs[2],
                LivingModelGroup::Body,
            ),
            model_box(
                [50, 40],
                [-2.0, 0.0, -2.0],
                [4, 10, 6],
                0.0,
                false,
                p.legs[3],
                LivingModelGroup::Body,
            ),
        ]
    }
}
