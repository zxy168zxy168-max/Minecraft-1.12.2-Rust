use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{write_bool, write_f32_be, write_f64_be};

/// Protocol-340 port of MCP `CPacketPlayer` and its three concrete movement
/// variants. Packet ids are the 1.12.2 serverbound Play ids.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CPacketPlayer {
    pub onGround: bool,
}

impl CPacketPlayer {
    pub const fn new(onGroundIn: bool) -> Self {
        Self {
            onGround: onGroundIn,
        }
    }

    pub fn writePacketData(&self) -> RawPacket {
        let mut payload = Vec::with_capacity(1);
        write_bool(self.onGround, &mut payload);
        RawPacket::new(0x0C, payload)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub onGround: bool,
}

impl Position {
    pub const fn new(x: f64, y: f64, z: f64, onGround: bool) -> Self {
        Self { x, y, z, onGround }
    }

    pub fn writePacketData(&self) -> RawPacket {
        let mut payload = Vec::with_capacity(25);
        write_f64_be(self.x, &mut payload);
        write_f64_be(self.y, &mut payload);
        write_f64_be(self.z, &mut payload);
        write_bool(self.onGround, &mut payload);
        RawPacket::new(0x0D, payload)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionRotation {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub onGround: bool,
}

impl PositionRotation {
    pub const fn new(x: f64, y: f64, z: f64, yaw: f32, pitch: f32, onGround: bool) -> Self {
        Self {
            x,
            y,
            z,
            yaw,
            pitch,
            onGround,
        }
    }

    pub fn writePacketData(&self) -> RawPacket {
        let mut payload = Vec::with_capacity(33);
        write_f64_be(self.x, &mut payload);
        write_f64_be(self.y, &mut payload);
        write_f64_be(self.z, &mut payload);
        write_f32_be(self.yaw, &mut payload);
        write_f32_be(self.pitch, &mut payload);
        write_bool(self.onGround, &mut payload);
        RawPacket::new(0x0E, payload)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
    pub onGround: bool,
}

impl Rotation {
    pub const fn new(yaw: f32, pitch: f32, onGround: bool) -> Self {
        Self {
            yaw,
            pitch,
            onGround,
        }
    }

    pub fn writePacketData(&self) -> RawPacket {
        let mut payload = Vec::with_capacity(9);
        write_f32_be(self.yaw, &mut payload);
        write_f32_be(self.pitch, &mut payload);
        write_bool(self.onGround, &mut payload);
        RawPacket::new(0x0F, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_340_player_packet_ids_match_mcp_variants() {
        assert_eq!(CPacketPlayer::new(true).writePacketData().id, 0x0C);
        assert_eq!(
            Position::new(1.0, 2.0, 3.0, false).writePacketData().id,
            0x0D
        );
        assert_eq!(
            PositionRotation::new(1.0, 2.0, 3.0, 4.0, 5.0, false)
                .writePacketData()
                .id,
            0x0E
        );
        assert_eq!(Rotation::new(4.0, 5.0, true).writePacketData().id, 0x0F);
    }
}
