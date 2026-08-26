use crate::net::minecraft::client::model::ModelBiped::{BipedPose, PartPose};
use crate::net::minecraft::client::model::ModelPlayer::ModelBoxSpec;
use crate::net::minecraft::inventory::EntityEquipmentSlot::EntityEquipmentSlot;

/// MCP `LayerBipedArmor`: `ModelBiped(0.5F)` for leggings and
/// `ModelBiped(1.0F)` for the other armor slots.
pub struct LayerBipedArmor;

impl LayerBipedArmor {
    pub fn boxes(mut pose: BipedPose, slot: EntityEquipmentSlot) -> Vec<ModelBoxSpec> {
        // ModelPlayer's slim arms alter only the player skin model. Armor keeps
        // the ordinary four-pixel ModelBiped arm pivot for every skin type.
        pose.rightArm.pivot[1] = 2.0;
        pose.leftArm.pivot[1] = 2.0;
        let delta = if slot == EntityEquipmentSlot::Legs {
            0.5
        } else {
            1.0
        };
        match slot {
            EntityEquipmentSlot::Head => vec![
                model_box(
                    [0, 0],
                    [-4.0, -8.0, -4.0],
                    [8, 8, 8],
                    delta,
                    false,
                    pose.head,
                ),
                model_box(
                    [32, 0],
                    [-4.0, -8.0, -4.0],
                    [8, 8, 8],
                    delta + 0.5,
                    false,
                    pose.head,
                ),
            ],
            EntityEquipmentSlot::Chest => vec![
                model_box(
                    [16, 16],
                    [-4.0, 0.0, -2.0],
                    [8, 12, 4],
                    delta,
                    false,
                    pose.body,
                ),
                model_box(
                    [40, 16],
                    [-3.0, -2.0, -2.0],
                    [4, 12, 4],
                    delta,
                    false,
                    pose.rightArm,
                ),
                model_box(
                    [40, 16],
                    [-1.0, -2.0, -2.0],
                    [4, 12, 4],
                    delta,
                    true,
                    pose.leftArm,
                ),
            ],
            EntityEquipmentSlot::Legs => vec![
                model_box(
                    [16, 16],
                    [-4.0, 0.0, -2.0],
                    [8, 12, 4],
                    delta,
                    false,
                    pose.body,
                ),
                model_box(
                    [0, 16],
                    [-2.0, 0.0, -2.0],
                    [4, 12, 4],
                    delta,
                    false,
                    pose.rightLeg,
                ),
                model_box(
                    [0, 16],
                    [-2.0, 0.0, -2.0],
                    [4, 12, 4],
                    delta,
                    true,
                    pose.leftLeg,
                ),
            ],
            EntityEquipmentSlot::Feet => vec![
                model_box(
                    [0, 16],
                    [-2.0, 0.0, -2.0],
                    [4, 12, 4],
                    delta,
                    false,
                    pose.rightLeg,
                ),
                model_box(
                    [0, 16],
                    [-2.0, 0.0, -2.0],
                    [4, 12, 4],
                    delta,
                    true,
                    pose.leftLeg,
                ),
            ],
            EntityEquipmentSlot::Mainhand | EntityEquipmentSlot::Offhand => Vec::new(),
        }
    }
}

const fn model_box(
    texture: [i32; 2],
    origin: [f32; 3],
    size: [i32; 3],
    delta: f32,
    mirror: bool,
    pose: PartPose,
) -> ModelBoxSpec {
    ModelBoxSpec {
        texture,
        origin,
        size,
        delta,
        mirror,
        pose,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pose() -> BipedPose {
        BipedPose {
            head: PartPose::default(),
            body: PartPose::default(),
            rightArm: PartPose::default(),
            leftArm: PartPose::default(),
            rightLeg: PartPose::default(),
            leftLeg: PartPose::default(),
        }
    }

    #[test]
    fn slots_select_exact_model_parts() {
        assert_eq!(
            LayerBipedArmor::boxes(pose(), EntityEquipmentSlot::Head).len(),
            2
        );
        assert_eq!(
            LayerBipedArmor::boxes(pose(), EntityEquipmentSlot::Chest).len(),
            3
        );
        assert_eq!(
            LayerBipedArmor::boxes(pose(), EntityEquipmentSlot::Legs).len(),
            3
        );
        assert_eq!(
            LayerBipedArmor::boxes(pose(), EntityEquipmentSlot::Feet).len(),
            2
        );
    }

    #[test]
    fn leggings_use_half_pixel_model_and_mirrored_left_leg() {
        let boxes = LayerBipedArmor::boxes(pose(), EntityEquipmentSlot::Legs);
        assert!(boxes.iter().all(|part| (part.delta - 0.5).abs() < 1.0e-6));
        assert!(boxes[2].mirror);
    }
}
