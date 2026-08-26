use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::inventory::Container::Container;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::PacketBuffer::CodecError;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

/// Client-side state for MCP 1.12.2 `ContainerChest` together with the
/// `InventoryBasic`/`ContainerLocalMenu` metadata supplied by
/// `SPacketOpenWindow`.
///
/// Slot order is exact: lower inventory first, then player main inventory
/// (indices 9..35), then hotbar (indices 0..8).
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerChest {
    pub windowId: i32,
    pub guiId: String,
    pub title: ITextComponent,
    numRows: usize,
    inventorySlots: Vec<ItemStack>,
    base: Container,
}

impl ContainerChest {
    pub fn new(
        windowId: i32,
        guiId: impl Into<String>,
        title: ITextComponent,
        slotCount: usize,
        playerInventory: &InventoryPlayer,
    ) -> Result<Self, CodecError> {
        // MCP `GuiChest` and `ContainerChest` both derive the visible lower
        // inventory from integer division by nine. Do not invent validation or
        // clamp the row count: unusual mod/plugin menu sizes must follow the
        // same floor semantics as the 1.12.2 client.
        let numRows = slotCount / 9;
        let lowerSlotCount = numRows * 9;
        let mut inventorySlots = vec![ItemStack::EMPTY; lowerSlotCount + 36];
        for playerIndex in 9..36 {
            inventorySlots[lowerSlotCount + playerIndex - 9] = playerInventory
                .mainInventory
                .get(playerIndex)
                .cloned()
                .unwrap_or(ItemStack::EMPTY);
        }
        for hotbarIndex in 0..9 {
            inventorySlots[lowerSlotCount + 27 + hotbarIndex] = playerInventory
                .mainInventory
                .get(hotbarIndex)
                .cloned()
                .unwrap_or(ItemStack::EMPTY);
        }
        Ok(Self {
            windowId,
            guiId: guiId.into(),
            title,
            numRows,
            inventorySlots,
            base: Container::default(),
        })
    }

    pub const fn getNumRows(&self) -> usize {
        self.numRows
    }
    pub const fn lowerSlotCount(&self) -> usize {
        self.numRows * 9
    }
    pub fn slotCount(&self) -> usize {
        self.inventorySlots.len()
    }
    pub fn slots(&self) -> &[ItemStack] {
        &self.inventorySlots
    }
    pub fn getSlot(&self, slotId: usize) -> Option<&ItemStack> {
        self.inventorySlots.get(slotId)
    }

    pub fn putStackInSlot(&mut self, slotId: i32, stack: ItemStack) -> Result<(), CodecError> {
        let index = usize::try_from(slotId).map_err(|_| {
            CodecError::InvalidData(format!("negative ContainerChest slot {slotId}"))
        })?;
        let maximum = self.inventorySlots.len().saturating_sub(1);
        let slot = self.inventorySlots.get_mut(index).ok_or_else(|| {
            CodecError::InvalidData(format!("ContainerChest slot {slotId} outside 0..{maximum}"))
        })?;
        *slot = stack;
        Ok(())
    }

    /// Port of `Container.func_190896_a` for the concrete chest container.
    pub fn setAll(&mut self, stacks: &[ItemStack]) -> Result<(), CodecError> {
        if stacks.len() != self.inventorySlots.len() {
            return Err(CodecError::InvalidData(format!(
                "{} stacks for {}-slot ContainerChest",
                stacks.len(),
                self.inventorySlots.len()
            )));
        }
        for (index, stack) in stacks.iter().cloned().enumerate() {
            self.inventorySlots[index] = stack;
        }
        Ok(())
    }

    pub fn syncFromPlayerInventory(&mut self, playerInventory: &InventoryPlayer) {
        let lower = self.lowerSlotCount();
        for playerIndex in 9..36 {
            if let Some(source) = playerInventory.mainInventory.get(playerIndex) {
                if let Some(target) = self.inventorySlots.get_mut(lower + playerIndex - 9) {
                    *target = source.clone();
                }
            }
        }
        for hotbarIndex in 0..9 {
            if let Some(source) = playerInventory.mainInventory.get(hotbarIndex) {
                if let Some(target) = self.inventorySlots.get_mut(lower + 27 + hotbarIndex) {
                    *target = source.clone();
                }
            }
        }
    }

    pub fn syncToPlayerInventory(&self, playerInventory: &mut InventoryPlayer) {
        let lower = self.lowerSlotCount();
        for playerIndex in 9..36 {
            if let Some(stack) = self.inventorySlots.get(lower + playerIndex - 9) {
                if let Some(target) = playerInventory.mainInventory.get_mut(playerIndex) {
                    *target = stack.clone();
                }
            }
        }
        for hotbarIndex in 0..9 {
            if let Some(stack) = self.inventorySlots.get(lower + 27 + hotbarIndex) {
                if let Some(target) = playerInventory.mainInventory.get_mut(hotbarIndex) {
                    *target = stack.clone();
                }
            }
        }
    }

