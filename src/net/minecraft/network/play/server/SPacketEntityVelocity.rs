use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i16_be, read_var_i32, CodecError};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketEntityVelocity {
    entityID: i32,
    motionX: i16,
    motionY: i16,
    motionZ: i16,
}
impl SPacketEntityVelocity {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            entityID: read_var_i32(&mut input)?,
            motionX: read_i16_be(&mut input)?,
            motionY: read_i16_be(&mut input)?,
            motionZ: read_i16_be(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread velocity bytes",
                input.len()
            )));
        }
        Ok(result)
    }
    pub const fn getEntityID(&self) -> i32 {
        self.entityID
    }
    pub const fn getMotionX(&self) -> i16 {
        self.motionX
    }
    pub const fn getMotionY(&self) -> i16 {
        self.motionY
    }
    pub const fn getMotionZ(&self) -> i16 {
        self.motionZ
    }
}
