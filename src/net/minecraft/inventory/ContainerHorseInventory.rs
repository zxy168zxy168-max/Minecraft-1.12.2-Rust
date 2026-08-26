use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::inventory::Container::Container;
use crate::net::minecraft::inventory::ContainerChest::ContainerChest;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::PacketBuffer::CodecError;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorseInventoryKind {
    Horse,
    Donkey,
    Mule,
    SkeletonHorse,
    ZombieHorse,
    Llama,
}

impl HorseInventoryKind {
    pub fn fromRegistryName(name: &str) -> Option<Self> {
        match name {
            "horse" => Some(Self::Horse),
            "donkey" => Some(Self::Donkey),
            "mule" => Some(Self::Mule),
            "skeleton_horse" => Some(Self::SkeletonHorse),
            "zombie_horse" => Some(Self::ZombieHorse),
            "llama" => Some(Self::Llama),
            _ => None,
        }
    }

    /// MCP `AbstractHorse#func_190685_dA`; only llama overrides this false.
    pub const fn canUseSaddleSlot(self) -> bool {
        !matches!(self, Self::Llama)
    }

    /// MCP `AbstractHorse#func_190677_dK`, overridden by EntityHorse/EntityLlama.
    pub const fn hasEquipmentSlot(self) -> bool {
        matches!(self, Self::Horse | Self::Llama)
    }

    pub const fn isLlama(self) -> bool {
        matches!(self, Self::Llama)
    }
}

/// Protocol/entity facts required to instantiate MCP 1.12.2
/// `ContainerHorseInventory` on the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HorseInventorySpec {
    pub entityId: i32,
    pub kind: HorseInventoryKind,
    pub chested: bool,
    /// AbstractChestHorse#func_190696_dl: 5 for donkey/mule, strength 1..5 for llama.
    pub chestColumns: i32,
}

impl HorseInventorySpec {
    pub fn lowerSlotCount(self) -> usize {
        if self.chested {
            (2 + self.chestColumns.clamp(1, 5) * 3) as usize
        } else {
            2
        }
    }
}

/// Client-side port of MCP 1.12.2 `ContainerHorseInventory`.
///
/// Slot order is the horse inventory reported by SPacketOpenWindow followed by
/// player main inventory and hotbar. Entity-dependent saddle, armor/carpet and
/// chest-column rules remain explicit rather than being treated as a chest.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerHorseInventory {
    inner: ContainerChest,
    spec: HorseInventorySpec,
}

impl ContainerHorseInventory {
    pub fn new(
        windowId: i32,
        title: ITextComponent,
        slotCount: usize,
        playerInventory: &InventoryPlayer,
        spec: HorseInventorySpec,
    ) -> Result<Self, CodecError> {
        let expected = spec.lowerSlotCount();
        if slotCount != expected {
            return Err(CodecError::InvalidData(format!(
                "EntityHorse {} reports {slotCount} slots; entity state requires {expected}",
                spec.entityId,
            )));
        }
        Ok(Self {
            inner: ContainerChest::new(windowId, "EntityHorse", title, slotCount, playerInventory)?,
            spec,
        })
    }

    pub const fn spec(&self) -> HorseInventorySpec {
        self.spec
    }
    pub fn lowerSlotCount(&self) -> usize {
        self.spec.lowerSlotCount()
    }
    pub const fn getNumRows(&self) -> usize {
        0
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
        if stack.isEmpty() || slotId < 0 || slotId as usize >= self.slotCount() {
            return true;
        }
        match slotId {
            0 => {
                self.spec.kind.canUseSaddleSlot()
                    && stack.itemId == 329
                    && self.getSlot(0).map_or(true, ItemStack::isEmpty)
            }
            1 => self.equipmentItemValid(stack),
            id if (id as usize) < self.lowerSlotCount() => true,
            _ => true,
        }
    }

    fn equipmentItemValid(&self, stack: &ItemStack) -> bool {
        match self.spec.kind {
            HorseInventoryKind::Horse => matches!(stack.itemId, 417..=419),
            HorseInventoryKind::Llama => stack.itemId == 171,
            _ => false,
        }
    }

    pub fn slotLimit(&self, slotId: i32, stack: &ItemStack) -> i32 {
        if slotId == 1 {
            1
        } else {
            stack.getMaxStackSize()
        }
    }

    pub fn putStackInSlot(&mut self, slotId: i32, stack: ItemStack) -> Result<(), CodecError> {
        self.inner.putStackInSlot(slotId, stack)
    }

    pub fn setAll(&mut self, stacks: &[ItemStack]) -> Result<(), CodecError> {
        self.inner.setAll(stacks)
    }

    pub fn syncFromPlayerInventory(&mut self, inventory: &InventoryPlayer) {
        self.inner.syncFromPlayerInventory(inventory);
    }

    pub fn syncToPlayerInventory(&self, inventory: &mut InventoryPlayer) {
        self.inner.syncToPlayerInventory(inventory);
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
        if Container::getDragEvent(dragType) == 1 && !self.isItemValidForSlot(slotId, cursor) {
            return false;
        }
        self.inner.quickCraft(slotId, dragType, cursor, creative)
    }

