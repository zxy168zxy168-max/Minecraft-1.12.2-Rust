use crate::net::minecraft::inventory::Container::Container;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::PacketBuffer::CodecError;

/// Network-visible slot state of MCP 1.12.2 `ContainerPlayer`.
///
/// Slot order is exact: crafting output 0, crafting input 1-4, armor 5-8,
/// main inventory 9-35, hotbar 36-44 and offhand 45.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerPlayer {
    pub windowId: i32,
    inventorySlots: Vec<ItemStack>,
    base: Container,
}

impl Default for ContainerPlayer {
    fn default() -> Self {
        Self {
            windowId: 0,
            inventorySlots: vec![ItemStack::EMPTY; Self::SLOT_COUNT],
            base: Container::default(),
        }
    }
}

impl ContainerPlayer {
    pub const SLOT_COUNT: usize = 46;

    pub fn putStackInSlot(&mut self, slotId: i32, stack: ItemStack) -> Result<(), CodecError> {
        let index = usize::try_from(slotId).map_err(|_| {
            CodecError::InvalidData(format!("negative ContainerPlayer slot {slotId}"))
        })?;
        let slot = self.inventorySlots.get_mut(index).ok_or_else(|| {
            CodecError::InvalidData(format!(
                "ContainerPlayer slot {slotId} outside 0..{}",
                Self::SLOT_COUNT - 1
            ))
        })?;
        *slot = stack;
        Ok(())
    }

    /// Port of `Container.func_190896_a`, with a protocol guard against a
    /// malformed list that would address slots beyond this concrete container.
    pub fn setAll(&mut self, stacks: &[ItemStack]) -> Result<(), CodecError> {
        if stacks.len() > Self::SLOT_COUNT {
            return Err(CodecError::InvalidData(format!(
                "{} stacks for {}-slot ContainerPlayer",
                stacks.len(),
                Self::SLOT_COUNT
            )));
        }
        for (index, stack) in stacks.iter().cloned().enumerate() {
            self.inventorySlots[index] = stack;
        }
        Ok(())
    }

    pub fn getSlot(&self, slotId: usize) -> Option<&ItemStack> {
        self.inventorySlots.get(slotId)
    }

    pub fn getSlotMut(&mut self, slotId: usize) -> Option<&mut ItemStack> {
        self.inventorySlots.get_mut(slotId)
    }

    /// Direct port of `Container.getNextTransactionID`; its InventoryPlayer
    /// parameter is intentionally unused by the original implementation.
    pub fn getNextTransactionID(&mut self) -> i16 {
        self.base.getNextTransactionID()
    }

    pub fn slots(&self) -> &[ItemStack] {
        &self.inventorySlots
    }

    /// Resets the protocol-visible `Container` quick-craft state. Vanilla does
    /// this whenever a non-QUICK_CRAFT click interrupts an active drag.
    pub fn resetQuickCraft(&mut self) {
        self.base.resetDrag();
    }

    /// Direct port of the `ClickType.QUICK_CRAFT` branch in
    /// `Container.slotClick` for the concrete player container.
    ///
    /// The cursor stack is mutated in place. The three packet phases are:
    /// start (`event=0`), add slot (`event=1`), and finish (`event=2`).
    pub fn quickCraft(
        &mut self,
        slotId: i32,
        dragType: i32,
        cursor: &mut ItemStack,
        creative: bool,
    ) -> bool {
        let previousEvent = self.base.dragEvent;
        self.base.dragEvent = Container::getDragEvent(dragType);

        if (previousEvent != 1 || self.base.dragEvent != 2) && previousEvent != self.base.dragEvent
        {
            self.resetQuickCraft();
            return false;
        }
        if cursor.isEmpty() {
            self.resetQuickCraft();
            return false;
        }

        match self.base.dragEvent {
            0 => {
                self.base.dragMode = Container::extractDragMode(dragType);
                if Container::isValidDragMode(self.base.dragMode, creative) {
                    self.base.dragEvent = 1;
                    self.base.dragSlots.clear();
                    true
                } else {
                    self.resetQuickCraft();
                    false
                }
            }
            1 => {
                let Ok(index) = usize::try_from(slotId) else {
                    return false;
                };
                let Some(slotStack) = self.inventorySlots.get(index) else {
                    return false;
                };
                if Container::canAddItemToSlot(slotStack, cursor, true)
                    && playerContainerSlotAccepts(slotId, cursor)
                    && (self.base.dragMode == 2
                        || cursor.getCount() > self.base.dragSlots.len() as i32)
                {
                    self.base.dragSlots.insert(index)
                } else {
                    false
                }
            }
            2 => {
                let mut changed = false;
                if !self.base.dragSlots.is_empty() {
                    let source = cursor.clone();
                    let mut remaining = cursor.getCount();
                    let slotCount = self.base.dragSlots.len();
                    let selected = self.base.dragSlots.iter().copied().collect::<Vec<_>>();
                    for index in selected {
                        let existing = self.inventorySlots[index].clone();
                        if !Container::canAddItemToSlot(&existing, cursor, true)
                            || !playerContainerSlotAccepts(index as i32, cursor)
                            || (self.base.dragMode != 2 && cursor.getCount() < slotCount as i32)
                        {
                            continue;
                        }
                        let oldCount = if existing.isEmpty() {
                            0
                        } else {
                            existing.getCount()
                        };
                        let mut placed = source.clone();
                        Container::computeStackSize(
                            slotCount,
                            self.base.dragMode,
                            &mut placed,
                            oldCount,
                        );
                        let limit = placed
                            .getMaxStackSize()
                            .min(playerContainerSlotLimit(index as i32, &placed));
                        if placed.getCount() > limit {
                            placed.setCount(limit);
                        }
                        remaining -= placed.getCount() - oldCount;
                        self.inventorySlots[index] = placed;
                        changed = true;
                    }
                    cursor.setCount(remaining);
                }
                self.resetQuickCraft();
                changed
            }
            _ => {
                self.resetQuickCraft();
                false
            }
        }
    }

