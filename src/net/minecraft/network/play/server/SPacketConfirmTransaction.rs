use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_bool, read_i16_be, read_u8, CodecError};

/// Protocol-340 port of MCP 1.12.2 `SPacketConfirmTransaction`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketConfirmTransaction {
    windowId: u8,
    actionNumber: i16,
    accepted: bool,
}

impl SPacketConfirmTransaction {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            windowId: read_u8(&mut input)?,
            actionNumber: read_i16_be(&mut input)?,
            accepted: read_bool(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread confirm-transaction bytes",
                input.len()
            )));
        }
        Ok(result)
    }
    pub const fn getWindowId(&self) -> i32 {
        self.windowId as i32
    }
    pub const fn getActionNumber(&self) -> i16 {
        self.actionNumber
    }
    pub const fn wasAccepted(&self) -> bool {
        self.accepted
    }
}
