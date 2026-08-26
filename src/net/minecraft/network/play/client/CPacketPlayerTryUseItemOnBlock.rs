use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{write_f32_be, write_i64_be, write_var_i32};
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::EnumHand::EnumHand;

/// Protocol-340 port of MCP 1.12.2 `CPacketPlayerTryUseItemOnBlock`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CPacketPlayerTryUseItemOnBlock {
    pub position: BlockPos,
    pub placedBlockDirection: EnumFacing,
    pub hand: EnumHand,
    pub facingX: f32,
    pub facingY: f32,
    pub facingZ: f32,
}

impl CPacketPlayerTryUseItemOnBlock {
    pub const fn new(
        posIn: BlockPos,
        placedBlockDirectionIn: EnumFacing,
        handIn: EnumHand,
        facingXIn: f32,
        facingYIn: f32,
        facingZIn: f32,
    ) -> Self {
        Self {
            position: posIn,
            placedBlockDirection: placedBlockDirectionIn,
            hand: handIn,
            facingX: facingXIn,
            facingY: facingYIn,
            facingZ: facingZIn,
        }
    }

    pub fn writePacketData(self) -> RawPacket {
        let mut payload = Vec::with_capacity(24);
        write_i64_be(self.position.to_long(), &mut payload);
        write_var_i32(self.placedBlockDirection.index(), &mut payload);
        write_var_i32(self.hand.ordinal(), &mut payload);
        write_f32_be(self.facingX, &mut payload);
        write_f32_be(self.facingY, &mut payload);
        write_f32_be(self.facingZ, &mut payload);
        RawPacket::new(0x1F, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_340_use_on_block_packet_id_matches_registry() {
        assert_eq!(
            CPacketPlayerTryUseItemOnBlock::new(
                BlockPos::ORIGIN,
                EnumFacing::Up,
                EnumHand::MainHand,
                0.5,
                1.0,
                0.5,
            )
            .writePacketData()
            .id,
            0x1F,
        );
    }
}
