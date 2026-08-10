use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_f32_be, read_u8, write_f32_be, CodecError};
use crate::net::minecraft::entity::player::PlayerCapabilities::PlayerCapabilities;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SPacketPlayerAbilities {
    invulnerable: bool,
    flying: bool,
    allowFlying: bool,
    creativeMode: bool,
    flySpeed: f32,
    walkSpeed: f32,
}

impl SPacketPlayerAbilities {
    pub fn new(capabilities: &PlayerCapabilities) -> Self { Self { invulnerable: capabilities.disableDamage, flying: capabilities.isFlying, allowFlying: capabilities.allowFlying, creativeMode: capabilities.isCreativeMode, flySpeed: capabilities.getFlySpeed(), walkSpeed: capabilities.getWalkSpeed() } }
    pub fn writePacketData(&self) -> RawPacket { let mut flags=0u8; if self.invulnerable{flags|=1;} if self.flying{flags|=2;} if self.allowFlying{flags|=4;} if self.creativeMode{flags|=8;} let mut payload=vec![flags]; write_f32_be(self.flySpeed,&mut payload); write_f32_be(self.walkSpeed,&mut payload); RawPacket::new(0x2C,payload) }
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let flags = read_u8(&mut input)?;
        let result = Self {
            invulnerable: flags & 0x01 != 0,
            flying: flags & 0x02 != 0,
            allowFlying: flags & 0x04 != 0,
            creativeMode: flags & 0x08 != 0,
            flySpeed: read_f32_be(&mut input)?,
            walkSpeed: read_f32_be(&mut input)?,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread player-abilities bytes",
                input.len()
            )));
        }
        Ok(result)
    }

    pub const fn isInvulnerable(&self) -> bool { self.invulnerable }
    pub const fn isFlying(&self) -> bool { self.flying }
    pub const fn isAllowFlying(&self) -> bool { self.allowFlying }
    pub const fn isCreativeMode(&self) -> bool { self.creativeMode }
    pub const fn getFlySpeed(&self) -> f32 { self.flySpeed }
    pub const fn getWalkSpeed(&self) -> f32 { self.walkSpeed }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_mcp_flags_and_speeds() {
        let mut payload = vec![0x07];
        payload.extend_from_slice(&0.05_f32.to_bits().to_be_bytes());
        payload.extend_from_slice(&0.1_f32.to_bits().to_be_bytes());
        let packet = SPacketPlayerAbilities::readPacketData(&RawPacket::new(0x2C, payload)).unwrap();
        assert!(packet.isInvulnerable());
        assert!(packet.isFlying());
        assert!(packet.isAllowFlying());
        assert!(!packet.isCreativeMode());
        assert!((packet.getFlySpeed() - 0.05).abs() < f32::EPSILON);
        assert!((packet.getWalkSpeed() - 0.1).abs() < f32::EPSILON);
    }
}
