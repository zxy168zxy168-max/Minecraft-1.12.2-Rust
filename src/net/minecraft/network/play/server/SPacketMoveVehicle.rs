use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_f32_be, read_f64_be, CodecError};

/// Protocol-340 port of MCP 1.12.2 `SPacketMoveVehicle`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SPacketMoveVehicle {
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
}

impl SPacketMoveVehicle {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            x: read_f64_be(&mut input)?,
            y: read_f64_be(&mut input)?,
            z: read_f64_be(&mut input)?,
            yaw: read_f32_be(&mut input)?,
            pitch: read_f32_be(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread move-vehicle bytes",
                input.len()
            )));
        }
        Ok(result)
    }

    pub const fn getX(&self) -> f64 {
        self.x
    }
    pub const fn getY(&self) -> f64 {
        self.y
    }
    pub const fn getZ(&self) -> f64 {
        self.z
    }
    pub const fn getYaw(&self) -> f32 {
        self.yaw
    }
    pub const fn getPitch(&self) -> f32 {
        self.pitch
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{write_f32_be, write_f64_be};

    #[test]
    fn reads_exact_vehicle_transform() {
        let mut payload = Vec::new();
        write_f64_be(1.0, &mut payload);
        write_f64_be(2.0, &mut payload);
        write_f64_be(3.0, &mut payload);
        write_f32_be(4.0, &mut payload);
        write_f32_be(5.0, &mut payload);
        let packet = SPacketMoveVehicle::readPacketData(&RawPacket::new(0x29, payload)).unwrap();
        assert_eq!(
            (packet.getX(), packet.getY(), packet.getZ()),
            (1.0, 2.0, 3.0)
        );
        assert_eq!((packet.getYaw(), packet.getPitch()), (4.0, 5.0));
    }
}