    /// Exact branch ordering from `ContainerHorseInventory#transferStackInSlot`.
    pub fn transferStackInSlot(&mut self, index: usize) -> ItemStack {
        if index >= self.slotCount() {
            return ItemStack::EMPTY;
        }
        let original = self.getSlot(index).cloned().unwrap_or(ItemStack::EMPTY);
        if original.isEmpty() {
            return ItemStack::EMPTY;
        }
        let mut moving = original.clone();
        let lower = self.lowerSlotCount();
        let merged = if index < lower {
            self.inner
                .mergeItemStack(&mut moving, lower, self.slotCount(), true)
        } else if self.isItemValidForSlot(1, &moving)
            && self.getSlot(1).is_some_and(ItemStack::isEmpty)
        {
            self.mergeValid(&mut moving, 1, 2, false)
        } else if self.isItemValidForSlot(0, &moving) {
            self.mergeValid(&mut moving, 0, 1, false)
        } else if lower > 2 {
            self.mergeValid(&mut moving, 2, lower, false)
        } else {
            false
        };
        if !merged || moving.getCount() == original.getCount() {
            return ItemStack::EMPTY;
        }
        let _ = self.putStackInSlot(index as i32, moving);
        original
    }

    fn mergeValid(
        &mut self,
        stack: &mut ItemStack,
        start: usize,
        end: usize,
        reverse: bool,
    ) -> bool {
        if stack.isEmpty() || start >= end || end > self.slotCount() {
            return false;
        }
        let indices: Vec<usize> = if reverse {
            (start..end).rev().collect()
        } else {
            (start..end).collect()
        };
        let mut changed = false;
        if stack.getMaxStackSize() > 1 {
            for &slot in &indices {
                if stack.isEmpty() {
                    break;
                }
                if !self.isItemValidForSlot(slot as i32, stack) {
                    continue;
                }
                let existing = self.getSlot(slot).cloned().unwrap_or(ItemStack::EMPTY);
                if existing.isEmpty() || !existing.canStackWith(stack) {
                    continue;
                }
                let limit = self
                    .slotLimit(slot as i32, stack)
                    .min(stack.getMaxStackSize());
                let capacity = limit - existing.getCount();
                if capacity <= 0 {
                    continue;
                }
                let moved = capacity.min(stack.getCount());
                let mut merged = existing;
                merged.grow(moved);
                stack.shrink(moved);
                let _ = self.putStackInSlot(slot as i32, merged);
                changed = true;
            }
        }
        for &slot in &indices {
            if stack.isEmpty() {
                break;
            }
            if !self.isItemValidForSlot(slot as i32, stack)
                || self
                    .getSlot(slot)
                    .is_some_and(|existing| !existing.isEmpty())
            {
                continue;
            }
            let moved = self
                .slotLimit(slot as i32, stack)
                .min(stack.getMaxStackSize())
                .min(stack.getCount());
            let placed = stack.splitStack(moved);
            let _ = self.putStackInSlot(slot as i32, placed);
            changed = true;
        }
        changed
    }

    pub fn swapWithHotbar(&mut self, slotId: usize, hotbarIndex: usize) -> bool {
        if slotId >= self.slotCount() || hotbarIndex >= 9 {
            return false;
        }
        let hotbarSlot = self.lowerSlotCount() + 27 + hotbarIndex;
        if slotId == hotbarSlot {
            return false;
        }
        let hotbar = self
            .getSlot(hotbarSlot)
            .cloned()
            .unwrap_or(ItemStack::EMPTY);
        if !self.isItemValidForSlot(slotId as i32, &hotbar) {
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
    fn donkey_and_llama_slot_counts_follow_entity_inventory_size() {
        let donkey = HorseInventorySpec {
            entityId: 4,
            kind: HorseInventoryKind::Donkey,
            chested: true,
            chestColumns: 5,
        };
        let llama = HorseInventorySpec {
            entityId: 5,
            kind: HorseInventoryKind::Llama,
            chested: true,
            chestColumns: 3,
        };
        assert_eq!(donkey.lowerSlotCount(), 17);
        assert_eq!(llama.lowerSlotCount(), 11);
    }

    #[test]
    fn equipment_slots_use_concrete_horse_overrides() {
        let horse = ContainerHorseInventory::new(
            1,
            ITextComponent::fromPlainText("Horse"),
            2,
            &InventoryPlayer::default(),
            HorseInventorySpec {
                entityId: 9,
                kind: HorseInventoryKind::Horse,
                chested: false,
                chestColumns: 0,
            },
        )
        .unwrap();
        assert!(horse.isItemValidForSlot(0, &stack(329, 1)));
        assert!(horse.isItemValidForSlot(1, &stack(417, 1)));
        assert!(!horse.isItemValidForSlot(1, &stack(171, 1)));

        let llama = ContainerHorseInventory::new(
            2,
            ITextComponent::fromPlainText("Llama"),
            2,
            &InventoryPlayer::default(),
            HorseInventorySpec {
                entityId: 10,
                kind: HorseInventoryKind::Llama,
                chested: false,
                chestColumns: 1,
            },
        )
        .unwrap();
        assert!(!llama.isItemValidForSlot(0, &stack(329, 1)));
        assert!(llama.isItemValidForSlot(1, &stack(171, 1)));
    }
}
