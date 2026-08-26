use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::write_f32_be;

/// Protocol-340 port of MCP 1.12.2 `CPacketInput`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CPacketInput {
    pub strafeSpeed: f32,
    pub forwardSpeed: f32,
    pub jumping: bool,
    pub sneaking: bool,
}

impl CPacketInput {
    pub const fn new(strafeSpeed: f32, forwardSpeed: f32, jumping: bool, sneaking: bool) -> Self {
        Self {
            strafeSpeed,
            forwardSpeed,
            jumping,
            sneaking,
        }
    }

    pub fn writePacketData(&self) -> RawPacket {
        let mut payload = Vec::with_capacity(9);
        write_f32_be(self.strafeSpeed, &mut payload);
        write_f32_be(self.forwardSpeed, &mut payload);
        let mut flags = 0_i8;
        if self.jumping {
            flags |= 1;
        }
        if self.sneaking {
            flags |= 2;
        }
        payload.push(flags as u8);
        RawPacket::new(0x16, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_id_and_flag_bits_match_mcp() {
        let packet = CPacketInput::new(1.0, -1.0, true, true).writePacketData();
        assert_eq!(packet.id, 0x16);
        assert_eq!(packet.payload.len(), 9);
        assert_eq!(packet.payload[8], 3);
    }
}
