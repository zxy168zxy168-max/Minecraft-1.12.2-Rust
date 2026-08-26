use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_f32_be, read_var_i32, CodecError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SPacketUpdateHealth {
    health: f32,
    foodLevel: i32,
    saturationLevel: f32,
}

impl SPacketUpdateHealth {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            health: read_f32_be(&mut input)?,
            foodLevel: read_var_i32(&mut input)?,
            saturationLevel: read_f32_be(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread update-health bytes",
                input.len()
            )));
        }
        Ok(result)
    }

    pub const fn getHealth(&self) -> f32 {
        self.health
    }
    pub const fn getFoodLevel(&self) -> i32 {
        self.foodLevel
    }
    pub const fn getSaturationLevel(&self) -> f32 {
        self.saturationLevel
    }
}