    /// Direct port of `Container.mergeItemStack` for this concrete 46-slot
    /// container. Existing compatible stacks are filled before empty slots,
    /// and `reverseDirection` changes both traversal passes exactly as in MCP.
    pub fn mergeItemStack(
        &mut self,
        stack: &mut ItemStack,
        startIndex: usize,
        endIndex: usize,
        reverseDirection: bool,
    ) -> bool {
        if stack.isEmpty() || startIndex >= endIndex || endIndex > Self::SLOT_COUNT {
            return false;
        }
        let mut changed = false;
        let indices: Vec<usize> = if reverseDirection {
            (startIndex..endIndex).rev().collect()
        } else {
            (startIndex..endIndex).collect()
        };

        if stack.getMaxStackSize() > 1 {
            for &index in &indices {
                if stack.isEmpty() {
                    break;
                }
                let existing = self.inventorySlots[index].clone();
                if existing.isEmpty() || !existing.canStackWith(stack) {
                    continue;
                }
                let limit =
                    playerContainerSlotLimit(index as i32, stack).min(stack.getMaxStackSize());
                let capacity = limit - existing.getCount();
                if capacity <= 0 {
                    continue;
                }
                let moved = capacity.min(stack.getCount());
                let mut merged = existing;
                merged.grow(moved);
                stack.shrink(moved);
                self.inventorySlots[index] = merged;
                changed = true;
            }
        }

        for &index in &indices {
            if stack.isEmpty() {
                break;
            }
            if !self.inventorySlots[index].isEmpty()
                || !playerContainerSlotAccepts(index as i32, stack)
            {
                continue;
            }
            let moved = playerContainerSlotLimit(index as i32, stack)
                .min(stack.getMaxStackSize())
                .min(stack.getCount());
            if moved <= 0 {
                continue;
            }
            self.inventorySlots[index] = stack.splitStack(moved);
            changed = true;
        }
        changed
    }

    /// Port of `ContainerPlayer.transferStackInSlot`. Craft-result side effects
    /// (recipe unlock/stat/drop) remain server-owned, but all slot ranges and
    /// merge directions match the 1.12.2 method.
    pub fn transferStackInSlot(&mut self, index: usize) -> ItemStack {
        if index >= Self::SLOT_COUNT {
            return ItemStack::EMPTY;
        }
        let original = self.inventorySlots[index].clone();
        if original.isEmpty() {
            return ItemStack::EMPTY;
        }
        let mut moving = original.clone();
        let equipmentSlot = playerEquipmentContainerSlot(&moving);
        let merged = if index == 0 {
            self.mergeItemStack(&mut moving, 9, 45, true)
        } else if (1..9).contains(&index) {
            self.mergeItemStack(&mut moving, 9, 45, false)
        } else if let Some(slot) = equipmentSlot.filter(|slot| self.inventorySlots[*slot].isEmpty())
        {
            self.mergeItemStack(&mut moving, slot, slot + 1, false)
        } else if moving.itemId == 442 && self.inventorySlots[45].isEmpty() {
            self.mergeItemStack(&mut moving, 45, 46, false)
        } else if (9..36).contains(&index) {
            self.mergeItemStack(&mut moving, 36, 45, false)
        } else if (36..45).contains(&index) {
            self.mergeItemStack(&mut moving, 9, 36, false)
        } else {
            self.mergeItemStack(&mut moving, 9, 45, false)
        };
        if !merged || moving.getCount() == original.getCount() {
            return ItemStack::EMPTY;
        }
        self.inventorySlots[index] = moving;
        original
    }

