use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::inventory::ContainerPlayer::ContainerPlayer;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::PacketBuffer::CodecError;
use crate::net::minecraft::nbt::NBTBase::NBTBase;
use crate::net::minecraft::nbt::NBTTagList::NBTTagList;

/// Gameplay-bearing inventory layout from MCP 1.12.2 `InventoryPlayer`.
#[derive(Debug, Clone, PartialEq)]
pub struct InventoryPlayer {
    pub mainInventory: Vec<ItemStack>,
    pub armorInventory: Vec<ItemStack>,
    pub offHandInventory: Vec<ItemStack>,
    /// Index of the selected hotbar slot, always 0..8 when set by a valid packet.
    pub currentItem: i32,
    /// Cursor stack used by `SPacketSetSlot(windowId = -1)`.
    itemStack: ItemStack,
}

impl Default for InventoryPlayer {
    fn default() -> Self {
        Self {
            mainInventory: vec![ItemStack::EMPTY; 36],
            armorInventory: vec![ItemStack::EMPTY; 4],
            offHandInventory: vec![ItemStack::EMPTY; 1],
            currentItem: 0,
            itemStack: ItemStack::EMPTY,
        }
    }
}

impl InventoryPlayer {
    /// MCP `InventoryPlayer#writeToNBT`: main slots 0..35, armor 100..103
    /// and offhand 150.
    pub fn writeToNBT(&self, mut list: NBTTagList) -> NBTTagList {
        for (index, stack) in self.mainInventory.iter().enumerate() {
            if stack.isEmpty() { continue; }
            let mut tag=crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound::new();
            tag.setByte("Slot", index as i8); stack.writeToNBT(&mut tag); list.appendTag(NBTBase::Compound(tag));
        }
        for (index, stack) in self.armorInventory.iter().enumerate() {
            if stack.isEmpty() { continue; }
            let mut tag=crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound::new();
            tag.setByte("Slot", (index+100) as i8); stack.writeToNBT(&mut tag); list.appendTag(NBTBase::Compound(tag));
        }
        for (index, stack) in self.offHandInventory.iter().enumerate() {
            if stack.isEmpty() { continue; }
            let mut tag=crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound::new();
            tag.setByte("Slot", (index+150) as i8); stack.writeToNBT(&mut tag); list.appendTag(NBTBase::Compound(tag));
        }
        list
    }

    /// MCP `InventoryPlayer#readFromNBT`. Fixed-size NonNullLists are reset to
    /// EMPTY and only valid vanilla slot ranges are accepted.
    pub fn readFromNBT(&mut self, list: &NBTTagList) {
        self.mainInventory.fill(ItemStack::EMPTY); self.armorInventory.fill(ItemStack::EMPTY); self.offHandInventory.fill(ItemStack::EMPTY);
        for index in 0..list.tagCount() {
            let tag=list.getCompoundTagAt(index);
            let slot=(tag.getByte("Slot") as u8) as usize;
            let stack=ItemStack::fromNBT(&tag); if stack.isEmpty(){continue;}
            if slot < self.mainInventory.len(){self.mainInventory[slot]=stack;}
            else if (100..100+self.armorInventory.len()).contains(&slot){self.armorInventory[slot-100]=stack;}
            else if (150..150+self.offHandInventory.len()).contains(&slot){self.offHandInventory[slot-150]=stack;}
        }
    }

    pub const fn getHotbarSize() -> usize { 9 }

    pub const fn isHotbar(index: i32) -> bool { index >= 0 && index < 9 }

    pub fn getCurrentItem(&self) -> &ItemStack {
        if Self::isHotbar(self.currentItem) {
            &self.mainInventory[self.currentItem as usize]
        } else {
            &ItemStack::EMPTY
        }
    }

    /// Direct port of MCP `InventoryPlayer.getStrVsBlock`.
    pub fn getStrVsBlock(&self, state: IBlockState) -> f32 {
        let stack = self.getCurrentItem();
        if stack.isEmpty() { 1.0 } else { stack.getStrVsBlock(state) }
    }

    /// Direct port of MCP `InventoryPlayer.canHarvestBlock`.
    pub fn canHarvestBlock(&self, state: IBlockState) -> bool {
        state.getBlock().isToolNotRequired() || self.getCurrentItem().canHarvestBlock(state)
    }

    pub fn setCurrentItem(&mut self, index: i32) -> Result<(), CodecError> {
        if !Self::isHotbar(index) {
            return Err(CodecError::InvalidData(format!(
                "held hotbar index {index} outside 0..8"
            )));
        }
        self.currentItem = index;
        Ok(())
    }

