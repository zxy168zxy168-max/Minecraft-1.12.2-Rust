use crate::net::minecraft::client::model::ModelQuadruped::QuadrupedPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup,
};

pub struct ModelSheep1;

impl ModelSheep1 {
    pub fn boxes(mut pose: QuadrupedPose) -> Vec<LivingModelBox> {
        pose.leg1.pivot = [-3.0, 12.0, 7.0];
        pose.leg2.pivot = [3.0, 12.0, 7.0];
        pose.leg3.pivot = [-3.0, 12.0, -5.0];
        pose.leg4.pivot = [3.0, 12.0, -5.0];
        vec![
            model_box(
                [0, 0],
                [-3.0, -4.0, -4.0],
                [6, 6, 6],
                0.6,
                false,
                pose.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [28, 8],
                [-4.0, -10.0, -7.0],
                [8, 16, 6],
                1.75,
                false,
                pose.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, 6, 4],
                0.5,
                false,
                pose.leg1,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, 6, 4],
                0.5,
                false,
                pose.leg2,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, 6, 4],
                0.5,
                false,
                pose.leg3,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, 6, 4],
                0.5,
                false,
                pose.leg4,
                LivingModelGroup::Body,
            ),
        ]
    }
}
