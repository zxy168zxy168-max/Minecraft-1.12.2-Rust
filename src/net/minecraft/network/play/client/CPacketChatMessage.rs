use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{write_string, CodecError};

/// Protocol 340 serverbound 0x02, matching MCP 1.12.2 `CPacketChatMessage`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CPacketChatMessage {
    message: String,
}

impl CPacketChatMessage {
    pub fn new(messageIn: impl Into<String>) -> Self {
        let messageIn = messageIn.into();
        let message = truncate_utf16(&messageIn, 256);
        Self { message }
    }

    pub fn writePacketData(&self) -> Result<RawPacket, CodecError> {
        let mut payload = Vec::new();
        write_string(&self.message, 256, &mut payload)?;
        Ok(RawPacket::new(0x02, payload))
    }

    pub fn getMessage(&self) -> &str {
        &self.message
    }
}

fn truncate_utf16(value: &str, maximum: usize) -> String {
    let units = value.encode_utf16().take(maximum).collect::<Vec<_>>();
    String::from_utf16_lossy(&units)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_using_java_utf16_units() {
        let packet = CPacketChatMessage::new(format!("{}😀B", "A".repeat(254)));
        assert_eq!(packet.getMessage().encode_utf16().count(), 256);
        assert!(!packet.getMessage().ends_with('B'));
        assert_eq!(packet.writePacketData().unwrap().id, 0x02);
    }
}
