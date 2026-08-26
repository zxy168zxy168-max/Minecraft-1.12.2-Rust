use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_f32_be, read_var_i32, CodecError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SPacketSetExperience {
    experienceBar: f32,
    totalExperience: i32,
    level: i32,
}

impl SPacketSetExperience {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            experienceBar: read_f32_be(&mut input)?,
            level: read_var_i32(&mut input)?,
            totalExperience: read_var_i32(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread set-experience bytes",
                input.len()
            )));
        }
        Ok(result)
    }

    pub const fn getExperienceBar(&self) -> f32 {
        self.experienceBar
    }
    pub const fn getTotalExperience(&self) -> i32 {
        self.totalExperience
    }
    pub const fn getLevel(&self) -> i32 {
        self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_float_level_then_total_like_mcp() {
        let mut payload = 0.5_f32.to_bits().to_be_bytes().to_vec();
        payload.extend_from_slice(&[12, 100]);
        let packet = SPacketSetExperience::readPacketData(&RawPacket::new(0x40, payload)).unwrap();
        assert!((packet.getExperienceBar() - 0.5).abs() < f32::EPSILON);
        assert_eq!(packet.getLevel(), 12);
        assert_eq!(packet.getTotalExperience(), 100);
    }
}
