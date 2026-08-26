use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_f64_be, read_i16_be, read_var_i32, CodecError,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SPacketSpawnExperienceOrb {
    entityID: i32,
    posX: f64,
    posY: f64,
    posZ: f64,
    xpValue: i16,
}

impl SPacketSpawnExperienceOrb {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            entityID: read_var_i32(&mut input)?,
            posX: read_f64_be(&mut input)?,
            posY: read_f64_be(&mut input)?,
            posZ: read_f64_be(&mut input)?,
            xpValue: read_i16_be(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread spawn-xp-orb bytes",
                input.len()
            )));
        }
        Ok(result)
    }

    pub const fn getEntityID(&self) -> i32 {
        self.entityID
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
    pub const fn getXPValue(&self) -> i16 {
        self.xpValue
    }
}
