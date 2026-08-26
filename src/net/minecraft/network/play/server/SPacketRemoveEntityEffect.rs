use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i8, read_var_i32, CodecError};

/// Clientbound Play 0x33 in protocol 340.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketRemoveEntityEffect {
    entityId: i32,
    potionId: u8,
}

impl SPacketRemoveEntityEffect {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            entityId: read_var_i32(&mut input)?,
            potionId: read_i8(&mut input)? as u8,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread remove-entity-effect bytes",
                input.len()
            )));
        }
        Ok(result)
    }

    pub const fn getEntityId(&self) -> i32 {
        self.entityId
    }
    pub const fn getPotionId(&self) -> u8 {
        self.potionId
    }
}
