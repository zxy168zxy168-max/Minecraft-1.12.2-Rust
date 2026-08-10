use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_bool, read_i32_be, read_string, read_u8, write_bool, write_i32_be, write_string, CodecError};
use crate::net::minecraft::world::EnumDifficulty::EnumDifficulty;
use crate::net::minecraft::world::GameType::GameType;
use crate::net::minecraft::world::WorldType::WorldType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketJoinGame {
    playerId:i32, hardcoreMode:bool, gameType:GameType, dimension:i32,
    difficulty:EnumDifficulty, maxPlayers:u8, worldType:WorldType, reducedDebugInfo:bool,
}
impl SPacketJoinGame {
    pub fn new(playerId:i32, gameType:GameType, hardcoreMode:bool, dimension:i32, difficulty:EnumDifficulty, maxPlayers:u8, worldType:WorldType, reducedDebugInfo:bool) -> Self {
        Self { playerId, hardcoreMode, gameType, dimension, difficulty, maxPlayers, worldType, reducedDebugInfo }
    }
    pub fn writePacketData(&self) -> Result<RawPacket, CodecError> {
        let mut payload=Vec::new();
        write_i32_be(self.playerId,&mut payload);
        let mut game=self.gameType.getID() as u8; if self.hardcoreMode { game|=8; } payload.push(game);
        write_i32_be(self.dimension,&mut payload);
        payload.push(self.difficulty.getDifficultyId());
        payload.push(self.maxPlayers);
        write_string(self.worldType.getWorldTypeName(),16,&mut payload)?;
        write_bool(self.reducedDebugInfo,&mut payload);
        Ok(RawPacket::new(0x23,payload))
    }
    pub fn readPacketData(packet:&RawPacket)->Result<Self,CodecError>{
        let mut input=packet.payload.as_slice();
        let playerId=read_i32_be(&mut input)?;
        let mut game=read_u8(&mut input)?;
        let hardcoreMode=(game&8)==8; game&=!8;
        let gameType=GameType::getByID(i32::from(game));
        let dimension=read_i32_be(&mut input)?;
        let difficulty=EnumDifficulty::getDifficultyEnum(read_u8(&mut input)?);
        let maxPlayers=read_u8(&mut input)?;
        let worldType=WorldType::parseWorldType(&read_string(&mut input,16)?);
        let reducedDebugInfo=read_bool(&mut input)?;
        Ok(Self{playerId,hardcoreMode,gameType,dimension,difficulty,maxPlayers,worldType,reducedDebugInfo})
    }
    pub const fn getPlayerId(&self)->i32{self.playerId}
    pub const fn isHardcoreMode(&self)->bool{self.hardcoreMode}
    pub const fn getGameType(&self)->GameType{self.gameType}
    pub const fn getDimension(&self)->i32{self.dimension}
    pub const fn getDifficulty(&self)->EnumDifficulty{self.difficulty}
    pub const fn getMaxPlayers(&self)->u8{self.maxPlayers}
    pub fn getWorldType(&self)->&WorldType{&self.worldType}
    pub const fn isReducedDebugInfo(&self)->bool{self.reducedDebugInfo}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{write_bool, write_i32_be, write_string};

    #[test]
    fn join_game_writer_round_trips_source_fields() {
        let packet=SPacketJoinGame::new(77,GameType::Survival,false,0,EnumDifficulty::Hard,8,WorldType::Flat,false);
        let raw=packet.writePacketData().unwrap();
        assert_eq!(raw.id,0x23);
        assert_eq!(SPacketJoinGame::readPacketData(&raw).unwrap(),packet);
    }

    #[test]
    fn join_game_reads_hardcore_and_game_type_bits() {
        let mut payload = Vec::new();
        write_i32_be(42, &mut payload);
        payload.push(0x08 | 0x01);
        write_i32_be(-1, &mut payload);
        payload.push(2);
        payload.push(20);
        write_string("default", 16, &mut payload).unwrap();
        write_bool(true, &mut payload);
        let packet = SPacketJoinGame::readPacketData(&RawPacket::new(0x23, payload)).unwrap();
        assert_eq!(packet.getPlayerId(), 42);
        assert!(packet.isHardcoreMode());
        assert_eq!(packet.getGameType(), GameType::Creative);
        assert_eq!(packet.getDimension(), -1);
        assert_eq!(packet.getDifficulty(), EnumDifficulty::Normal);
        assert_eq!(packet.getMaxPlayers(), 20);
        assert!(packet.isReducedDebugInfo());
    }
}
