use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i64_be,write_i64_be,CodecError};
use crate::net::minecraft::util::math::BlockPos::BlockPos;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct SPacketSpawnPosition { spawnBlockPos:BlockPos }
impl SPacketSpawnPosition {
    pub const fn new(pos:BlockPos)->Self{Self{spawnBlockPos:pos}}
    pub fn readPacketData(packet:&RawPacket)->Result<Self,CodecError>{let mut input=packet.payload.as_slice();let pos=BlockPos::from_long(read_i64_be(&mut input)?);if !input.is_empty(){return Err(CodecError::InvalidData(format!("{} unread spawn-position bytes",input.len())));}Ok(Self::new(pos))}
    pub fn writePacketData(&self)->RawPacket{let mut payload=Vec::new();write_i64_be(self.spawnBlockPos.to_long(),&mut payload);RawPacket::new(0x46,payload)}
    pub const fn getSpawnPos(&self)->BlockPos{self.spawnBlockPos}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_position_round_trips_vanilla_blockpos_wire_format() {
        let packet = SPacketSpawnPosition::new(BlockPos::new(-12345, 255, 67890));
        let raw = packet.writePacketData();
        assert_eq!(raw.id, 0x46);
        assert_eq!(SPacketSpawnPosition::readPacketData(&raw).unwrap(), packet);
    }
}
