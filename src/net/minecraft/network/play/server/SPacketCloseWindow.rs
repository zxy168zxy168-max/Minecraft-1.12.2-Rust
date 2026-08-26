use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_u8, CodecError};

/// Protocol-340 port of MCP 1.12.2 `SPacketCloseWindow` (clientbound 0x12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketCloseWindow {
    windowId: u8,
}

impl SPacketCloseWindow {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let windowId = read_u8(&mut input)?;
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread close-window bytes",
                input.len()
            )));
        }
        Ok(Self { windowId })
    }

    pub const fn getWindowId(&self) -> u8 {
        self.windowId
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_server_window_id() {
        let packet = SPacketCloseWindow::readPacketData(&RawPacket::new(0x12, vec![7])).unwrap();
        assert_eq!(packet.getWindowId(), 7);
    }
}
