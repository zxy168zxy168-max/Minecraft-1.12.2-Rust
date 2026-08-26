use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i16_be, read_u8, CodecError};

#[derive(Debug, Clone, PartialEq)]
pub struct SPacketWindowItems {
    windowId: u8,
    itemStacks: Vec<ItemStack>,
}

impl SPacketWindowItems {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let windowId = read_u8(&mut input)?;
        let count = read_i16_be(&mut input)?;
        if count < 0 {
            return Err(CodecError::InvalidData(format!(
                "negative window item count {count}"
            )));
        }
        let mut itemStacks = Vec::with_capacity(count as usize);
        for _ in 0..count {
            itemStacks.push(ItemStack::readFromBuffer(&mut input)?);
        }
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread window-items bytes",
                input.len()
            )));
        }
        Ok(Self {
            windowId,
            itemStacks,
        })
    }

    pub const fn getWindowId(&self) -> u8 {
        self.windowId
    }
    pub fn getItemStacks(&self) -> &[ItemStack] {
        &self.itemStacks
    }
}
