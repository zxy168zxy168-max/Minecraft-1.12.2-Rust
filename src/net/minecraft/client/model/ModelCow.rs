use crate::net::minecraft::client::model::ModelQuadruped::{ModelQuadruped, QuadrupedPose};
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingChildLayout, LivingModelBox, LivingModelGroup, LivingRenderInput, RenderLivingBase,
};

pub struct ModelCow;

impl ModelCow {
    pub fn input(input: LivingRenderInput) -> LivingRenderInput {
        RenderLivingBase::withChildLayout(input, LivingChildLayout::quadruped(8.0, 6.0))
    }

    pub fn pose(input: LivingRenderInput) -> QuadrupedPose {
        let mut pose = ModelQuadruped::pose(input, 12);
        pose.head.pivot = [0.0, 4.0, -8.0];
        pose.body.pivot = [0.0, 5.0, 2.0];
        pose.leg1.pivot[0] -= 1.0;
        pose.leg2.pivot[0] += 1.0;
        pose.leg3.pivot[0] -= 1.0;
        pose.leg4.pivot[0] += 1.0;
        pose.leg3.pivot[2] -= 1.0;
        pose.leg4.pivot[2] -= 1.0;
        pose
    }

    pub fn boxes(pose: QuadrupedPose) -> Vec<LivingModelBox> {
        vec![
            model_box(
                [0, 0],
                [-4.0, -4.0, -6.0],
                [8, 8, 6],
                0.0,
                false,
                pose.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [22, 0],
                [-5.0, -5.0, -4.0],
                [1, 3, 1],
                0.0,
                false,
                pose.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [22, 0],
                [4.0, -5.0, -4.0],
                [1, 3, 1],
                0.0,
                false,
                pose.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [18, 4],
                [-6.0, -10.0, -7.0],
                [12, 18, 10],
                0.0,
                false,
                pose.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [52, 0],
                [-2.0, 2.0, -8.0],
                [4, 6, 1],
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
