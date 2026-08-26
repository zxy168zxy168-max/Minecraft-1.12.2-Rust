use crate::net::minecraft::client::model::ModelArmorStandArmor::ModelArmorStandArmor;
use crate::net::minecraft::client::model::ModelBiped::{BipedPose, PartPose};
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup, LivingRenderInput,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmorStandPose {
    pub biped: BipedPose,
    pub standRightSide: PartPose,
    pub standLeftSide: PartPose,
    pub standWaist: PartPose,
    pub standBase: PartPose,
    pub showArms: bool,
    pub showBase: bool,
    pub marker: bool,
}

pub struct ModelArmorStand;

impl ModelArmorStand {
    #[allow(clippy::too_many_arguments)]
    pub fn pose(
        input: LivingRenderInput,
        head: [f32; 3],
        body: [f32; 3],
        leftArm: [f32; 3],
        rightArm: [f32; 3],
        leftLeg: [f32; 3],
        rightLeg: [f32; 3],
        showArms: bool,
        noBasePlate: bool,
        marker: bool,
    ) -> ArmorStandPose {
        let biped = ModelArmorStandArmor::pose(head, body, leftArm, rightArm, leftLeg, rightLeg);
        let supportPose = PartPose {
            pivot: [0.0, 0.0, 0.0],
            rotation: biped.body.rotation,
        };
        ArmorStandPose {
            biped,
            standRightSide: supportPose,
            standLeftSide: supportPose,
            standWaist: supportPose,
            standBase: PartPose {
                pivot: [0.0, 12.0, 0.0],
                rotation: [0.0, -input.bodyYaw.to_radians(), 0.0],
            },
            showArms,
            showBase: !noBasePlate,
            marker,
        }
    }

    pub fn boxes(pose: ArmorStandPose, delta: f32) -> Vec<LivingModelBox> {
        let mut boxes = vec![
            model_box(
                [0, 0],
                [-1.0, -7.0, -1.0],
                [2, 7, 2],
                delta,
                false,
                pose.biped.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [0, 26],
                [-6.0, 0.0, -1.5],
                [12, 3, 3],
                delta,
                false,
                pose.biped.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [8, 0],
                [-1.0, 0.0, -1.0],
                [2, 11, 2],
                delta,
                false,
                pose.biped.rightLeg,
                LivingModelGroup::Body,
            ),
            model_box(
                [40, 16],
                [-1.0, 0.0, -1.0],
                [2, 11, 2],
                delta,
                true,
                pose.biped.leftLeg,
                LivingModelGroup::Body,
            ),
            model_box(
                [16, 0],
                [-3.0, 3.0, -1.0],
                [2, 7, 2],
                delta,
                false,
                pose.standRightSide,
                LivingModelGroup::Body,
            ),
            model_box(
                [48, 16],
                [1.0, 3.0, -1.0],
                [2, 7, 2],
                delta,
                false,
                pose.standLeftSide,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 48],
                [-4.0, 10.0, -1.0],
                [8, 2, 2],
                delta,
                false,
                pose.standWaist,
                LivingModelGroup::Body,
            ),
        ];
        if pose.showArms {
            boxes.push(model_box(
                [24, 0],
                [-2.0, -2.0, -1.0],
                [2, 12, 2],
                delta,
                false,
                pose.biped.rightArm,
                LivingModelGroup::Body,
            ));
            boxes.push(model_box(
                [32, 16],
                [0.0, -2.0, -1.0],
                [2, 12, 2],
                delta,
                true,
                pose.biped.leftArm,
                LivingModelGroup::Body,
            ));
        }
        if pose.showBase {
            boxes.push(model_box(
                [0, 32],
                [-6.0, 11.0, -6.0],
                [12, 1, 12],
                delta,
                false,
                pose.standBase,
                LivingModelGroup::Body,
            ));
        }
        boxes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> LivingRenderInput {
        LivingRenderInput {
            position: [0.0; 3], bodyYaw: 90.0, headYaw: 90.0, headPitch: 0.0,
            limbSwing: 0.0, limbSwingAmount: 0.0, ageInTicks: 0.0,
            swingProgress: 0.0, sneaking: false, child: false,
            deathRotation: 0.0, preScale: 1.0,
            preScaleXYZ: [1.0; 3],
            childLayout: crate::net::minecraft::client::renderer::entity::RenderLivingBase::LivingChildLayout::BIPED,
            adultTranslation: [0.0; 3],
        }
    }

    #[test]
    fn marker_does_not_hide_the_visible_model() {
        let pose = ModelArmorStand::pose(
            input(),
            [0.0; 3],
            [0.0; 3],
            [-10.0, 0.0, -10.0],
            [-15.0, 0.0, 10.0],
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, 1.0],
            false,
            false,
            true,
        );
        assert!(!ModelArmorStand::boxes(pose, 0.0).is_empty());
    }

    #[test]
    fn status_controls_arms_and_base_geometry() {
        let pose = ModelArmorStand::pose(
            input(),
            [0.0; 3],
            [0.0; 3],
            [-10.0, 0.0, -10.0],
            [-15.0, 0.0, 10.0],
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, 1.0],
            false,
            true,
            false,
        );
        let boxes = ModelArmorStand::boxes(pose, 0.0);
        assert_eq!(boxes.len(), 7);
    }
}
