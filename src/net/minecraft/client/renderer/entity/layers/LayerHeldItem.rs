use crate::net::minecraft::client::renderer::block::model::ItemCameraTransforms::TransformType;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::util::EnumHandSide::EnumHandSide;

/// State/ordering portion of MCP 1.12.2 `LayerHeldItem`.
///
/// The Vulkan backend consumes these source-level decisions when it submits
/// the baked item quads. Arm post-rendering remains owned by `ModelBiped`, and
/// item camera transforms remain owned by `ItemCameraTransforms`.
pub struct LayerHeldItem;

impl LayerHeldItem {
    pub fn stackForSide<'a>(
        primaryHand: EnumHandSide,
        mainHand: &'a ItemStack,
        offHand: &'a ItemStack,
        physicalSide: EnumHandSide,
    ) -> &'a ItemStack {
        match (primaryHand, physicalSide) {
            (EnumHandSide::Right, EnumHandSide::Right)
            | (EnumHandSide::Left, EnumHandSide::Left) => mainHand,
            (EnumHandSide::Right, EnumHandSide::Left)
            | (EnumHandSide::Left, EnumHandSide::Right) => offHand,
        }
    }

    pub const fn transformType(physicalSide: EnumHandSide) -> TransformType {
        match physicalSide {
            EnumHandSide::Right => TransformType::ThirdPersonRightHand,
            EnumHandSide::Left => TransformType::ThirdPersonLeftHand,
        }
    }

    pub const fn leftHanded(physicalSide: EnumHandSide) -> bool {
        matches!(physicalSide, EnumHandSide::Left)
    }

    /// Translation performed after the -90 degree X and 180 degree Y turns in
    /// `LayerHeldItem.renderHeldItem`.
    pub const fn handTranslation(physicalSide: EnumHandSide) -> [f32; 3] {
        [
            match physicalSide {
                EnumHandSide::Right => 1.0 / 16.0,
                EnumHandSide::Left => -1.0 / 16.0,
            },
            0.125,
            -0.625,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(id: i16) -> ItemStack {
        ItemStack {
            itemId: id,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        }
    }

    #[test]
    fn physical_hand_stack_order_matches_primary_hand() {
        let main = stack(339);
        let off = stack(442);
        assert_eq!(
            LayerHeldItem::stackForSide(EnumHandSide::Right, &main, &off, EnumHandSide::Right,),
            &main,
        );
        assert_eq!(
            LayerHeldItem::stackForSide(EnumHandSide::Right, &main, &off, EnumHandSide::Left,),
            &off,
        );
        assert_eq!(
            LayerHeldItem::stackForSide(EnumHandSide::Left, &main, &off, EnumHandSide::Right,),
            &off,
        );
    }

    #[test]
    fn left_hand_uses_mirrored_translation_and_transform() {
        assert_eq!(
            LayerHeldItem::handTranslation(EnumHandSide::Left),
            [-0.0625, 0.125, -0.625]
        );
        assert_eq!(
            LayerHeldItem::transformType(EnumHandSide::Left),
            TransformType::ThirdPersonLeftHand,
        );
        assert!(LayerHeldItem::leftHanded(EnumHandSide::Left));
    }
}
