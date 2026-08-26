use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup,
};

pub struct ModelSlime;
impl ModelSlime {
    pub fn innerBoxes() -> Vec<LivingModelBox> {
        let pose = PartPose::default();
        vec![
            model_box(
                [0, 16],
                [-3.0, 17.0, -3.0],
                [6, 6, 6],
                0.0,
                false,
                pose,
                LivingModelGroup::Body,
            ),
            model_box(
                [32, 0],
                [-3.25, 18.0, -3.5],
                [2, 2, 2],
                0.0,
                false,
                pose,
                LivingModelGroup::Body,
            ),
            model_box(
                [32, 4],
                [1.25, 18.0, -3.5],
                [2, 2, 2],
                0.0,
                false,
                pose,
                LivingModelGroup::Body,
            ),
            model_box(
                [32, 8],
                [0.0, 21.0, -3.5],
                [1, 1, 1],
                0.0,
                false,
                pose,
                LivingModelGroup::Body,
            ),
        ]
    }
    pub fn gelBoxes() -> Vec<LivingModelBox> {
        vec![model_box(
            [0, 0],
            [-4.0, 16.0, -4.0],
            [8, 8, 8],
            0.0,
            false,
            PartPose::default(),
            LivingModelGroup::Body,
        )]
    }
}
