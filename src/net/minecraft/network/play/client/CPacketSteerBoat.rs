use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::write_bool;

/// Protocol-340 port of MCP 1.12.2 `CPacketSteerBoat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPacketSteerBoat {
    pub left: bool,
    pub right: bool,
}

impl CPacketSteerBoat {
    pub const fn new(left: bool, right: bool) -> Self {
        Self { left, right }
    }

    pub fn writePacketData(&self) -> RawPacket {
        let mut payload = Vec::with_capacity(2);
        write_bool(self.left, &mut payload);
        write_bool(self.right, &mut payload);
        RawPacket::new(0x11, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_id_and_two_booleans_match_mcp() {
        let packet = CPacketSteerBoat::new(true, false).writePacketData();
        assert_eq!(packet.id, 0x11);
        assert_eq!(packet.payload, vec![1, 0]);
    }
}
