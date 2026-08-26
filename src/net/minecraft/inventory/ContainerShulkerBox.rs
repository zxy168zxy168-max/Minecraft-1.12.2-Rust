use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::inventory::Container::Container;
use crate::net::minecraft::inventory::ContainerChest::ContainerChest;
use crate::net::minecraft::inventory::SlotShulkerBox::SlotShulkerBox;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::PacketBuffer::CodecError;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

/// Client-side port of MCP 1.12.2 `ContainerShulkerBox`.
///
/// The slot order is the original fixed 27-slot shulker inventory followed by
/// player main inventory and hotbar. `SlotShulkerBox#isItemValid` is preserved:
/// no shulker-box item may be placed in one of the first 27 slots.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerShulkerBox {
    inner: ContainerChest,
}

impl ContainerShulkerBox {
    pub const LOWER_SLOT_COUNT: usize = 27;

    pub fn new(
        windowId: i32,
        title: ITextComponent,
        slotCount: usize,
        playerInventory: &InventoryPlayer,
    ) -> Result<Self, CodecError> {
        if slotCount != Self::LOWER_SLOT_COUNT {
            return Err(CodecError::InvalidData(format!(
                "ContainerShulkerBox requires 27 lower slots, received {slotCount}"
            )));
        }
        Ok(Self {
            inner: ContainerChest::new(
                windowId,
                "minecraft:shulker_box",
                title,
                Self::LOWER_SLOT_COUNT,
                playerInventory,
            )?,
        })
    }

    pub const fn getNumRows(&self) -> usize {
        3
    }
    pub const fn lowerSlotCount(&self) -> usize {
        Self::LOWER_SLOT_COUNT
    }
    pub fn slotCount(&self) -> usize {
        self.inner.slotCount()
    }
    pub fn slots(&self) -> &[ItemStack] {
        self.inner.slots()
    }
    pub fn getSlot(&self, slotId: usize) -> Option<&ItemStack> {
        self.inner.getSlot(slotId)
    }
    pub fn windowId(&self) -> i32 {
        self.inner.windowId
    }
    pub fn title(&self) -> &ITextComponent {
        &self.inner.title
    }

    pub fn isItemValidForSlot(&self, slotId: i32, stack: &ItemStack) -> bool {
        if !(0..Self::LOWER_SLOT_COUNT as i32).contains(&slotId) || stack.isEmpty() {
            return true;
        }
        SlotShulkerBox::isItemValid(stack)
    }

    pub fn putStackInSlot(&mut self, slotId: i32, stack: ItemStack) -> Result<(), CodecError> {
        // Server synchronization remains authoritative. Slot validity is
        // enforced only for client-predicted user insertion paths, matching
        // Container/Slot separation in the Java client.
        self.inner.putStackInSlot(slotId, stack)
    }

    pub fn setAll(&mut self, stacks: &[ItemStack]) -> Result<(), CodecError> {
        self.inner.setAll(stacks)
    }

    pub fn syncFromPlayerInventory(&mut self, playerInventory: &InventoryPlayer) {
        self.inner.syncFromPlayerInventory(playerInventory);
    }

    pub fn syncToPlayerInventory(&self, playerInventory: &mut InventoryPlayer) {
        self.inner.syncToPlayerInventory(playerInventory);
    }

    pub fn getNextTransactionID(&mut self) -> i16 {
        self.inner.getNextTransactionID()
    }
    pub fn resetQuickCraft(&mut self) {
        self.inner.resetQuickCraft();
    }

    pub fn quickCraft(
        &mut self,
        slotId: i32,
        dragType: i32,
        cursor: &mut ItemStack,
        creative: bool,
    ) -> bool {
        // During QUICK_CRAFT event 1, GuiContainer adds the hovered slot only
        // when Slot#isItemValid succeeds. Prevent invalid shulker slots from
        // entering Container.dragSlots; event 2 can then delegate unchanged.
        if Container::getDragEvent(dragType) == 1 && !self.isItemValidForSlot(slotId, cursor) {
            return false;
        }
        self.inner.quickCraft(slotId, dragType, cursor, creative)
    }

