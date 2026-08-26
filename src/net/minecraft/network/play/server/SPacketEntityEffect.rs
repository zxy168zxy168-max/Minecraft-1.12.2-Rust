use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i8, read_var_i32, CodecError};

/// Clientbound Play 0x4F in protocol 340.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketEntityEffect {
    entityId: i32,
    effectId: u8,
    amplifier: u8,
    duration: i32,
    flags: u8,
}

impl SPacketEntityEffect {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let result = Self {
            entityId: read_var_i32(&mut input)?,
            effectId: read_i8(&mut input)? as u8,
            amplifier: read_i8(&mut input)? as u8,
            duration: read_var_i32(&mut input)?,
            flags: read_i8(&mut input)? as u8,
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread entity-effect bytes",
                input.len()
            )));
        }
        Ok(result)
    }

    pub const fn getEntityId(&self) -> i32 {
        self.entityId
    }
    pub const fn getEffectId(&self) -> u8 {
        self.effectId
    }
    pub const fn getAmplifier(&self) -> u8 {
        self.amplifier
    }
    pub const fn getDuration(&self) -> i32 {
        self.duration
    }
    pub const fn getIsAmbient(&self) -> bool {
        self.flags & 1 != 0
    }
    pub const fn doesShowParticles(&self) -> bool {
        self.flags & 2 != 0
    }
    pub const fn isMaxDuration(&self) -> bool {
        self.duration == 32_767
    }
}
