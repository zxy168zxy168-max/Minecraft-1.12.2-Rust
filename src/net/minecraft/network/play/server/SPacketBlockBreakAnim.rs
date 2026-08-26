use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_i64_be, read_u8, read_var_i32, CodecError,
};
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// Protocol 340 clientbound packet 0x08, MCP 1.12.2 `SPacketBlockBreakAnim`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketBlockBreakAnim {
    breakerId: i32,
    position: BlockPos,
    progress: i32,
}

impl SPacketBlockBreakAnim {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let breakerId = read_var_i32(&mut input)?;
        let position = BlockPos::from_long(read_i64_be(&mut input)?);
        let progress = read_u8(&mut input)? as i32;
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} trailing BlockBreakAnim bytes",
                input.len()
            )));
        }
        Ok(Self {
            breakerId,
            position,
            progress,
        })
    }

    pub const fn getBreakerId(&self) -> i32 {
        self.breakerId
    }
    pub const fn getPosition(&self) -> BlockPos {
        self.position
    }
    pub const fn getProgress(&self) -> i32 {
        self.progress
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{write_i64_be, write_var_i32};

    #[test]
    fn protocol_340_layout_matches_mcp() {
        let position = BlockPos::new(-12, 64, 37);
        let mut payload = Vec::new();
        write_var_i32(23, &mut payload);
        write_i64_be(position.to_long(), &mut payload);
        payload.push(7);
        let decoded =
            SPacketBlockBreakAnim::readPacketData(&RawPacket::new(0x08, payload)).unwrap();
        assert_eq!(decoded.getBreakerId(), 23);
        assert_eq!(decoded.getPosition(), position);
        assert_eq!(decoded.getProgress(), 7);
    }
}
