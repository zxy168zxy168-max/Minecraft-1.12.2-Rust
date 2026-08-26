use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i32_be, CodecError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketEntityAttach {
    entityId: i32,
    vehicleEntityId: i32,
}

impl SPacketEntityAttach {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            entityId: read_i32_be(&mut input)?,
            vehicleEntityId: read_i32_be(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread entity-attach bytes",
                input.len()
            )));
        }
        Ok(result)
    }

    pub const fn getEntityId(&self) -> i32 {
        self.entityId
    }
    pub const fn getVehicleEntityId(&self) -> i32 {
        self.vehicleEntityId
    }
}