    /// Port of `ContainerShulkerBox#transferStackInSlot`.
    pub fn transferStackInSlot(&mut self, index: usize) -> ItemStack {
        if index >= self.slotCount() {
            return ItemStack::EMPTY;
        }
        let original = self.getSlot(index).cloned().unwrap_or(ItemStack::EMPTY);
        if original.isEmpty() {
            return ItemStack::EMPTY;
        }
        let mut moving = original.clone();
        let merged = if index < Self::LOWER_SLOT_COUNT {
            self.mergeItemStack(&mut moving, Self::LOWER_SLOT_COUNT, self.slotCount(), true)
        } else {
            self.mergeItemStack(&mut moving, 0, Self::LOWER_SLOT_COUNT, false)
        };
        if !merged || moving.getCount() == original.getCount() {
            return ItemStack::EMPTY;
        }
        let _ = self.putStackInSlot(index as i32, moving);
        original
    }

    fn mergeItemStack(
        &mut self,
        stack: &mut ItemStack,
        startIndex: usize,
        endIndex: usize,
        reverseDirection: bool,
    ) -> bool {
        if stack.isEmpty() || startIndex >= endIndex || endIndex > self.slotCount() {
            return false;
        }
        let indices: Vec<usize> = if reverseDirection {
            (startIndex..endIndex).rev().collect()
        } else {
            (startIndex..endIndex).collect()
        };
        let mut changed = false;

        if stack.getMaxStackSize() > 1 {
            for &slotId in &indices {
                if stack.isEmpty() {
                    break;
                }
                if !self.isItemValidForSlot(slotId as i32, stack) {
                    continue;
                }
                let existing = self.getSlot(slotId).cloned().unwrap_or(ItemStack::EMPTY);
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
                let _ = self.putStackInSlot(slotId as i32, merged);
                changed = true;
            }
        }

        for &slotId in &indices {
            if stack.isEmpty() {
                break;
            }
            if !self.isItemValidForSlot(slotId as i32, stack) {
                continue;
            }
            if self
                .getSlot(slotId)
                .is_some_and(|existing| !existing.isEmpty())
            {
                continue;
            }
            let moved = stack.getMaxStackSize().min(stack.getCount());
            let placed = stack.splitStack(moved);
            let _ = self.putStackInSlot(slotId as i32, placed);
            changed = true;
        }
        changed
    }

    pub fn swapWithHotbar(&mut self, slotId: usize, hotbarIndex: usize) -> bool {
        if slotId >= self.slotCount() || hotbarIndex >= 9 {
            return false;
        }
        let hotbarSlot = Self::LOWER_SLOT_COUNT + 27 + hotbarIndex;
        if slotId == hotbarSlot {
            return false;
        }
        let hotbarStack = self
            .getSlot(hotbarSlot)
            .cloned()
            .unwrap_or(ItemStack::EMPTY);
        if !self.isItemValidForSlot(slotId as i32, &hotbarStack) {
            return false;
        }
        self.inner.swapWithHotbar(slotId, hotbarIndex)
    }

    pub fn throwFromSlot(&mut self, slotId: usize, wholeStack: bool) -> bool {
        self.inner.throwFromSlot(slotId, wholeStack)
    }

    pub fn pickupAll(&mut self, cursor: &mut ItemStack, reverse: bool) -> bool {
        self.inner.pickupAll(cursor, reverse)
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
    fn geometry_and_slot_order_match_container_shulker_box() {
        let mut player = InventoryPlayer::default();
        player.mainInventory[0] = stack(1, 3);
        let container =
            ContainerShulkerBox::new(7, ITextComponent::fromPlainText("Shulker Box"), 27, &player)
                .unwrap();
        assert_eq!(container.slotCount(), 63);
        assert_eq!(container.getSlot(54).unwrap().getCount(), 3);
    }

    #[test]
    fn slot_shulker_box_rejects_nested_shulker_boxes() {
        let container = ContainerShulkerBox::new(
            7,
            ITextComponent::fromPlainText("Shulker Box"),
            27,
            &InventoryPlayer::default(),
        )
        .unwrap();
        assert!(!container.isItemValidForSlot(0, &stack(219, 1)));
        assert!(!container.isItemValidForSlot(26, &stack(234, 1)));
        assert!(container.isItemValidForSlot(27, &stack(219, 1)));
        assert!(container.isItemValidForSlot(0, &stack(1, 1)));
    }

    #[test]
    fn shift_click_does_not_insert_shulker_box_into_lower_inventory() {
        let mut player = InventoryPlayer::default();
        player.mainInventory[9] = stack(219, 1);
        let mut container =
            ContainerShulkerBox::new(7, ITextComponent::fromPlainText("Shulker Box"), 27, &player)
                .unwrap();
        assert!(container.transferStackInSlot(27).isEmpty());
        assert_eq!(container.getSlot(27).unwrap().itemId, 219);
    }
}
