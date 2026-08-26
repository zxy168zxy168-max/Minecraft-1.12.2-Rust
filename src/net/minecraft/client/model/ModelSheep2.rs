use crate::net::minecraft::client::model::ModelQuadruped::{ModelQuadruped, QuadrupedPose};
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingChildLayout, LivingModelBox, LivingModelGroup, LivingRenderInput, RenderLivingBase,
};

pub struct ModelSheep2;

impl ModelSheep2 {
    pub fn input(input: LivingRenderInput) -> LivingRenderInput {
        RenderLivingBase::withChildLayout(input, LivingChildLayout::quadruped(8.0, 4.0))
    }

    pub fn pose(input: LivingRenderInput, headPointY: f32, headAngleX: f32) -> QuadrupedPose {
        let mut pose = ModelQuadruped::pose(input, 12);
        pose.head.pivot = [0.0, 6.0 + headPointY * 9.0, -8.0];
        pose.head.rotation[0] = headAngleX;
        pose.body.pivot = [0.0, 5.0, 2.0];
        pose
    }

    pub fn boxes(pose: QuadrupedPose) -> Vec<LivingModelBox> {
        vec![
            model_box(
                [0, 0],
                [-3.0, -4.0, -6.0],
                [6, 6, 8],
                0.0,
                false,
                pose.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [28, 8],
                [-4.0, -10.0, -7.0],
                [8, 16, 6],
                0.0,
                false,
                pose.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, 12, 4],
                0.0,
                false,
                pose.leg1,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, 12, 4],
                0.0,
                false,
                pose.leg2,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, 12, 4],
                0.0,
                false,
                pose.leg3,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, 12, 4],
                0.0,
                false,
                pose.leg4,
                LivingModelGroup::Body,
            ),
        ]
    }
}
