use uuid::Uuid;

use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_i64_be, read_string, read_u8, read_uuid, read_var_i32, CodecError,
};
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketSpawnPainting {
    entityID: i32,
    uniqueId: Uuid,
    position: BlockPos,
    facing: EnumFacing,
    title: String,
}

impl SPacketSpawnPainting {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let entityID = read_var_i32(&mut input)?;
        let uniqueId = read_uuid(&mut input)?;
        let title = read_string(&mut input, "SkullAndRoses".len())?;
        let position = BlockPos::from_long(read_i64_be(&mut input)?);
        let horizontal = read_u8(&mut input)?;
        let facing = match horizontal & 3 {
            0 => EnumFacing::South,
            1 => EnumFacing::West,
            2 => EnumFacing::North,
            _ => EnumFacing::East,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread spawn-painting bytes",
                input.len()
            )));
        }
        Ok(Self {
            entityID,
            uniqueId,
            position,
            facing,
            title,
        })
    }

    pub const fn getEntityID(&self) -> i32 {
        self.entityID
    }
    pub const fn getUniqueId(&self) -> Uuid {
        self.uniqueId
    }
    pub const fn getPosition(&self) -> BlockPos {
        self.position
    }
    pub const fn getFacing(&self) -> EnumFacing {
        self.facing
    }
    pub fn getTitle(&self) -> &str {
        &self.title
    }
}
