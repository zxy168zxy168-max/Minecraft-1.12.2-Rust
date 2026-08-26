use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::write_var_i32;
use crate::net::minecraft::util::EnumHand::EnumHand;

/// Protocol-340 port of MCP 1.12.2 `CPacketAnimation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPacketAnimation {
    pub hand: EnumHand,
}

impl CPacketAnimation {
    pub const fn new(handIn: EnumHand) -> Self {
        Self { hand: handIn }
    }

    pub fn writePacketData(self) -> RawPacket {
        let mut payload = Vec::with_capacity(1);
        write_var_i32(self.hand.ordinal(), &mut payload);
        RawPacket::new(0x1D, payload)
    }
}
