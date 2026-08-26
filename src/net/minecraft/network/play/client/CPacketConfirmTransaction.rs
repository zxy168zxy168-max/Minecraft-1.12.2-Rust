use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{write_bool, write_i16_be};

/// Protocol-340 port of MCP 1.12.2 `CPacketConfirmTransaction`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPacketConfirmTransaction {
    pub windowId: i8,
    pub uid: i16,
    pub accepted: bool,
}

impl CPacketConfirmTransaction {
    pub const fn new(windowIdIn: i32, uidIn: i16, acceptedIn: bool) -> Self {
        Self {
            windowId: windowIdIn as i8,
            uid: uidIn,
            accepted: acceptedIn,
        }
    }
    pub fn writePacketData(self) -> RawPacket {
        let mut payload = vec![self.windowId as u8];
        write_i16_be(self.uid, &mut payload);
        write_bool(self.accepted, &mut payload);
        RawPacket::new(0x05, payload)
    }
}
