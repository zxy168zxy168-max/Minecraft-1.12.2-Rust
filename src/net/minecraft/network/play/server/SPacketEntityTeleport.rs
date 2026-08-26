use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_bool, read_f64_be, read_i8, read_var_i32, CodecError,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SPacketEntityTeleport {
    entityId: i32,
    posX: f64,
    posY: f64,
    posZ: f64,
    yaw: i8,
    pitch: i8,
    onGround: bool,
}
impl SPacketEntityTeleport {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            entityId: read_var_i32(&mut input)?,
            posX: read_f64_be(&mut input)?,
            posY: read_f64_be(&mut input)?,
            posZ: read_f64_be(&mut input)?,
            yaw: read_i8(&mut input)?,
            pitch: read_i8(&mut input)?,
            onGround: read_bool(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread entity-teleport bytes",
                input.len()
            )));
        }
        Ok(result)
    }
    pub const fn getEntityId(&self) -> i32 {
        self.entityId
    }
    pub const fn getX(&self) -> f64 {
        self.posX
    }
    pub const fn getY(&self) -> f64 {
        self.posY
    }
    pub const fn getZ(&self) -> f64 {
        self.posZ
    }
    pub const fn getYaw(&self) -> i8 {
        self.yaw
    }
    pub const fn getPitch(&self) -> i8 {
        self.pitch
    }
    pub const fn getOnGround(&self) -> bool {
        self.onGround
    }
}
