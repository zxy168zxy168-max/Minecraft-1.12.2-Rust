use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_f32_be, read_u8, CodecError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SPacketChangeGameState {
    state: i32,
    value: f32,
}

impl SPacketChangeGameState {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            state: read_u8(&mut input)? as i32,
            value: read_f32_be(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread change-game-state bytes",
                input.len()
            )));
        }
        Ok(result)
    }

    pub const fn getGameState(&self) -> i32 {
        self.state
    }
    pub const fn getValue(&self) -> f32 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_mode_reason_uses_unsigned_byte_then_float() {
        let mut payload = vec![3];
        payload.extend_from_slice(&1.0_f32.to_bits().to_be_bytes());
        let packet =
            SPacketChangeGameState::readPacketData(&RawPacket::new(0x1E, payload)).unwrap();
        assert_eq!(packet.getGameState(), 3);
        assert_eq!(packet.getValue(), 1.0);
    }
}
