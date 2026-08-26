use uuid::Uuid;

use crate::net::minecraft::network::datasync::DataSerializers::DataValue;
use crate::net::minecraft::network::datasync::EntityDataManager::EntityDataManager;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_f64_be, read_i8, read_uuid, read_var_i32, CodecError,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SPacketSpawnPlayer {
    entityId: i32,
    uniqueId: Uuid,
    x: f64,
    y: f64,
    z: f64,
    yaw: i8,
    pitch: i8,
    dataManagerEntries: Vec<(u8, DataValue)>,
}

impl SPacketSpawnPlayer {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            entityId: read_var_i32(&mut input)?,
            uniqueId: read_uuid(&mut input)?,
            x: read_f64_be(&mut input)?,
            y: read_f64_be(&mut input)?,
            z: read_f64_be(&mut input)?,
            yaw: read_i8(&mut input)?,
            pitch: read_i8(&mut input)?,
            dataManagerEntries: EntityDataManager::readEntries(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread spawn-player bytes",
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
    pub fn getDataManagerEntries(&self) -> &[(u8, DataValue)] {
        &self.dataManagerEntries
    }
}
