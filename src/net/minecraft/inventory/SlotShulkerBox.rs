use crate::net::minecraft::item::ItemStack::ItemStack;

/// Client-side semantic port of MCP 1.12.2 `SlotShulkerBox`.
///
/// The generic `Slot` storage is still represented by the concrete container's
/// slot vector, but this class owns the original placement-validity rule so it
/// is not folded into `ContainerShulkerBox` or a renderer-only special case.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SlotShulkerBox;

impl SlotShulkerBox {
    /// `!(Block.getBlockFromItem(stack.getItem()) instanceof BlockShulkerBox)`.
    pub const fn isItemValid(stack: &ItemStack) -> bool {
        !is_shulker_box_item(stack)
    }
}

pub const fn is_shulker_box_item(stack: &ItemStack) -> bool {
    // Protocol-340 block/item registry IDs for the sixteen 1.12.2 shulker
    // boxes. This is the concrete BlockShulkerBox family, not an NBT/name test.
    matches!(stack.itemId, 219..=234)
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn stack(id: i16) -> ItemStack {
        ItemStack {
            itemId: id,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        }
    }

    #[test]
    fn rejects_all_block_shulker_box_items_only() {
        assert!(!SlotShulkerBox::isItemValid(&stack(219)));
        assert!(!SlotShulkerBox::isItemValid(&stack(234)));
        assert!(SlotShulkerBox::isItemValid(&stack(54)));
        assert!(SlotShulkerBox::isItemValid(&ItemStack::EMPTY));
    }
}
