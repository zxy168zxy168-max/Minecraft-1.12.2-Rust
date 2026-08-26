use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::write_var_i32;
use crate::net::minecraft::util::EnumHand::EnumHand;

/// Protocol-340 port of MCP 1.12.2 `CPacketPlayerTryUseItem`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPacketPlayerTryUseItem {
    pub hand: EnumHand,
}

impl CPacketPlayerTryUseItem {
    pub const fn new(handIn: EnumHand) -> Self {
        Self { hand: handIn }
    }

    pub fn writePacketData(self) -> RawPacket {
        let mut payload = Vec::with_capacity(1);
        write_var_i32(self.hand.ordinal(), &mut payload);
        RawPacket::new(0x20, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_340_use_item_packet_id_matches_registry() {
        assert_eq!(
            CPacketPlayerTryUseItem::new(EnumHand::OffHand)
                .writePacketData()
                .id,
            0x20
        );
    }
}
