use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i64_be, read_var_i32, CodecError};
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// Protocol-340 clientbound `SPacketUseBed` (`0x30`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketUseBed {
    playerId: i32,
    bedPos: BlockPos,
}

impl SPacketUseBed {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            playerId: read_var_i32(&mut input)?,
            bedPos: BlockPos::from_long(read_i64_be(&mut input)?),
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread use-bed bytes",
                input.len()
            )));
        }
        Ok(result)
    }

    pub const fn getPlayerId(&self) -> i32 {
        self.playerId
    }
    pub const fn getBedPosition(&self) -> BlockPos {
        self.bedPos
    }
}
