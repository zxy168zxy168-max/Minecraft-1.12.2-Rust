use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_f64_be, read_i8, read_var_i32, CodecError,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SPacketSpawnGlobalEntity {
    entityId: i32,
    typeId: i8,
    x: f64,
    y: f64,
    z: f64,
}

impl SPacketSpawnGlobalEntity {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            entityId: read_var_i32(&mut input)?,
            typeId: read_i8(&mut input)?,
            x: read_f64_be(&mut input)?,
            y: read_f64_be(&mut input)?,
            z: read_f64_be(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread spawn-global-entity bytes",
                input.len()
            )));
        }
        Ok(result)
    }

    pub const fn getEntityId(&self) -> i32 {
        self.entityId
    }
    pub const fn getType(&self) -> i32 {
        self.typeId as i32
    }
    pub const fn getX(&self) -> f64 {
        self.x
    }
    pub const fn getY(&self) -> f64 {
        self.y
    }
    pub const fn getZ(&self) -> f64 {
        self.z
    }
}
