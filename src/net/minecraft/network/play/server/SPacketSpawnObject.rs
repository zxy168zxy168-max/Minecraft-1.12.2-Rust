use uuid::Uuid;

use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_f64_be, read_i16_be, read_i32_be, read_i8, read_u8, read_uuid, read_var_i32, CodecError,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SPacketSpawnObject {
    entityId: i32,
    uniqueId: Uuid,
    typeId: i8,
    x: f64,
    y: f64,
    z: f64,
    pitch: i8,
    yaw: i8,
    data: i32,
    speedX: i16,
    speedY: i16,
    speedZ: i16,
}

impl SPacketSpawnObject {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            entityId: read_var_i32(&mut input)?,
            uniqueId: read_uuid(&mut input)?,
            typeId: read_u8(&mut input)? as i8,
            x: read_f64_be(&mut input)?,
            y: read_f64_be(&mut input)?,
            z: read_f64_be(&mut input)?,
            pitch: read_i8(&mut input)?,
            yaw: read_i8(&mut input)?,
            data: read_i32_be(&mut input)?,
            speedX: read_i16_be(&mut input)?,
            speedY: read_i16_be(&mut input)?,
            speedZ: read_i16_be(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread spawn-object bytes",
                input.len()
            )));
        }
        Ok(result)
    }

    pub const fn getEntityID(&self) -> i32 {
        self.entityId
    }
    pub const fn getUniqueId(&self) -> Uuid {
        self.uniqueId
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
    pub const fn getPitch(&self) -> i8 {
        self.pitch
    }
    pub const fn getYaw(&self) -> i8 {
        self.yaw
    }
    pub const fn getData(&self) -> i32 {
        self.data
    }
    pub const fn getSpeedX(&self) -> i16 {
        self.speedX
    }
    pub const fn getSpeedY(&self) -> i16 {
        self.speedY
    }
    pub const fn getSpeedZ(&self) -> i16 {
        self.speedZ
    }
}
