use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::write_i16_be;

/// Protocol-340 port of MCP 1.12.2 `CPacketHeldItemChange`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPacketHeldItemChange {
    slotId: i32,
}

impl CPacketHeldItemChange {
    pub const fn new(slotIdIn: i32) -> Self {
        Self { slotId: slotIdIn }
    }
    pub const fn getSlotId(self) -> i32 {
        self.slotId
    }

    pub fn writePacketData(self) -> RawPacket {
        let mut payload = Vec::with_capacity(2);
        write_i16_be(self.slotId as i16, &mut payload);
        RawPacket::new(0x1A, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_340_id_and_short_payload_match_mcp() {
        let packet = CPacketHeldItemChange::new(8).writePacketData();
        assert_eq!(packet.id, 0x1A);
        assert_eq!(packet.payload, vec![0, 8]);
    }
}
