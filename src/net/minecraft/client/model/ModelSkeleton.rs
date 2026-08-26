use crate::net::minecraft::client::model::ModelBiped::{ArmPose, BipedPose, ModelBiped};
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup, LivingRenderInput,
};

pub struct ModelSkeleton;

impl ModelSkeleton {
    pub fn pose(
        input: LivingRenderInput,
        swingingArms: bool,
        holdingBow: bool,
        primaryLeft: bool,
    ) -> BipedPose {
        let (leftPose, rightPose) = if swingingArms && holdingBow {
            if primaryLeft {
                (ArmPose::BowAndArrow, ArmPose::Empty)
            } else {
                (ArmPose::Empty, ArmPose::BowAndArrow)
            }
        } else {
            (ArmPose::Empty, ArmPose::Empty)
        };
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
            leftPose,
            rightPose,
        );
        if swingingArms && !holdingBow {
            let f = (input.swingProgress * std::f32::consts::PI).sin();
            let f1 = ((1.0 - (1.0 - input.swingProgress) * (1.0 - input.swingProgress))
                * std::f32::consts::PI)
                .sin();
            pose.rightArm.rotation[2] = 0.0;
            pose.leftArm.rotation[2] = 0.0;
            pose.rightArm.rotation[1] = -(0.1 - f * 0.6);
            pose.leftArm.rotation[1] = 0.1 - f * 0.6;
            pose.rightArm.rotation[0] = -std::f32::consts::FRAC_PI_2 - f * 1.2 + f1 * 0.4;
            pose.leftArm.rotation[0] = -std::f32::consts::FRAC_PI_2 - f * 1.2 + f1 * 0.4;
            pose.rightArm.rotation[2] += (input.ageInTicks * 0.09).cos() * 0.05 + 0.05;
            pose.leftArm.rotation[2] -= (input.ageInTicks * 0.09).cos() * 0.05 + 0.05;
            pose.rightArm.rotation[0] += (input.ageInTicks * 0.067).sin() * 0.05;
            pose.leftArm.rotation[0] -= (input.ageInTicks * 0.067).sin() * 0.05;
        }
        pose
    }

    pub fn boxes(pose: BipedPose, delta: f32, armorShape: bool) -> Vec<LivingModelBox> {
        if armorShape {
            return crate::net::minecraft::client::model::ModelZombie::standard_biped_boxes(
                pose, delta,
            );
        }
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
                [-1.0, -2.0, -1.0],
                [2, 12, 2],
                delta,
                false,
                crate::net::minecraft::client::model::ModelBiped::PartPose {
                    pivot: [-5.0, 2.0, 0.0],
                    ..pose.rightArm
                },
                LivingModelGroup::Body,
            ),
            model_box(
                [40, 16],
                [-1.0, -2.0, -1.0],
                [2, 12, 2],
                delta,
                true,
                crate::net::minecraft::client::model::ModelBiped::PartPose {
                    pivot: [5.0, 2.0, 0.0],
                    ..pose.leftArm
                },
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-1.0, 0.0, -1.0],
                [2, 12, 2],
                delta,
                false,
                crate::net::minecraft::client::model::ModelBiped::PartPose {
                    pivot: [-2.0, 12.0, 0.0],
                    ..pose.rightLeg
                },
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-1.0, 0.0, -1.0],
                [2, 12, 2],
                delta,
                true,
                crate::net::minecraft::client::model::ModelBiped::PartPose {
                    pivot: [2.0, 12.0, 0.0],
                    ..pose.leftLeg
                },
                LivingModelGroup::Body,
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_handed_bow_uses_left_arm_pose() {
        let input = LivingRenderInput {
            position: [0.0; 3], bodyYaw: 0.0, headYaw: 0.0, headPitch: 0.0,
            limbSwing: 0.0, limbSwingAmount: 0.0, ageInTicks: 0.0,
            swingProgress: 0.0, sneaking: false, child: false,
            deathRotation: 0.0, preScale: 1.0,
            preScaleXYZ: [1.0; 3],
            childLayout: crate::net::minecraft::client::renderer::entity::RenderLivingBase::LivingChildLayout::BIPED,
            adultTranslation: [0.0; 3],
        };
        let left = ModelSkeleton::pose(input, true, true, true);
        let right = ModelSkeleton::pose(input, true, true, false);
        assert!((left.rightArm.rotation[1] + 0.5).abs() < 1.0e-6);
        assert!((left.leftArm.rotation[1] - 0.1).abs() < 1.0e-6);
        assert!((right.rightArm.rotation[1] + 0.1).abs() < 1.0e-6);
        assert!((right.leftArm.rotation[1] - 0.5).abs() < 1.0e-6);
    }

    #[test]
    fn skeleton_base_limbs_are_two_pixels_wide() {
        let input = LivingRenderInput {
            position: [0.0; 3], bodyYaw: 0.0, headYaw: 0.0, headPitch: 0.0,
            limbSwing: 0.0, limbSwingAmount: 0.0, ageInTicks: 0.0,
            swingProgress: 0.0, sneaking: false, child: false,
            deathRotation: 0.0, preScale: 1.0,
            preScaleXYZ: [1.0; 3],
            childLayout: crate::net::minecraft::client::renderer::entity::RenderLivingBase::LivingChildLayout::BIPED,
            adultTranslation: [0.0; 3],
        };
        let boxes =
            ModelSkeleton::boxes(ModelSkeleton::pose(input, false, false, false), 0.0, false);
        assert_eq!(boxes[3].size, [2, 12, 2]);
        assert_eq!(boxes[5].size, [2, 12, 2]);
    }
}
