use crate::net::minecraft::client::model::ModelBiped::{ArmPose, BipedPose, ModelBiped};
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup, LivingRenderInput,
};

pub struct ModelZombie;

impl ModelZombie {
    pub fn pose(input: LivingRenderInput, armsRaised: bool) -> BipedPose {
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
        let f = (input.swingProgress * std::f32::consts::PI).sin();
        let f1 = ((1.0 - (1.0 - input.swingProgress) * (1.0 - input.swingProgress))
            * std::f32::consts::PI)
            .sin();
        pose.rightArm.rotation[2] = 0.0;
        pose.leftArm.rotation[2] = 0.0;
        pose.rightArm.rotation[1] = -(0.1 - f * 0.6);
        pose.leftArm.rotation[1] = 0.1 - f * 0.6;
        let base = -std::f32::consts::PI / if armsRaised { 1.5 } else { 2.25 };
        pose.rightArm.rotation[0] = base + f * 1.2 - f1 * 0.4;
        pose.leftArm.rotation[0] = base + f * 1.2 - f1 * 0.4;
        pose.rightArm.rotation[2] += (input.ageInTicks * 0.09).cos() * 0.05 + 0.05;
        pose.leftArm.rotation[2] -= (input.ageInTicks * 0.09).cos() * 0.05 + 0.05;
        pose.rightArm.rotation[0] += (input.ageInTicks * 0.067).sin() * 0.05;
        pose.leftArm.rotation[0] -= (input.ageInTicks * 0.067).sin() * 0.05;
        pose
    }

    pub fn boxes(pose: BipedPose, delta: f32) -> Vec<LivingModelBox> {
        standard_biped_boxes(pose, delta)
    }
}

pub(crate) fn standard_biped_boxes(pose: BipedPose, delta: f32) -> Vec<LivingModelBox> {
    vec![
        model_box(
            [0, 0],
            [-4.0, -8.0, -4.0],
            [8, 8, 8],
            delta,
            false,
            pose.head,
            LivingModelGroup::Head,
        ),
        model_box(
            [32, 0],
            [-4.0, -8.0, -4.0],
            [8, 8, 8],
            delta + 0.5,
            false,
            pose.head,
            LivingModelGroup::Body,
        ),
        model_box(
            [16, 16],
            [-4.0, 0.0, -2.0],
            [8, 12, 4],
            delta,
            false,
            pose.body,
            LivingModelGroup::Body,
        ),
        model_box(
            [40, 16],
            [-3.0, -2.0, -2.0],
            [4, 12, 4],
            delta,
            false,
            pose.rightArm,
            LivingModelGroup::Body,
        ),
        model_box(
            [40, 16],
            [-1.0, -2.0, -2.0],
            [4, 12, 4],
            delta,
            true,
            pose.leftArm,
            LivingModelGroup::Body,
        ),
        model_box(
            [0, 16],
            [-2.0, 0.0, -2.0],
            [4, 12, 4],
            delta,
            false,
            pose.rightLeg,
            LivingModelGroup::Body,
        ),
        model_box(
            [0, 16],
            [-2.0, 0.0, -2.0],
            [4, 12, 4],
            delta,
            true,
            pose.leftLeg,
            LivingModelGroup::Body,
        ),
    ]
}

pub(crate) const fn model_box(
    texture: [i32; 2],
    origin: [f32; 3],
    size: [i32; 3],
    delta: f32,
    mirror: bool,
    pose: crate::net::minecraft::client::model::ModelBiped::PartPose,
    group: LivingModelGroup,
) -> LivingModelBox {
    LivingModelBox {
        texture,
        origin,
        size,
        delta,
        mirror,
        pose,
        group,
        parentPose: None,
        parentPose2: None,
        parentPose3: None,
        parentPose4: None,
        poseOffset: [0.0; 3],
        parentOffset: [0.0; 3],
        parentOffset2: [0.0; 3],
        parentOffset3: [0.0; 3],
        parentOffset4: [0.0; 3],
        childTransform: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raised_zombie_arms_use_mcp_one_point_five_divisor() {
        let input = LivingRenderInput {
            position: [0.0; 3], bodyYaw: 0.0, headYaw: 0.0, headPitch: 0.0,
            limbSwing: 0.0, limbSwingAmount: 0.0, ageInTicks: 0.0,
            swingProgress: 0.0, sneaking: false, child: false,
            deathRotation: 0.0, preScale: 1.0,
            preScaleXYZ: [1.0; 3],
            childLayout: crate::net::minecraft::client::renderer::entity::RenderLivingBase::LivingChildLayout::BIPED,
            adultTranslation: [0.0; 3],
        };
        let pose = ModelZombie::pose(input, true);
        assert!((pose.rightArm.rotation[0] + std::f32::consts::PI / 1.5).abs() < 1.0e-6);
    }
}
