use crate::net::minecraft::client::model::ModelBiped::{ArmPose, BipedPose, ModelBiped};
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{LivingModelBox, LivingModelGroup, LivingRenderInput};

pub struct ModelEnderman;

impl ModelEnderman {
    /// Direct port of MCP 1.12.2 `ModelEnderman#setRotationAngles` after the
    /// inherited `ModelBiped` animation pass.
    pub fn pose(input: LivingRenderInput, carrying: bool, attacking: bool) -> BipedPose {
        let mut pose = ModelBiped::setRotationAngles(
            input.limbSwing,
            input.limbSwingAmount,
            input.ageInTicks,
            input.headYaw - input.bodyYaw,
            input.headPitch,
            input.swingProgress,
            input.sneaking,
            false,
            false,
            false,
            ArmPose::Empty,
            ArmPose::Empty,
        );
        pose.body.rotation[0] = 0.0;
        pose.body.pivot = [0.0, -14.0, 0.0];
        for part in [&mut pose.rightArm, &mut pose.leftArm, &mut pose.rightLeg, &mut pose.leftLeg] {
            part.rotation[0] = (part.rotation[0] * 0.5).clamp(-0.4, 0.4);
        }
        if carrying {
            pose.rightArm.rotation[0] = -0.5;
            pose.leftArm.rotation[0] = -0.5;
            pose.rightArm.rotation[2] = 0.05;
            pose.leftArm.rotation[2] = -0.05;
        }
        // Replaced ModelRenderer constructor pivots. ModelBiped's attack-swing
        // branch overwrites arm X exactly, so retain that value only while a
        // swing is active; otherwise restore ModelEnderman's -3/+5 pivots.
        if input.swingProgress <= 0.0 {
            pose.rightArm.pivot[0] = -3.0;
            pose.leftArm.pivot[0] = 5.0;
        }
        pose.rightArm.pivot[1] = -12.0;
        pose.leftArm.pivot[1] = -12.0;
        pose.rightArm.pivot[2] = 0.0;
        pose.leftArm.pivot[2] = 0.0;
        pose.rightLeg.pivot = [-2.0, -5.0, 0.0];
        pose.leftLeg.pivot = [2.0, -5.0, 0.0];
        pose.head.pivot = [0.0, if attacking { -18.0 } else { -13.0 }, 0.0];
        pose
    }

    pub fn boxes(pose: BipedPose, scale: f32) -> Vec<LivingModelBox> {
        vec![
            model_box([0, 0], [-4.0, -8.0, -4.0], [8, 8, 8], scale, false, pose.head, LivingModelGroup::Head),
            // ModelEnderman constructs headwear with `scale - 0.5F`.
            model_box([0, 16], [-4.0, -8.0, -4.0], [8, 8, 8], scale - 0.5, false, pose.head, LivingModelGroup::Body),
            model_box([32, 16], [-4.0, 0.0, -2.0], [8, 12, 4], scale, false, pose.body, LivingModelGroup::Body),
            model_box([56, 0], [-1.0, -2.0, -1.0], [2, 30, 2], scale, false, pose.rightArm, LivingModelGroup::Body),
            model_box([56, 0], [-1.0, -2.0, -1.0], [2, 30, 2], scale, true, pose.leftArm, LivingModelGroup::Body),
            model_box([56, 0], [-1.0, 0.0, -1.0], [2, 30, 2], scale, false, pose.rightLeg, LivingModelGroup::Body),
            model_box([56, 0], [-1.0, 0.0, -1.0], [2, 30, 2], scale, true, pose.leftLeg, LivingModelGroup::Body),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::client::renderer::entity::RenderLivingBase::LivingChildLayout;
    fn input() -> LivingRenderInput { LivingRenderInput {
        position:[0.0;3], bodyYaw:0.0, headYaw:0.0, headPitch:0.0,
        limbSwing:0.0, limbSwingAmount:0.0, ageInTicks:0.0, swingProgress:0.0,
        sneaking:false, child:false, deathRotation:0.0, preScale:1.0,
        preScaleXYZ:[1.0;3], childLayout:LivingChildLayout::BIPED, adultTranslation:[0.0;3],
    }}
    #[test] fn attacking_lowers_head_five_model_pixels() {
        assert_eq!(ModelEnderman::pose(input(), false, false).head.pivot[1], -13.0);
        assert_eq!(ModelEnderman::pose(input(), false, true).head.pivot[1], -18.0);
    }
    #[test] fn carrying_forces_vanilla_arm_pose() {
        let p=ModelEnderman::pose(input(), true, false);
        assert_eq!(p.rightArm.rotation[0], -0.5);
        assert_eq!(p.leftArm.rotation[2], -0.05);
    }
}
