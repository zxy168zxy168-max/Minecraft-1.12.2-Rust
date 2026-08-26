use crate::net::minecraft::client::model::ModelZombie::{model_box, ModelZombie};
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup, LivingRenderInput,
};

pub struct ModelZombieVillager;
impl ModelZombieVillager {
    pub fn boxes(input: LivingRenderInput, armsRaised: bool, delta: f32) -> Vec<LivingModelBox> {
        let pose = ModelZombie::pose(input, armsRaised);
        vec![
            model_box(
                [0, 0],
                [-4.0, -10.0, -4.0],
                [8, 10, 8],
                delta,
                false,
                pose.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [24, 0],
                [-1.0, -3.0, -6.0],
                [2, 4, 2],
                delta,
                false,
                pose.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [16, 20],
                [-4.0, 0.0, -3.0],
                [8, 12, 6],
                delta,
                false,
                pose.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 38],
                [-4.0, 0.0, -3.0],
                [8, 18, 6],
                delta + 0.05,
                false,
                pose.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [44, 38],
                [-3.0, -2.0, -2.0],
                [4, 12, 4],
                delta,
                false,
                pose.rightArm,
                LivingModelGroup::Body,
            ),
            model_box(
                [44, 38],
                [-1.0, -2.0, -2.0],
                [4, 12, 4],
                delta,
                true,
                pose.leftArm,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 22],
                [-2.0, 0.0, -2.0],
                [4, 12, 4],
                delta,
                false,
                pose.rightLeg,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 22],
                [-2.0, 0.0, -2.0],
                [4, 12, 4],
                delta,
                true,
                pose.leftLeg,
                LivingModelGroup::Body,
            ),
        ]
    }
}