    /// Direct port of MCP 1.12.2 `InventoryPlayer.changeCurrentItem`.
    /// Positive wheel deltas select the previous slot and negative deltas the
    /// next slot, matching LWJGL `Mouse.getEventDWheel`.
    pub fn changeCurrentItem(&mut self, mut direction: i32) {
        if direction > 0 { direction = 1; }
        if direction < 0 { direction = -1; }
        self.currentItem -= direction;
        while self.currentItem < 0 { self.currentItem += Self::getHotbarSize() as i32; }
        while self.currentItem >= Self::getHotbarSize() as i32 {
            self.currentItem -= Self::getHotbarSize() as i32;
        }
    }

    pub fn setItemStack(&mut self, stack: ItemStack) { self.itemStack = stack; }
    pub fn getItemStack(&self) -> &ItemStack { &self.itemStack }

    /// MCP `InventoryPlayer.setInventorySlotContents` concatenates main,
    /// armor and offhand inventories in exactly this order.
    pub fn setInventorySlotContents(&mut self, index: i32, stack: ItemStack) -> Result<(), CodecError> {
        let mut index = usize::try_from(index).map_err(|_| {
            CodecError::InvalidData(format!("negative InventoryPlayer slot {index}"))
        })?;
        if index < self.mainInventory.len() {
            self.mainInventory[index] = stack;
            return Ok(());
        }
        index -= self.mainInventory.len();
        if index < self.armorInventory.len() {
            self.armorInventory[index] = stack;
            return Ok(());
        }
        index -= self.armorInventory.len();
        if index < self.offHandInventory.len() {
            self.offHandInventory[index] = stack;
            return Ok(());
        }
        Err(CodecError::InvalidData(format!(
            "InventoryPlayer slot outside 0..40"
        )))
    }

    pub fn getStackInSlot(&self, index: i32) -> Option<&ItemStack> {
        let mut index = usize::try_from(index).ok()?;
        if index < self.mainInventory.len() { return self.mainInventory.get(index); }
        index -= self.mainInventory.len();
        if index < self.armorInventory.len() { return self.armorInventory.get(index); }
        index -= self.armorInventory.len();
        self.offHandInventory.get(index)
    }

    /// Mirrors the player-inventory-backed slots in `ContainerPlayer` into
    /// `InventoryPlayer`; crafting slots remain owned by the container.
    pub fn syncFromContainerPlayer(&mut self, container: &ContainerPlayer) {
        // armor slots are HEAD, CHEST, LEGS, FEET in ContainerPlayer 5..8,
        // while InventoryPlayer armor order is FEET, LEGS, CHEST, HEAD.
        for (container_slot, armor_index) in [(5, 3), (6, 2), (7, 1), (8, 0)] {
            if let Some(stack) = container.getSlot(container_slot) {
                self.armorInventory[armor_index] = stack.clone();
            }
        }
        for container_slot in 9..36 {
            if let Some(stack) = container.getSlot(container_slot) {
                self.mainInventory[container_slot] = stack.clone();
            }
        }
        for container_slot in 36..45 {
            if let Some(stack) = container.getSlot(container_slot) {
                self.mainInventory[container_slot - 36] = stack.clone();
            }
        }
        if let Some(stack) = container.getSlot(45) {
            self.offHandInventory[0] = stack.clone();
        }
    }

    pub fn applyContainerPlayerSlot(&mut self, slot: i32, stack: ItemStack) -> Result<(), CodecError> {
        match slot {
            5 => self.armorInventory[3] = stack,
            6 => self.armorInventory[2] = stack,
            7 => self.armorInventory[1] = stack,
            8 => self.armorInventory[0] = stack,
            9..=35 => self.mainInventory[slot as usize] = stack,
            36..=44 => self.mainInventory[(slot - 36) as usize] = stack,
            45 => self.offHandInventory[0] = stack,
            0..=4 => {}, // crafting result/matrix are not InventoryPlayer slots
            _ => return Err(CodecError::InvalidData(format!("invalid ContainerPlayer slot {slot}"))),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(id: i16) -> ItemStack {
        ItemStack { itemId: id, count: 1, itemDamage: 0, tagCompound: None }
    }

    #[test]
    fn container_hotbar_and_armor_map_to_inventory_player() {
        let mut inventory = InventoryPlayer::default();
        inventory.applyContainerPlayerSlot(36, stack(1)).unwrap();
        inventory.applyContainerPlayerSlot(5, stack(2)).unwrap();
        inventory.applyContainerPlayerSlot(45, stack(3)).unwrap();
        assert_eq!(inventory.mainInventory[0].itemId, 1);
        assert_eq!(inventory.armorInventory[3].itemId, 2);
        assert_eq!(inventory.offHandInventory[0].itemId, 3);
    }

    #[test]
    fn wheel_direction_and_wrap_match_mcp() {
        let mut inventory = InventoryPlayer::default();
        inventory.currentItem = 0;
        inventory.changeCurrentItem(120);
        assert_eq!(inventory.currentItem, 8);
        inventory.changeCurrentItem(-120);
        assert_eq!(inventory.currentItem, 0);
    }
}