    pub fn getNextTransactionID(&mut self) -> i16 {
        self.base.getNextTransactionID()
    }
    pub fn resetQuickCraft(&mut self) {
        self.base.resetDrag();
    }

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
                        if placed.getCount() > placed.getMaxStackSize() {
                            placed.setCount(placed.getMaxStackSize());
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

    /// Direct port of `Container.mergeItemStack` for chest slots.
    pub fn mergeItemStack(
        &mut self,
        stack: &mut ItemStack,
        startIndex: usize,
        endIndex: usize,
        reverseDirection: bool,
    ) -> bool {
        if stack.isEmpty() || startIndex >= endIndex || endIndex > self.inventorySlots.len() {
            return false;
        }
        let indices: Vec<usize> = if reverseDirection {
            (startIndex..endIndex).rev().collect()
        } else {
            (startIndex..endIndex).collect()
        };
        let mut changed = false;
        if stack.getMaxStackSize() > 1 {
            for &index in &indices {
                if stack.isEmpty() {
                    break;
                }
                let existing = self.inventorySlots[index].clone();
                if existing.isEmpty() || !existing.canStackWith(stack) {
                    continue;
                }
                let capacity = stack.getMaxStackSize() - existing.getCount();
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
            if !self.inventorySlots[index].isEmpty() {
                continue;
            }
            let moved = stack.getMaxStackSize().min(stack.getCount());
            self.inventorySlots[index] = stack.splitStack(moved);
            changed = true;
        }
        changed
    }

    /// Port of `ContainerChest.transferStackInSlot`.
    pub fn transferStackInSlot(&mut self, index: usize) -> ItemStack {
        if index >= self.inventorySlots.len() {
            return ItemStack::EMPTY;
        }
        let original = self.inventorySlots[index].clone();
        if original.isEmpty() {
            return ItemStack::EMPTY;
        }
        let mut moving = original.clone();
        let lower = self.lowerSlotCount();
        let merged = if index < lower {
            self.mergeItemStack(&mut moving, lower, self.inventorySlots.len(), true)
        } else {
            self.mergeItemStack(&mut moving, 0, lower, false)
        };
        if !merged || moving.getCount() == original.getCount() {
            return ItemStack::EMPTY;
        }
        self.inventorySlots[index] = moving;
        original
    }

    /// `Container.slotClick` SWAP branch. The chest hotbar starts after the
    /// lower inventory and the 27 player main-inventory slots.
    pub fn swapWithHotbar(&mut self, slotId: usize, hotbarIndex: usize) -> bool {
        if slotId >= self.inventorySlots.len() || hotbarIndex >= 9 {
            return false;
        }
        let hotbarSlot = self.lowerSlotCount() + 27 + hotbarIndex;
        if slotId == hotbarSlot {
            return false;
        }
        self.inventorySlots.swap(slotId, hotbarSlot);
        true
    }

    pub fn throwFromSlot(&mut self, slotId: usize, wholeStack: bool) -> bool {
        let Some(stack) = self.inventorySlots.get_mut(slotId) else {
            return false;
        };
        if stack.isEmpty() {
            return false;
        }
        let amount = if wholeStack { stack.getCount() } else { 1 };
        !stack.splitStack(amount).isEmpty()
    }

    pub fn pickupAll(&mut self, cursor: &mut ItemStack, reverse: bool) -> bool {
        if cursor.isEmpty() || cursor.getCount() >= cursor.getMaxStackSize() {
            return false;
        }
        let indices: Vec<usize> = if reverse {
            (0..self.inventorySlots.len()).rev().collect()
        } else {
            (0..self.inventorySlots.len()).collect()
        };
        let mut changed = false;
        for pass in 0..2 {
            for &index in &indices {
                if cursor.getCount() >= cursor.getMaxStackSize() {
                    break;
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
    fn chest_slot_order_and_shift_merge_match_mcp() {
        let mut player = InventoryPlayer::default();
        player.mainInventory[0] = stack(339, 2);
        let mut chest = ContainerChest::new(
            2,
            "minecraft:container",
            ITextComponent::fromPlainText("Chest"),
            27,
            &player,
        )
        .unwrap();
        assert_eq!(chest.getNumRows(), 3);
        assert_eq!(chest.getSlot(27 + 27).unwrap().getCount(), 2);
        chest.putStackInSlot(0, stack(1, 8)).unwrap();
        assert_eq!(chest.transferStackInSlot(0).itemId, 1);
        assert!(chest.getSlot(0).unwrap().isEmpty());
        assert_eq!(chest.getSlot(chest.slotCount() - 1).unwrap().getCount(), 8);
    }
}
