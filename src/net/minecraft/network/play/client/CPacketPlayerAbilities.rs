use crate::net::minecraft::entity::player::PlayerCapabilities::PlayerCapabilities;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::write_f32_be;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CPacketPlayerAbilities {
    invulnerable: bool,
    flying: bool,
    allowFlying: bool,
    creativeMode: bool,
    flySpeed: f32,
    walkSpeed: f32,
}

impl CPacketPlayerAbilities {
    pub fn new(capabilities: &PlayerCapabilities) -> Self {
        Self {
            invulnerable: capabilities.disableDamage,
            flying: capabilities.isFlying,
            allowFlying: capabilities.allowFlying,
            creativeMode: capabilities.isCreativeMode,
            flySpeed: capabilities.getFlySpeed(),
            walkSpeed: capabilities.getWalkSpeed(),
        }
    }

    pub fn writePacketData(&self) -> RawPacket {
        let mut flags = 0_u8;
        if self.invulnerable {
            flags |= 0x01;
        }
        if self.flying {
            flags |= 0x02;
        }
        if self.allowFlying {
            flags |= 0x04;
        }
        if self.creativeMode {
            flags |= 0x08;
        }
        let mut payload = vec![flags];
        write_f32_be(self.flySpeed, &mut payload);
        write_f32_be(self.walkSpeed, &mut payload);
        RawPacket::new(0x13, payload)
    }

    pub const fn isFlying(&self) -> bool {
        self.flying
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_340_flags_and_field_order_match_mcp() {
        let mut capabilities = PlayerCapabilities::default();
        capabilities.disableDamage = true;
        capabilities.isFlying = true;
        capabilities.allowFlying = true;
        capabilities.isCreativeMode = true;
        capabilities.setFlySpeed(0.05);
        capabilities.setPlayerWalkSpeed(0.1);
        let packet = CPacketPlayerAbilities::new(&capabilities).writePacketData();
        assert_eq!(packet.id, 0x13);
        assert_eq!(packet.payload[0], 0x0F);
        assert_eq!(&packet.payload[1..5], &0.05_f32.to_bits().to_be_bytes());
        assert_eq!(&packet.payload[5..9], &0.1_f32.to_bits().to_be_bytes());
    }
}
