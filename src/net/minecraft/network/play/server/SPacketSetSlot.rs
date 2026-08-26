use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i16_be, read_i8, CodecError};

#[derive(Debug, Clone, PartialEq)]
pub struct SPacketSetSlot {
    windowId: i8,
    slot: i16,
    item: ItemStack,
}

impl SPacketSetSlot {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            windowId: read_i8(&mut input)?,
            slot: read_i16_be(&mut input)?,
            item: ItemStack::readFromBuffer(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread set-slot bytes",
                input.len()
            )));
        }
        Ok(result)
    }

    pub const fn getWindowId(&self) -> i8 {
        self.windowId
    }
    pub const fn getSlot(&self) -> i16 {
        self.slot
    }
    pub fn getStack(&self) -> &ItemStack {
        &self.item
    }
}
