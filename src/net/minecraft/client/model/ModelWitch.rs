use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelVillager::{ModelVillager, VillagerPose};
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup, LivingRenderInput,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WitchPose {
    pub villager: VillagerPose,
    pub noseOffset: [f32; 3],
    pub hatBase: PartPose,
    pub hatMiddle: PartPose,
    pub hatUpper: PartPose,
    pub hatTip: PartPose,
}

/// MCP 1.12.2 `ModelWitch`. The nested hat hierarchy is retained instead of
/// being flattened into a static approximation.
pub struct ModelWitch;

impl ModelWitch {
    pub fn pose(input: LivingRenderInput, entityId: i32, holdingItem: bool) -> WitchPose {
        let mut villager = ModelVillager::pose(input);
        let phase = 0.01 * (entityId.rem_euclid(10) as f32);
        villager.nose.rotation[0] = (input.ageInTicks * phase).sin() * 4.5_f32.to_radians();
        villager.nose.rotation[2] = (input.ageInTicks * phase).cos() * 2.5_f32.to_radians();
        let noseOffset = if holdingItem {
            villager.nose.rotation[0] = -0.9;
            // ModelRenderer offsetY=0.1875 and offsetZ=-0.09375 are outer
            // translations. Store their exact 1/16-model-pixel equivalents.
            [0.0, 3.0, -1.5]
        } else {
            [0.0; 3]
        };
        WitchPose {
            villager,
            noseOffset,
            hatBase: PartPose {
                pivot: [-5.0, -10.03125, -5.0],
                rotation: [0.0; 3],
            },
            hatMiddle: PartPose {
                pivot: [1.75, -4.0, 2.0],
                rotation: [-0.05235988, 0.0, 0.02617994],
            },
            hatUpper: PartPose {
                pivot: [1.75, -4.0, 2.0],
                rotation: [-0.10471976, 0.0, 0.05235988],
            },
            hatTip: PartPose {
                pivot: [1.75, -2.0, 2.0],
                rotation: [-0.20943952, 0.0, 0.10471976],
            },
        }
    }

    pub fn boxes(pose: WitchPose) -> Vec<LivingModelBox> {
        let mut boxes = ModelVillager::boxes(pose.villager, 0.0);
        // The villager nose is the second box. ModelRenderer offset is applied
        // after the nose pivot/rotation and before the parent head transform.
        boxes[1].poseOffset = pose.noseOffset;
        // Mole is a child of the nose, which itself is a child of the head.
        let mut mole = model_box(
            [0, 0],
            [0.0, 3.0, -6.75],
            [1, 1, 1],
            -0.25,
            false,
            PartPose {
                pivot: [0.0, -2.0, 0.0],
                rotation: [0.0; 3],
            },
            LivingModelGroup::Head,
        );
        mole.parentPose = Some(pose.villager.nose);
        mole.parentOffset = pose.noseOffset;
        mole.parentPose2 = Some(pose.villager.head);
        boxes.push(mole);

        let mut hat0 = model_box(
            [0, 64],
            [0.0, 0.0, 0.0],
            [10, 2, 10],
            0.0,
            false,
            pose.hatBase,
            LivingModelGroup::Head,
        );
        hat0.parentPose = Some(pose.villager.head);
        boxes.push(hat0);

        let mut hat1 = model_box(
            [0, 76],
            [0.0, 0.0, 0.0],
            [7, 4, 7],
            0.0,
            false,
            pose.hatMiddle,
            LivingModelGroup::Head,
        );
        hat1.parentPose = Some(pose.hatBase);
        hat1.parentPose2 = Some(pose.villager.head);
        boxes.push(hat1);

        let mut hat2 = model_box(
            [0, 87],
            [0.0, 0.0, 0.0],
            [4, 4, 4],
            0.0,
            false,
            pose.hatUpper,
            LivingModelGroup::Head,
        );
        hat2.parentPose = Some(pose.hatMiddle);
        hat2.parentPose2 = Some(pose.hatBase);
        hat2.parentPose3 = Some(pose.villager.head);
        boxes.push(hat2);

        let mut hat3 = model_box(
            [0, 95],
            [0.0, 0.0, 0.0],
            [1, 2, 1],
            0.25,
            false,
            pose.hatTip,
            LivingModelGroup::Head,
        );
        hat3.parentPose = Some(pose.hatUpper);
        hat3.parentPose2 = Some(pose.hatMiddle);
        hat3.parentPose3 = Some(pose.hatBase);
        hat3.parentPose4 = Some(pose.villager.head);
        boxes.push(hat3);
        boxes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::client::renderer::entity::RenderLivingBase::LivingChildLayout;

    fn input() -> LivingRenderInput {
        LivingRenderInput {
            position: [0.0; 3],
            bodyYaw: 0.0,
            headYaw: 0.0,
            headPitch: 0.0,
            limbSwing: 0.0,
            limbSwingAmount: 0.0,
            ageInTicks: 100.0,
            swingProgress: 0.0,
            sneaking: false,
            child: false,
            deathRotation: 0.0,
            preScale: 1.0,
            preScaleXYZ: [1.0; 3],
            childLayout: LivingChildLayout::BIPED,
            adultTranslation: [0.0; 3],
        }
    }

    #[test]
    fn held_item_forces_nose_down_and_forward_offset() {
        let pose = ModelWitch::pose(input(), 4, true);
        assert!((pose.villager.nose.rotation[0] + 0.9).abs() < 1.0e-6);
        assert_eq!(pose.villager.nose.pivot, [0.0, -2.0, 0.0]);
        assert_eq!(pose.noseOffset, [0.0, 3.0, -1.5]);
    }
}
