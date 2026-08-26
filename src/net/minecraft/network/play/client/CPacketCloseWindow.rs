use crate::net::minecraft::network::Packet::RawPacket;

/// Protocol-340 port of MCP 1.12.2 `CPacketCloseWindow`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPacketCloseWindow {
    pub windowId: i8,
}

impl CPacketCloseWindow {
    pub const fn new(windowIdIn: i32) -> Self {
        Self {
            windowId: windowIdIn as i8,
        }
    }
    pub fn writePacketData(self) -> RawPacket {
        RawPacket::new(0x08, vec![self.windowId as u8])
    }
}
