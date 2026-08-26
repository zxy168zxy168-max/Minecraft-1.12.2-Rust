use crate::net::minecraft::entity::Entity::Entity;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{write_f32_be, write_f64_be};

/// Protocol-340 port of MCP 1.12.2 `CPacketVehicleMove`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CPacketVehicleMove {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

impl CPacketVehicleMove {
    pub const fn new(x: f64, y: f64, z: f64, yaw: f32, pitch: f32) -> Self {
        Self {
            x,
            y,
            z,
            yaw,
            pitch,
        }
    }

    pub fn fromEntity(entity: &Entity) -> Self {
        Self::new(
            entity.posX,
            entity.posY,
            entity.posZ,
            entity.rotationYaw,
            entity.rotationPitch,
        )
    }

    pub fn writePacketData(&self) -> RawPacket {
        let mut payload = Vec::with_capacity(32);
        write_f64_be(self.x, &mut payload);
        write_f64_be(self.y, &mut payload);
        write_f64_be(self.z, &mut payload);
        write_f32_be(self.yaw, &mut payload);
        write_f32_be(self.pitch, &mut payload);
        RawPacket::new(0x10, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_id_and_payload_size_match_mcp() {
        let packet = CPacketVehicleMove::new(1.0, 2.0, 3.0, 4.0, 5.0).writePacketData();
        assert_eq!(packet.id, 0x10);
        assert_eq!(packet.payload.len(), 32);
    }
}
