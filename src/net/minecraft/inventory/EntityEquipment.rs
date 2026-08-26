use crate::net::minecraft::inventory::EntityEquipmentSlot::EntityEquipmentSlot;
use crate::net::minecraft::item::ItemStack::ItemStack;

/// Storage equivalent of the hand/armor iterables implemented by
/// `EntityLivingBase` subclasses in MCP 1.12.2.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityEquipment {
    slots: Vec<ItemStack>,
}

impl Default for EntityEquipment {
    fn default() -> Self {
        Self {
            slots: vec![ItemStack::EMPTY; 6],
        }
    }
}

impl EntityEquipment {
    pub fn setItemStackToSlot(&mut self, slot: EntityEquipmentSlot, stack: ItemStack) {
        self.slots[slot.getSlotIndex()] = stack;
    }

    pub fn getItemStackFromSlot(&self, slot: EntityEquipmentSlot) -> &ItemStack {
        &self.slots[slot.getSlotIndex()]
    }

    pub fn slots(&self) -> &[ItemStack] {
        &self.slots
    }
}
