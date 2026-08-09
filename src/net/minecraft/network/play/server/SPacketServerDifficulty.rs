use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_u8, CodecError};
use crate::net::minecraft::world::EnumDifficulty::EnumDifficulty;

/// MCP 1.12.2 `SPacketServerDifficulty` (0x0D): world difficulty. The wire
/// format is a single unsigned byte — the `difficultyLocked` field exists on
/// the class but is never read from or written to the network in 1.12.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketServerDifficulty {
    difficulty: EnumDifficulty,
}

impl SPacketServerDifficulty {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let difficulty = EnumDifficulty::getDifficultyEnum(read_u8(&mut input)?);
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread server-difficulty bytes",
                input.len()
            )));
        }
        Ok(Self { difficulty })
    }

    pub const fn getDifficulty(&self) -> EnumDifficulty { self.difficulty }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difficulty_decodes_from_single_byte() {
        let packet = SPacketServerDifficulty::readPacketData(&RawPacket::new(0x0D, vec![2])).unwrap();
        assert_eq!(packet.getDifficulty(), EnumDifficulty::Normal);
        // No trailing lock byte in 1.12.2: a second byte must be rejected.
        assert!(SPacketServerDifficulty::readPacketData(&RawPacket::new(0x0D, vec![2, 1])).is_err());
    }
}

