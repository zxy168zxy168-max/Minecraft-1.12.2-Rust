use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_bool, read_f32_be, read_i32_be, read_var_i32, CodecError,
};
use crate::net::minecraft::util::EnumParticleTypes::EnumParticleTypes;

/// Protocol-340 clientbound play packet `SPacketParticles` (`0x22`).
#[derive(Debug, Clone, PartialEq)]
pub struct SPacketParticles {
    particleType: EnumParticleTypes,
    longDistance: bool,
    xCoord: f32,
    yCoord: f32,
    zCoord: f32,
    xOffset: f32,
    yOffset: f32,
    zOffset: f32,
    particleSpeed: f32,
    particleCount: i32,
    particleArguments: [i32; 2],
}

impl SPacketParticles {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let particleType = EnumParticleTypes::fromId(read_i32_be(&mut input)?)
            .unwrap_or(EnumParticleTypes::Barrier);
        let longDistance = read_bool(&mut input)?;
        let xCoord = read_f32_be(&mut input)?;
        let yCoord = read_f32_be(&mut input)?;
        let zCoord = read_f32_be(&mut input)?;
        let xOffset = read_f32_be(&mut input)?;
        let yOffset = read_f32_be(&mut input)?;
        let zOffset = read_f32_be(&mut input)?;
        let particleSpeed = read_f32_be(&mut input)?;
        let particleCount = read_i32_be(&mut input)?;
        let mut particleArguments = [0; 2];
        for argument in particleArguments
            .iter_mut()
            .take(particleType.argumentCount())
        {
            *argument = read_var_i32(&mut input)?;
        }
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread particle packet bytes",
                input.len(),
            )));
        }
        Ok(Self {
            particleType,
            longDistance,
            xCoord,
            yCoord,
            zCoord,
            xOffset,
            yOffset,
            zOffset,
            particleSpeed,
            particleCount,
            particleArguments,
        })
    }

    pub const fn getParticleType(&self) -> EnumParticleTypes {
        self.particleType
    }
    pub const fn isLongDistance(&self) -> bool {
        self.longDistance
    }
    pub const fn getXCoordinate(&self) -> f64 {
        self.xCoord as f64
    }
    pub const fn getYCoordinate(&self) -> f64 {
        self.yCoord as f64
    }
    pub const fn getZCoordinate(&self) -> f64 {
        self.zCoord as f64
    }
    pub const fn getXOffset(&self) -> f32 {
        self.xOffset
    }
    pub const fn getYOffset(&self) -> f32 {
        self.yOffset
    }
    pub const fn getZOffset(&self) -> f32 {
        self.zOffset
    }
    pub const fn getParticleSpeed(&self) -> f32 {
        self.particleSpeed
    }
    pub const fn getParticleCount(&self) -> i32 {
        self.particleCount
    }
    pub const fn getParticleArgs(&self) -> [i32; 2] {
        self.particleArguments
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{
        write_bool, write_f32_be, write_i32_be, write_var_i32,
    };

    #[test]
    fn protocol_layout_and_unknown_fallback_match_mcp() {
        let mut payload = Vec::new();
        write_i32_be(EnumParticleTypes::ItemCrack.particleId(), &mut payload);
        write_bool(true, &mut payload);
        for value in [1.25, 2.5, -3.75, 0.1, 0.2, 0.3, 0.4] {
            write_f32_be(value, &mut payload);
        }
        write_i32_be(12, &mut payload);
        write_var_i32(358, &mut payload);
        write_var_i32(7, &mut payload);
        let packet = SPacketParticles::readPacketData(&RawPacket::new(0x22, payload)).unwrap();
        assert_eq!(packet.getParticleType(), EnumParticleTypes::ItemCrack);
        assert!(packet.isLongDistance());
        assert_eq!(packet.getParticleCount(), 12);
        assert_eq!(packet.getParticleArgs(), [358, 7]);

        let mut unknown = Vec::new();
        write_i32_be(999, &mut unknown);
        write_bool(false, &mut unknown);
        for _ in 0..7 {
            write_f32_be(0.0, &mut unknown);
        }
        write_i32_be(0, &mut unknown);
        assert_eq!(
            SPacketParticles::readPacketData(&RawPacket::new(0x22, unknown))
                .unwrap()
                .getParticleType(),
            EnumParticleTypes::Barrier,
        );
    }
}
