use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_var_i32, CodecError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketSetPassengers {
    entityId: i32,
    passengerIds: Vec<i32>,
}

impl SPacketSetPassengers {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let entityId = read_var_i32(&mut input)?;
        // MCP `PacketBuffer.readVarIntArray()` passes the current readable-byte
        // count as its maximum array length before consuming the length VarInt.
        let maxLength = input.len();
        let count = read_var_i32(&mut input)?;
        if count < 0 || count as usize > maxLength {
            return Err(CodecError::InvalidData(format!(
                "VarIntArray with size {count} is bigger than allowed {maxLength}"
            )));
        }
        let mut passengerIds = Vec::with_capacity(count as usize);
        for _ in 0..count {
            passengerIds.push(read_var_i32(&mut input)?);
        }
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread set-passengers bytes",
                input.len()
            )));
        }
        Ok(Self {
            entityId,
            passengerIds,
        })
    }

    pub const fn getEntityId(&self) -> i32 {
        self.entityId
    }
    pub fn getPassengerIds(&self) -> &[i32] {
        &self.passengerIds
    }
}
