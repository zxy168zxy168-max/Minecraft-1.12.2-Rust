use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_i64_be, read_nbt_compound, read_u8, CodecError,
};
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// Protocol 340 clientbound 0x09, MCP `SPacketUpdateTileEntity`.
#[derive(Debug, Clone, PartialEq)]
pub struct SPacketUpdateTileEntity {
    blockPosition: BlockPos,
    tileEntityType: u8,
    nbtCompound: NBTTagCompound,
}

impl SPacketUpdateTileEntity {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let blockPosition = BlockPos::from_long(read_i64_be(&mut input)?);
        let tileEntityType = read_u8(&mut input)?;
        let nbtCompound = read_nbt_compound(&mut input)?.ok_or_else(|| {
            CodecError::InvalidData("UpdateTileEntity requires a compound NBT payload".to_owned())
        })?;
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} trailing UpdateTileEntity bytes",
                input.len()
            )));
        }
        Ok(Self {
            blockPosition,
            tileEntityType,
            nbtCompound,
        })
    }

    pub const fn getPos(&self) -> BlockPos {
        self.blockPosition
    }
    pub const fn getTileEntityType(&self) -> u8 {
        self.tileEntityType
    }
    pub const fn getNbtCompound(&self) -> &NBTTagCompound {
        &self.nbtCompound
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::nbt::CompressedStreamTools;

    #[test]
    fn skull_update_packet_retains_position_action_and_nbt() {
        let pos = BlockPos::new(4, 80, -9);
        let mut tag = NBTTagCompound::new();
        tag.setString("id", "minecraft:skull");
        tag.setInteger("x", pos.x);
        tag.setInteger("y", pos.y);
        tag.setInteger("z", pos.z);
        tag.setByte("SkullType", 5);
        tag.setByte("Rot", 3);
        let mut payload = pos.to_long().to_be_bytes().to_vec();
        payload.push(4);
        CompressedStreamTools::writeRoot(&tag, &mut payload).unwrap();
        let decoded =
            SPacketUpdateTileEntity::readPacketData(&RawPacket { id: 0x09, payload }).unwrap();
        assert_eq!(decoded.getPos(), pos);
        assert_eq!(decoded.getTileEntityType(), 4);
        assert_eq!(decoded.getNbtCompound().getByte("SkullType"), 5);
    }
}
