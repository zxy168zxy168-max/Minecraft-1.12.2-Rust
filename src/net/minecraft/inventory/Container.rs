use std::collections::BTreeSet;

use crate::net::minecraft::item::ItemStack::ItemStack;

/// Shared MCP 1.12.2 `Container` quick-craft helpers.
///
/// Concrete slot ownership remains in each Rust container, but these bit masks
/// and stack-distribution rules are protocol-visible and therefore live at the
/// same responsibility boundary as the Java base class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    transactionID: i16,
    pub(crate) dragMode: i32,
    pub(crate) dragEvent: i32,
    pub(crate) dragSlots: BTreeSet<usize>,
}

impl Default for Container {
    fn default() -> Self {
        Self {
            transactionID: 0,
            dragMode: -1,
            dragEvent: 0,
            dragSlots: BTreeSet::new(),
        }
    }
}

impl Container {
    /// MCP `Container.getNextTransactionID`; the Java InventoryPlayer
    /// argument is deliberately absent because the method never reads it.
    pub fn getNextTransactionID(&mut self) -> i16 {
        self.transactionID = self.transactionID.wrapping_add(1);
        self.transactionID
    }

    /// MCP `Container.resetDrag`.
    pub fn resetDrag(&mut self) {
        self.dragEvent = 0;
        self.dragSlots.clear();
    }

    /// MCP `Container.extractDragMode`.
    pub const fn extractDragMode(eventButton: i32) -> i32 {
        eventButton >> 2 & 3
    }

    /// MCP `Container.getDragEvent`.
    pub const fn getDragEvent(clickedButton: i32) -> i32 {
        clickedButton & 3
    }

    /// MCP `Container.getQuickcraftMask`.
    pub const fn getQuickcraftMask(event: i32, mode: i32) -> i32 {
        event & 3 | (mode & 3) << 2
    }

    /// MCP `Container.isValidDragMode`.
    pub const fn isValidDragMode(dragMode: i32, creative: bool) -> bool {
        dragMode == 0 || dragMode == 1 || (dragMode == 2 && creative)
    }

    /// MCP `Container.canAddItemToSlot`, expressed against the concrete stack
    /// currently in a slot. Slot validity and per-slot limits are checked by
    /// the owning container just as they are by `Slot.isItemValid` and
    /// `Slot.getItemStackLimit` in Java.
    pub fn canAddItemToSlot(
        slotStack: &ItemStack,
        stack: &ItemStack,
        stackSizeMatters: bool,
    ) -> bool {
        if slotStack.isEmpty() {
            return true;
        }
        slotStack.canStackWith(stack)
            && slotStack.getCount()
                + if stackSizeMatters {
                    0
                } else {
                    stack.getCount()
                }
                <= stack.getMaxStackSize()
    }

    /// MCP `Container.computeStackSize`.
    pub fn computeStackSize(
        dragSlotCount: usize,
        dragMode: i32,
        stack: &mut ItemStack,
        slotStackSize: i32,
    ) {
        if dragSlotCount == 0 || stack.isEmpty() {
            return;
        }
        let count = match dragMode {
            0 => stack.getCount() / dragSlotCount as i32,
            1 => 1,
            2 => stack.getMaxStackSize(),
            _ => stack.getCount(),
        };
        stack.setCount(count + slotStackSize);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(id: i16, count: u8) -> ItemStack {
        ItemStack {
            itemId: id,
            count,
            itemDamage: 0,
            tagCompound: None,
        }
    }

    #[test]
    fn quickcraft_masks_round_trip() {
        for mode in 0..=2 {
            for event in 0..=2 {
                let mask = Container::getQuickcraftMask(event, mode);
                assert_eq!(Container::getDragEvent(mask), event);
                assert_eq!(Container::extractDragMode(mask), mode);
            }
        }
    }

    #[test]
    fn split_modes_match_container_compute_stack_size() {
        let mut even = stack(339, 10);
        Container::computeStackSize(3, 0, &mut even, 2);
        assert_eq!(even.getCount(), 5);

        let mut one = stack(339, 10);
        Container::computeStackSize(3, 1, &mut one, 2);
        assert_eq!(one.getCount(), 3);

        let mut creative = stack(339, 10);
        Container::computeStackSize(3, 2, &mut creative, 2);
        assert_eq!(creative.getCount(), 66);
    }
}
