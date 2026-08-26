use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i32_be, read_i8, CodecError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketEntityStatus {
    entityId: i32,
    logicOpcode: i8,
}

impl SPacketEntityStatus {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            entityId: read_i32_be(&mut input)?,
            logicOpcode: read_i8(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread entity-status bytes",
                input.len()
            )));
        }
        Ok(result)
    }

    pub const fn getEntityId(&self) -> i32 {
        self.entityId
    }
    pub const fn getOpCode(&self) -> i8 {
        self.logicOpcode
    }
}
