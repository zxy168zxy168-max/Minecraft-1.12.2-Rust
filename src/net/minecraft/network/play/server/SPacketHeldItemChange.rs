use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i8, CodecError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketHeldItemChange {
    heldItemHotbarIndex: i8,
}

impl SPacketHeldItemChange {
    pub fn new(hotbarIndexIn: i32) -> Self {
        Self {
            heldItemHotbarIndex: hotbarIndexIn as i8,
        }
    }
    pub fn writePacketData(&self) -> RawPacket {
        RawPacket::new(0x3A, vec![self.heldItemHotbarIndex as u8])
    }
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            heldItemHotbarIndex: read_i8(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread held-item-change bytes",
                input.len()
            )));
        }
        Ok(result)
    }

    pub const fn getHeldItemHotbarIndex(&self) -> i32 {
        self.heldItemHotbarIndex as i32
    }
}
