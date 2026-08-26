use crate::net::minecraft::inventory::EntityEquipmentSlot::EntityEquipmentSlot;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_var_i32, CodecError};

#[derive(Debug, Clone, PartialEq)]
pub struct SPacketEntityEquipment {
    entityID: i32,
    equipmentSlot: EntityEquipmentSlot,
    itemStack: ItemStack,
}

impl SPacketEntityEquipment {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let entityID = read_var_i32(&mut input)?;
        let equipmentSlot = EntityEquipmentSlot::fromNetworkOrdinal(read_var_i32(&mut input)?)?;
        let itemStack = ItemStack::readFromBuffer(&mut input)?;
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread entity-equipment bytes",
                input.len()
            )));
        }
        Ok(Self {
            entityID,
            equipmentSlot,
            itemStack,
        })
    }

    pub const fn getEntityID(&self) -> i32 {
        self.entityID
    }
    pub const fn getEquipmentSlot(&self) -> EntityEquipmentSlot {
        self.equipmentSlot
    }
    pub fn getItemStack(&self) -> &ItemStack {
        &self.itemStack
    }
}
