use crate::net::minecraft::client::model::ModelBiped::{BipedPose, PartPose};
use crate::net::minecraft::util::EnumHandSide::EnumHandSide;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelBoxSpec {
    pub texture: [i32; 2],
    pub origin: [f32; 3],
    pub size: [i32; 3],
    pub delta: f32,
    pub mirror: bool,
    pub pose: PartPose,
}

/// Geometry/state projection of MCP 1.12.2 `ModelPlayer` after
/// `setRotationAngles` has copied base-part transforms to the wear layers.
pub struct ModelPlayer;

impl ModelPlayer {
    pub fn boxes(pose: BipedPose, slim: bool, skinParts: u8) -> Vec<ModelBoxSpec> {
        let armWidth = if slim { 3 } else { 4 };
        let leftArmX = -1.0;
        let rightArmX = if slim { -2.0 } else { -3.0 };
        let mut boxes = vec![
            box_spec([0, 0], [-4.0, -8.0, -4.0], [8, 8, 8], 0.0, false, pose.head),
            box_spec(
                [16, 16],
                [-4.0, 0.0, -2.0],
                [8, 12, 4],
                0.0,
                false,
                pose.body,
            ),
            box_spec(
                [40, 16],
                [rightArmX, -2.0, -2.0],
                [armWidth, 12, 4],
                0.0,
                false,
                pose.rightArm,
            ),
            // ModelPlayer replaces ModelBiped's mirrored legacy left limbs
            // with the dedicated 64x64 skin regions. These boxes are not
            // mirrored in the 1.12.2 constructor.
            box_spec(
                [32, 48],
                [leftArmX, -2.0, -2.0],
                [armWidth, 12, 4],
                0.0,
                false,
                pose.leftArm,
            ),
            box_spec(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, 12, 4],
                0.0,
                false,
                pose.rightLeg,
            ),
            box_spec(
                [16, 48],
                [-2.0, 0.0, -2.0],
                [4, 12, 4],
                0.0,
                false,
                pose.leftLeg,
            ),
        ];

        // `EnumPlayerModelParts` masks are side-specific. Preserve the MCP
        // LEFT/RIGHT mapping rather than assigning the old mirrored limb UVs.
        if skinParts & 0x40 != 0 {
            boxes.push(box_spec(
                [32, 0],
                [-4.0, -8.0, -4.0],
                [8, 8, 8],
                0.5,
                false,
                pose.head,
            ));
        }
        if skinParts & 0x02 != 0 {
            boxes.push(box_spec(
                [16, 32],
                [-4.0, 0.0, -2.0],
                [8, 12, 4],
                0.25,
                false,
                pose.body,
            ));
        }
        if skinParts & 0x04 != 0 {
            boxes.push(box_spec(
                [48, 48],
                [leftArmX, -2.0, -2.0],
                [armWidth, 12, 4],
                0.25,
                false,
                pose.leftArm,
            ));
        }
        if skinParts & 0x08 != 0 {
            boxes.push(box_spec(
                [40, 32],
                [rightArmX, -2.0, -2.0],
                [armWidth, 12, 4],
                0.25,
                false,
                pose.rightArm,
            ));
        }
        if skinParts & 0x10 != 0 {
            boxes.push(box_spec(
                [0, 48],
                [-2.0, 0.0, -2.0],
                [4, 12, 4],
                0.25,
                false,
                pose.leftLeg,
            ));
        }
        if skinParts & 0x20 != 0 {
            boxes.push(box_spec(
                [0, 32],
                [-2.0, 0.0, -2.0],
                [4, 12, 4],
                0.25,
                false,
                pose.rightLeg,
            ));
        }
        boxes
    }

    /// The exact base-arm and optional sleeve boxes used by
    /// `RenderPlayer.renderRightArm` / `renderLeftArm`.
    pub fn firstPersonArmBoxes(
        pose: PartPose,
        slim: bool,
        side: EnumHandSide,
        showWear: bool,
    ) -> Vec<ModelBoxSpec> {
        let armWidth = if slim { 3 } else { 4 };
        let (texture, origin, wearTexture) = match side {
            EnumHandSide::Right => (
                [40, 16],
                [if slim { -2.0 } else { -3.0 }, -2.0, -2.0],
                [40, 32],
            ),
            EnumHandSide::Left => ([32, 48], [-1.0, -2.0, -2.0], [48, 48]),
        };
        let mut boxes = vec![box_spec(
            texture,
            origin,
            [armWidth, 12, 4],
            0.0,
            false,
            pose,
        )];
        if showWear {
            boxes.push(box_spec(
                wearTexture,
                origin,
                [armWidth, 12, 4],
                0.25,
                false,
                pose,
            ));
        }
        boxes
    }
}

const fn box_spec(
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

    fn neutral_pose() -> BipedPose {
        let part = PartPose::default();
        BipedPose {
            head: part,
            body: part,
            rightArm: part,
            leftArm: part,
            rightLeg: part,
            leftLeg: part,
        }
    }

    #[test]
    fn player_64x64_left_limbs_use_dedicated_unmirrored_regions() {
        let boxes = ModelPlayer::boxes(neutral_pose(), false, 0);
        assert_eq!(boxes[3].texture, [32, 48]);
        assert!(!boxes[3].mirror);
        assert_eq!(boxes[5].texture, [16, 48]);
        assert!(!boxes[5].mirror);
    }

    #[test]
    fn model_part_masks_keep_mcp_left_and_right_sides() {
        let boxes = ModelPlayer::boxes(neutral_pose(), false, 0x04 | 0x08 | 0x10 | 0x20);
        let overlays = &boxes[6..];
        assert_eq!(overlays[0].texture, [48, 48]);
        assert_eq!(overlays[0].pose, PartPose::default());
        assert_eq!(overlays[1].texture, [40, 32]);
        assert_eq!(overlays[2].texture, [0, 48]);
        assert_eq!(overlays[3].texture, [0, 32]);
        assert!(overlays.iter().all(|part| !part.mirror));
    }

    #[test]
    fn first_person_arm_uses_side_specific_skin_regions_and_wear_inflation() {
        let right =
            ModelPlayer::firstPersonArmBoxes(PartPose::default(), false, EnumHandSide::Right, true);
        assert_eq!(right[0].texture, [40, 16]);
        assert_eq!(right[1].texture, [40, 32]);
        assert_eq!(right[1].delta, 0.25);

        let leftSlim =
            ModelPlayer::firstPersonArmBoxes(PartPose::default(), true, EnumHandSide::Left, false);
        assert_eq!(leftSlim[0].texture, [32, 48]);
        assert_eq!(leftSlim[0].size, [3, 12, 4]);
        assert_eq!(leftSlim.len(), 1);
    }
}
