use crate::net::minecraft::client::model::ModelQuadruped::{ModelQuadruped, QuadrupedPose};
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingChildLayout, LivingModelBox, LivingModelGroup, LivingRenderInput,
};

pub struct ModelPig;

impl ModelPig {
    pub fn input(input: LivingRenderInput) -> LivingRenderInput {
        crate::net::minecraft::client::renderer::entity::RenderLivingBase::RenderLivingBase::withChildLayout(
            input,
            LivingChildLayout::quadruped(4.0, 4.0),
        )
    }

    pub fn pose(input: LivingRenderInput) -> QuadrupedPose {
        ModelQuadruped::pose(input, 6)
    }

    pub fn boxes(pose: QuadrupedPose, scale: f32) -> Vec<LivingModelBox> {
        let mut boxes = ModelQuadruped::boxes(pose, 6, scale);
        boxes.push(model_box(
            [16, 16],
            [-2.0, 0.0, -9.0],
            [4, 3, 1],
            scale,
            false,
            pose.head,
            LivingModelGroup::Head,
        ));
        boxes
    }
}
