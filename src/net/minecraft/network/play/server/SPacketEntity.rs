use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_bool, read_i16_be, read_i8, read_var_i32, CodecError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketEntity {
    entityId: i32,
    posX: i16,
    posY: i16,
    posZ: i16,
    yaw: i8,
    pitch: i8,
    onGround: bool,
    rotating: bool,
}

impl SPacketEntity {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let entityId = read_var_i32(&mut input)?;
        let mut result = Self {
            entityId,
            posX: 0,
            posY: 0,
            posZ: 0,
            yaw: 0,
            pitch: 0,
            onGround: false,
            rotating: false,
        };
        match packet.id {
            0x25 => {}
            0x26 => {
                result.posX = read_i16_be(&mut input)?;
                result.posY = read_i16_be(&mut input)?;
                result.posZ = read_i16_be(&mut input)?;
                result.onGround = read_bool(&mut input)?;
            }
            0x27 => {
                result.posX = read_i16_be(&mut input)?;
                result.posY = read_i16_be(&mut input)?;
                result.posZ = read_i16_be(&mut input)?;
                result.yaw = read_i8(&mut input)?;
                result.pitch = read_i8(&mut input)?;
                result.rotating = true;
                result.onGround = read_bool(&mut input)?;
            }
            0x28 => {
                result.yaw = read_i8(&mut input)?;
                result.pitch = read_i8(&mut input)?;
                result.rotating = true;
                result.onGround = read_bool(&mut input)?;
            }
            id => {
                return Err(CodecError::InvalidData(format!(
                    "invalid SPacketEntity packet id {id:#x}"
                )))
            }
        }
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread entity-move bytes",
                input.len()
            )));
        }
        Ok(result)
    }
    pub const fn getEntityId(&self) -> i32 {
        self.entityId
    }
    pub const fn getX(&self) -> i16 {
        self.posX
    }
    pub const fn getY(&self) -> i16 {
        self.posY
    }
    pub const fn getZ(&self) -> i16 {
        self.posZ
    }
    pub const fn getYaw(&self) -> i8 {
        self.yaw
    }
    pub const fn getPitch(&self) -> i8 {
        self.pitch
    }
    pub const fn isRotating(&self) -> bool {
        self.rotating
    }
    pub const fn getOnGround(&self) -> bool {
        self.onGround
    }
}
