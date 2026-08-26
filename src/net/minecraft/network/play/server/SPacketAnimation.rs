use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_u8, read_var_i32, CodecError};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketAnimation {
    entityId: i32,
    animationType: u8,
}
impl SPacketAnimation {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            entityId: read_var_i32(&mut input)?,
            animationType: read_u8(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread animation bytes",
                input.len()
            )));
        }
        Ok(result)
    }
    pub const fn getEntityID(&self) -> i32 {
        self.entityId
    }
    pub const fn getAnimationType(&self) -> u8 {
        self.animationType
    }
}