    /// MCP `Container.slotClick` SWAP branch for hotbar buttons 0..8.
    pub fn swapWithHotbar(&mut self, slotId: usize, hotbarIndex: usize) -> bool {
        if slotId >= Self::SLOT_COUNT || hotbarIndex >= 9 {
            return false;
        }
        let hotbarSlot = 36 + hotbarIndex;
        if slotId == hotbarSlot {
            return false;
        }
        let mut hotbar = self.inventorySlots[hotbarSlot].clone();
        let target = self.inventorySlots[slotId].clone();
        if hotbar.isEmpty() {
            self.inventorySlots[hotbarSlot] = target;
            self.inventorySlots[slotId] = ItemStack::EMPTY;
            return true;
        }
        if target.isEmpty() {
            if !playerContainerSlotAccepts(slotId as i32, &hotbar) {
                return false;
            }
            let limit = playerContainerSlotLimit(slotId as i32, &hotbar);
            self.inventorySlots[slotId] = hotbar.splitStack(limit);
            self.inventorySlots[hotbarSlot] = hotbar;
            return true;
        }
        if !playerContainerSlotAccepts(slotId as i32, &hotbar) {
            return false;
        }
        let limit = playerContainerSlotLimit(slotId as i32, &hotbar);
        if hotbar.getCount() <= limit {
            self.inventorySlots[slotId] = hotbar;
            self.inventorySlots[hotbarSlot] = target;
            return true;
        }
        self.inventorySlots[slotId] = hotbar.splitStack(limit);
        self.inventorySlots[hotbarSlot] = hotbar;
        let mut displaced = target;
        self.mergeItemStack(&mut displaced, 9, 45, false);
        true
    }

    /// MCP THROW branch. Returns whether a non-empty stack was removed.
    pub fn throwFromSlot(&mut self, slotId: usize, wholeStack: bool) -> bool {
        if slotId >= Self::SLOT_COUNT || self.inventorySlots[slotId].isEmpty() {
            return false;
        }
        let amount = if wholeStack {
            self.inventorySlots[slotId].getCount()
        } else {
            1
        };
        let removed = self.inventorySlots[slotId].splitStack(amount);
        !removed.isEmpty()
    }

    /// MCP PICKUP_ALL two-pass collection. The cursor stack is updated in
    /// place; crafting-result slot 0 is excluded by ContainerPlayer.canMergeSlot.
    pub fn pickupAll(&mut self, cursor: &mut ItemStack, reverse: bool) -> bool {
        if cursor.isEmpty() || cursor.getCount() >= cursor.getMaxStackSize() {
            return false;
        }
        let indices: Vec<usize> = if reverse {
            (0..Self::SLOT_COUNT).rev().collect()
        } else {
            (0..Self::SLOT_COUNT).collect()
        };
        let mut changed = false;
        for pass in 0..2 {
            for &index in &indices {
                if cursor.getCount() >= cursor.getMaxStackSize() {
                    break;
                }
                if index == 0 {
                    continue;
                }
                let stack = self.inventorySlots[index].clone();
                if stack.isEmpty() || !stack.canStackWith(cursor) {
                    continue;
                }
                if pass == 0 && stack.getCount() == stack.getMaxStackSize() {
                    continue;
                }
                let moved = (cursor.getMaxStackSize() - cursor.getCount()).min(stack.getCount());
                if moved <= 0 {
                    continue;
                }
                let mut remaining = stack;
                remaining.shrink(moved);
                cursor.grow(moved);
                self.inventorySlots[index] = remaining;
                changed = true;
            }
        }
        changed
    }
}

pub fn playerContainerSlotLimit(slotId: i32, stack: &ItemStack) -> i32 {
    if (5..=8).contains(&slotId) {
        1
    } else {
        stack.getMaxStackSize().max(1)
    }
}

pub fn playerContainerSlotAccepts(slotId: i32, stack: &ItemStack) -> bool {
    if stack.isEmpty() {
        return true;
    }
    match slotId {
        0 => false,
        5 => matches!(stack.itemId, 86 | 298 | 302 | 306 | 310 | 314 | 397),
        6 => matches!(stack.itemId, 299 | 303 | 307 | 311 | 315 | 443),
        7 => matches!(stack.itemId, 300 | 304 | 308 | 312 | 316),
        8 => matches!(stack.itemId, 301 | 305 | 309 | 313 | 317),
        _ => (1..=45).contains(&slotId),
    }
}

