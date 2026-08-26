use crate::net::minecraft::network::datasync::DataSerializers::DataValue;
use crate::net::minecraft::network::datasync::EntityDataManager::EntityDataManager;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_var_i32, CodecError};
#[derive(Debug, Clone, PartialEq)]
pub struct SPacketEntityMetadata {
    entityId: i32,
    dataManagerEntries: Vec<(u8, DataValue)>,
}
impl SPacketEntityMetadata {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            entityId: read_var_i32(&mut input)?,
            dataManagerEntries: EntityDataManager::readEntries(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread entity-metadata bytes",
                input.len()
            )));
        }
        Ok(result)
    }
    pub const fn getEntityId(&self) -> i32 {
        self.entityId
    }
    pub fn getDataManagerEntries(&self) -> &[(u8, DataValue)] {
        &self.dataManagerEntries
    }
}
