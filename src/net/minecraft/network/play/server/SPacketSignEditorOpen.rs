use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i64_be, CodecError};
use crate::net::minecraft::util::math::BlockPos::BlockPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketSignEditorOpen {
    signPosition: BlockPos,
}

impl SPacketSignEditorOpen {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let signPosition = BlockPos::from_long(read_i64_be(&mut input)?);
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} trailing SignEditorOpen bytes",
                input.len()
            )));
        }
        Ok(Self { signPosition })
    }

    pub const fn getSignPosition(&self) -> BlockPos {
        self.signPosition
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_block_position_round_trips() {
        let pos = BlockPos::new(-12, 70, 345);
        let packet = RawPacket {
            id: 0x2A,
            payload: pos.to_long().to_be_bytes().to_vec(),
        };
        assert_eq!(
            SPacketSignEditorOpen::readPacketData(&packet)
                .unwrap()
                .getSignPosition(),
            pos
        );
    }
}