fn playerEquipmentContainerSlot(stack: &ItemStack) -> Option<usize> {
    if playerContainerSlotAccepts(5, stack) {
        Some(5)
    } else if playerContainerSlotAccepts(6, stack) {
        Some(6)
    } else if playerContainerSlotAccepts(7, stack) {
        Some(7)
    } else if playerContainerSlotAccepts(8, stack) {
        Some(8)
    } else {
        None
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
    fn shift_click_main_inventory_moves_to_hotbar() {
        let mut container = ContainerPlayer::default();
        container.putStackInSlot(9, stack(339, 7)).unwrap();
        let returned = container.transferStackInSlot(9);
        assert_eq!(returned.itemId, 339);
        assert!(container.getSlot(9).unwrap().isEmpty());
        assert_eq!(container.getSlot(36).unwrap().getCount(), 7);
    }

    #[test]
    fn shift_click_equipment_uses_the_original_target_slot() {
        let mut container = ContainerPlayer::default();
        container.putStackInSlot(9, stack(310, 1)).unwrap();
        container.transferStackInSlot(9);
        assert_eq!(container.getSlot(5).unwrap().itemId, 310);
        container.putStackInSlot(10, stack(442, 1)).unwrap();
        container.transferStackInSlot(10);
        assert_eq!(container.getSlot(45).unwrap().itemId, 442);
    }

    #[test]
    fn hotbar_swap_respects_armor_slot_limits() {
        let mut container = ContainerPlayer::default();
        container.putStackInSlot(36, stack(310, 1)).unwrap();
        assert!(container.swapWithHotbar(5, 0));
        assert_eq!(container.getSlot(5).unwrap().itemId, 310);
        assert!(container.getSlot(36).unwrap().isEmpty());
    }

    #[test]
    fn quickcraft_evenly_splits_and_keeps_the_remainder() {
        let mut container = ContainerPlayer::default();
        let mut cursor = stack(339, 10);
        assert!(container.quickCraft(-999, Container::getQuickcraftMask(0, 0), &mut cursor, false));
        for slot in [9, 10, 11] {
            assert!(container.quickCraft(
                slot,
                Container::getQuickcraftMask(1, 0),
                &mut cursor,
                false
            ));
        }
        assert!(container.quickCraft(-999, Container::getQuickcraftMask(2, 0), &mut cursor, false));
        assert_eq!(cursor.getCount(), 1);
        for slot in [9_usize, 10, 11] {
            assert_eq!(container.getSlot(slot).unwrap().getCount(), 3);
        }
    }

    #[test]
    fn quickcraft_right_drag_places_one_per_slot() {
        let mut container = ContainerPlayer::default();
        let mut cursor = stack(339, 5);
        assert!(container.quickCraft(-999, Container::getQuickcraftMask(0, 1), &mut cursor, false));
        assert!(container.quickCraft(9, Container::getQuickcraftMask(1, 1), &mut cursor, false));
        assert!(container.quickCraft(10, Container::getQuickcraftMask(1, 1), &mut cursor, false));
        assert!(container.quickCraft(-999, Container::getQuickcraftMask(2, 1), &mut cursor, false));
        assert_eq!(cursor.getCount(), 3);
        assert_eq!(container.getSlot(9).unwrap().getCount(), 1);
        assert_eq!(container.getSlot(10).unwrap().getCount(), 1);
    }

    #[test]
    fn quickcraft_respects_equipment_slot_limit() {
        let mut container = ContainerPlayer::default();
        let mut cursor = stack(310, 3);
        assert!(container.quickCraft(-999, Container::getQuickcraftMask(0, 0), &mut cursor, false));
        assert!(container.quickCraft(5, Container::getQuickcraftMask(1, 0), &mut cursor, false));
        assert!(container.quickCraft(-999, Container::getQuickcraftMask(2, 0), &mut cursor, false));
        assert_eq!(container.getSlot(5).unwrap().getCount(), 1);
        assert_eq!(cursor.getCount(), 2);
    }

    #[test]
    fn creative_fill_mode_is_rejected_for_survival() {
        let mut container = ContainerPlayer::default();
        let mut cursor = stack(339, 1);
        assert!(!container.quickCraft(
            -999,
            Container::getQuickcraftMask(0, 2),
            &mut cursor,
            false
        ));
        assert!(container.base.dragSlots.is_empty());
    }

    #[test]
    fn pickup_all_uses_non_full_stacks_before_full_stacks() {
        let mut container = ContainerPlayer::default();
        container.putStackInSlot(9, stack(339, 64)).unwrap();
        container.putStackInSlot(10, stack(339, 3)).unwrap();
        let mut cursor = stack(339, 1);
        assert!(container.pickupAll(&mut cursor, false));
        assert_eq!(cursor.getCount(), 64);
        assert!(container.getSlot(10).unwrap().isEmpty());
        assert_eq!(container.getSlot(9).unwrap().getCount(), 4);
    }
}
