use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_i64_be, read_var_i32, write_i64_be, write_var_i32, CodecError,
};
use crate::net::minecraft::util::math::BlockPos::BlockPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketBlockChange {
    blockPosition: BlockPos,
    blockState: IBlockState,
}
impl SPacketBlockChange {
    pub const fn new(blockPosition: BlockPos, blockState: IBlockState) -> Self {
        Self {
            blockPosition,
            blockState,
        }
    }
    pub fn writePacketData(&self) -> RawPacket {
        let mut payload = Vec::with_capacity(12);
        write_i64_be(self.blockPosition.to_long(), &mut payload);
        write_var_i32(self.blockState.getGlobalStateId(), &mut payload);
        RawPacket::new(0x0B, payload)
    }
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let blockPosition = BlockPos::from_long(read_i64_be(&mut input)?);
        let blockState = IBlockState::fromGlobalStateId(read_var_i32(&mut input)?);
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} trailing BlockChange bytes",
                input.len()
            )));
        }
        Ok(Self {
            blockPosition,
            blockState,
        })
    }
    pub const fn getBlockState(&self) -> IBlockState {
        self.blockState
    }
    pub const fn getBlockPosition(&self) -> BlockPos {
        self.blockPosition
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protocol_340_write_read_round_trip() {
        let original = SPacketBlockChange::new(
            BlockPos::new(-17, 64, 31),
            IBlockState::fromGlobalStateId((57 << 4) | 3),
        );
        let raw = original.writePacketData();
        assert_eq!(raw.id, 0x0B);
        assert_eq!(SPacketBlockChange::readPacketData(&raw).unwrap(), original);
    }
}
