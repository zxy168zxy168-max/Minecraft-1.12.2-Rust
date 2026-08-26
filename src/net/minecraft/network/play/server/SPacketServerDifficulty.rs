use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_u8, CodecError};
use crate::net::minecraft::world::EnumDifficulty::EnumDifficulty;

/// MCP 1.12.2 `SPacketServerDifficulty` (0x0D). In 1.12.2 its wire format is
/// one unsigned difficulty byte; `difficultyLocked` is not serialized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketServerDifficulty {
    difficulty: EnumDifficulty,
    /// Present on the MCP class but deliberately absent from protocol-340 wire data.
    difficultyLocked: bool,
}

impl SPacketServerDifficulty {
    pub const fn new(difficulty: EnumDifficulty, difficultyLocked: bool) -> Self {
        Self {
            difficulty,
            difficultyLocked,
        }
    }
    pub fn writePacketData(&self) -> RawPacket {
        RawPacket::new(0x0D, vec![self.difficulty.getDifficultyId()])
    }
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let difficulty = EnumDifficulty::getDifficultyEnum(read_u8(&mut input)?);
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread server-difficulty bytes",
                input.len()
            )));
        }
        Ok(Self {
            difficulty,
            difficultyLocked: false,
        })
    }

    pub const fn getDifficulty(&self) -> EnumDifficulty {
        self.difficulty
    }
    pub const fn isDifficultyLocked(&self) -> bool {
        self.difficultyLocked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difficulty_decodes_from_single_byte() {
        let packet =
            SPacketServerDifficulty::readPacketData(&RawPacket::new(0x0D, vec![2])).unwrap();
        assert_eq!(packet.getDifficulty(), EnumDifficulty::Normal);
        assert!(!packet.isDifficultyLocked());
        assert!(
            SPacketServerDifficulty::readPacketData(&RawPacket::new(0x0D, vec![2, 1])).is_err()
        );
    }
}
