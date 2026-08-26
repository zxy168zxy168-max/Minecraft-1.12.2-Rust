use uuid::Uuid;

use crate::net::minecraft::network::datasync::DataSerializers::DataValue;
use crate::net::minecraft::network::datasync::EntityDataManager::EntityDataManager;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_f64_be, read_i16_be, read_i8, read_uuid, read_var_i32, CodecError,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SPacketSpawnMob {
    entityId: i32,
    uniqueId: Uuid,
    typeId: i32,
    x: f64,
    y: f64,
    z: f64,
    yaw: i8,
    pitch: i8,
    headPitch: i8,
    velocityX: i16,
    velocityY: i16,
    velocityZ: i16,
    dataManagerEntries: Vec<(u8, DataValue)>,
}

impl SPacketSpawnMob {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            entityId: read_var_i32(&mut input)?,
            uniqueId: read_uuid(&mut input)?,
            typeId: read_var_i32(&mut input)?,
            x: read_f64_be(&mut input)?,
            y: read_f64_be(&mut input)?,
            z: read_f64_be(&mut input)?,
            yaw: read_i8(&mut input)?,
            pitch: read_i8(&mut input)?,
            headPitch: read_i8(&mut input)?,
            velocityX: read_i16_be(&mut input)?,
            velocityY: read_i16_be(&mut input)?,
            velocityZ: read_i16_be(&mut input)?,
            dataManagerEntries: EntityDataManager::readEntries(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread spawn-mob bytes",
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
    pub const fn getEntityType(&self) -> i32 {
        self.typeId
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
    pub const fn getYaw(&self) -> i8 {
        self.yaw
    }
    pub const fn getPitch(&self) -> i8 {
        self.pitch
    }
    pub const fn getHeadPitch(&self) -> i8 {
        self.headPitch
    }
    pub const fn getVelocityX(&self) -> i16 {
        self.velocityX
    }
    pub const fn getVelocityY(&self) -> i16 {
        self.velocityY
    }
    pub const fn getVelocityZ(&self) -> i16 {
        self.velocityZ
    }
    pub fn getDataManagerEntries(&self) -> &[(u8, DataValue)] {
        &self.dataManagerEntries
    }
}
