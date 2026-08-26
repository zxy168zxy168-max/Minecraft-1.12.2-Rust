use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i8, read_var_i32, CodecError};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketEntityHeadLook {
    entityId: i32,
    yaw: i8,
}
impl SPacketEntityHeadLook {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            entityId: read_var_i32(&mut input)?,
            yaw: read_i8(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread head-look bytes",
                input.len()
            )));
        }
        Ok(result)
    }
    pub const fn getEntityId(&self) -> i32 {
        self.entityId
    }
    pub const fn getYaw(&self) -> i8 {
        self.yaw
    }
}
