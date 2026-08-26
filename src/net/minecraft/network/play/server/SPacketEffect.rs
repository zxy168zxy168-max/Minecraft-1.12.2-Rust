use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_bool, read_i32_be, read_i64_be, CodecError,
};
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// MCP 1.12.2 `SPacketEffect` (clientbound play packet 0x21).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketEffect {
    soundType: i32,
    soundPos: BlockPos,
    soundData: i32,
    serverWide: bool,
}

impl SPacketEffect {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let soundType = read_i32_be(&mut input)?;
        let soundPos = BlockPos::from_long(read_i64_be(&mut input)?);
        let soundData = read_i32_be(&mut input)?;
        let serverWide = read_bool(&mut input)?;
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} trailing SPacketEffect bytes",
                input.len()
            )));
        }
        Ok(Self {
            soundType,
            soundPos,
            soundData,
            serverWide,
        })
    }

    pub const fn isSoundServerwide(&self) -> bool {
        self.serverWide
    }
    pub const fn getSoundType(&self) -> i32 {
        self.soundType
    }
    pub const fn getSoundData(&self) -> i32 {
        self.soundData
    }
    pub const fn getSoundPos(&self) -> BlockPos {
        self.soundPos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{write_bool, write_i32_be, write_i64_be};

    #[test]
    fn reads_world_event_payload() {
        let pos = BlockPos::new(-3, 70, 18);
        let mut payload = Vec::new();
        write_i32_be(2001, &mut payload);
        write_i64_be(pos.to_long(), &mut payload);
        write_i32_be(5, &mut payload);
        write_bool(true, &mut payload);
        let packet = SPacketEffect::readPacketData(&RawPacket::new(0x21, payload)).unwrap();
        assert_eq!(packet.getSoundType(), 2001);
        assert_eq!(packet.getSoundPos(), pos);
        assert!(packet.isSoundServerwide());
    }
}
