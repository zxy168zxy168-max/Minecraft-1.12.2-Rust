use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup,
};

pub struct ModelMagmaCube;
impl ModelMagmaCube {
    pub fn boxes(squish: f32) -> Vec<LivingModelBox> {
        let mut out = Vec::with_capacity(9);
        out.push(model_box(
            [0, 16],
            [-2.0, 18.0, -2.0],
            [4, 4, 4],
            0.0,
            false,
            PartPose::default(),
            LivingModelGroup::Body,
        ));
        for i in 0..8 {
            let (u, v) = if i == 2 {
                (24, 10)
            } else if i == 3 {
                (24, 19)
            } else {
                (0, i as i32)
            };
            let pose = PartPose {
                pivot: [0.0, -((4 - i) as f32) * squish * 1.7, 0.0],
                rotation: [0.0; 3],
            };
            out.push(model_box(
                [u, v],
                [-4.0, (16 + i) as f32, -4.0],
                [8, 1, 8],
                0.0,
                false,
                pose,
                LivingModelGroup::Body,
            ));
        }
        out
    }
}
