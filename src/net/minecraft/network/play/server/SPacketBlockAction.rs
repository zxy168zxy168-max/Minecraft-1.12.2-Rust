use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_i64_be, read_u8, read_var_i32, CodecError,
};
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// Protocol-340 clientbound `SPacketBlockAction` (`0x0A`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketBlockAction {
    blockPosition: BlockPos,
    instrument: u8,
    pitch: u8,
    blockType: i32,
}

impl SPacketBlockAction {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let blockPosition = BlockPos::from_long(read_i64_be(&mut input)?);
        let instrument = read_u8(&mut input)?;
        let pitch = read_u8(&mut input)?;
        let blockType = read_var_i32(&mut input)?;
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} trailing BlockAction bytes",
                input.len(),
            )));
        }
        Ok(Self {
            blockPosition,
            instrument,
            pitch,
            blockType,
        })
    }
    pub const fn getBlockPosition(&self) -> BlockPos {
        self.blockPosition
    }
    pub const fn getData1(&self) -> i32 {
        self.instrument as i32
    }
    pub const fn getData2(&self) -> i32 {
        self.pitch as i32
    }
    pub const fn getBlockTypeId(&self) -> i32 {
        self.blockType
    }
}
