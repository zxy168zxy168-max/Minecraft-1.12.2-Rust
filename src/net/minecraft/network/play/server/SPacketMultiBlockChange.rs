use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_i32_be, read_u16_be, read_var_i32, CodecError,
};
use crate::net::minecraft::util::math::BlockPos::BlockPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockUpdateData {
    offset: u16,
    blockState: IBlockState,
    chunkX: i32,
    chunkZ: i32,
}
impl BlockUpdateData {
    pub const fn getOffset(&self) -> u16 {
        self.offset
    }
    pub const fn getBlockState(&self) -> IBlockState {
        self.blockState
    }
    pub const fn getPos(&self) -> BlockPos {
        BlockPos::new(
            (self.chunkX << 4) + ((self.offset >> 12) & 15) as i32,
            (self.offset & 255) as i32,
            (self.chunkZ << 4) + ((self.offset >> 8) & 15) as i32,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketMultiBlockChange {
    chunkX: i32,
    chunkZ: i32,
    changedBlocks: Vec<BlockUpdateData>,
}
impl SPacketMultiBlockChange {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let chunkX = read_i32_be(&mut input)?;
        let chunkZ = read_i32_be(&mut input)?;
        let count = read_var_i32(&mut input)?;
        if count < 0 {
            return Err(CodecError::NegativeLength(count));
        }
        let mut changedBlocks = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let offset = read_u16_be(&mut input)?;
            let blockState = IBlockState::fromGlobalStateId(read_var_i32(&mut input)?);
            changedBlocks.push(BlockUpdateData {
                offset,
                blockState,
                chunkX,
                chunkZ,
            });
        }
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} trailing MultiBlockChange bytes",
                input.len()
            )));
        }
        Ok(Self {
            chunkX,
            chunkZ,
            changedBlocks,
        })
    }
    pub fn getChangedBlocks(&self) -> &[BlockUpdateData] {
        &self.changedBlocks
    }
    pub const fn getChunkX(&self) -> i32 {
        self.chunkX
    }
    pub const fn getChunkZ(&self) -> i32 {
        self.chunkZ
    }
}
