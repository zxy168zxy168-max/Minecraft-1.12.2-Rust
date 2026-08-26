use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i32_be, read_string, read_u8, CodecError};
use crate::net::minecraft::world::EnumDifficulty::EnumDifficulty;
use crate::net::minecraft::world::GameType::GameType;
use crate::net::minecraft::world::WorldType::WorldType;

/// Clientbound Play 0x35 in protocol 340.
///
/// MCP 1.12.2 `SPacketRespawn`: the dimension is a fixed signed 32-bit
/// integer, followed by difficulty, game mode and a 16-character world type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketRespawn {
    dimensionId: i32,
    difficulty: EnumDifficulty,
    gameType: GameType,
    worldType: WorldType,
}

impl SPacketRespawn {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let dimensionId = read_i32_be(&mut input)?;
        let difficulty = EnumDifficulty::getDifficultyEnum(read_u8(&mut input)?);
        let gameType = GameType::getByID(i32::from(read_u8(&mut input)?));
        let worldType = WorldType::parseWorldType(&read_string(&mut input, 16)?);
        Ok(Self {
            dimensionId,
            difficulty,
            gameType,
            worldType,
        })
    }

    pub const fn getDimensionID(&self) -> i32 {
        self.dimensionId
    }
    pub const fn getDifficulty(&self) -> EnumDifficulty {
        self.difficulty
    }
    pub const fn getGameType(&self) -> GameType {
        self.gameType
    }
    pub fn getWorldType(&self) -> &WorldType {
        &self.worldType
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{write_i32_be, write_string};

    #[test]
    fn respawn_reads_protocol_340_field_order() {
        let mut payload = Vec::new();
        write_i32_be(-1, &mut payload);
        payload.push(EnumDifficulty::Hard.getDifficultyId());
        payload.push(GameType::Adventure.getID() as u8);
        write_string("default", 16, &mut payload).unwrap();

        let packet = SPacketRespawn::readPacketData(&RawPacket::new(0x35, payload)).unwrap();
        assert_eq!(packet.getDimensionID(), -1);
        assert_eq!(packet.getDifficulty(), EnumDifficulty::Hard);
        assert_eq!(packet.getGameType(), GameType::Adventure);
        assert_eq!(packet.getWorldType().getWorldTypeName(), "default");
    }
}
