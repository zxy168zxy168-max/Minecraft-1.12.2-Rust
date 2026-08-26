use crate::net::minecraft::network::play::server::SPacketCustomSound::read_sound_category;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_f32_be, read_i32_be, read_var_i32, CodecError,
};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::util::SoundCategory::SoundCategory;
use crate::net::minecraft::util::SoundEvent::SoundEvent;

/// MCP 1.12.2 `SPacketSoundEffect` (clientbound play packet 0x49).
#[derive(Debug, Clone, PartialEq)]
pub struct SPacketSoundEffect {
    sound: ResourceLocation,
    category: SoundCategory,
    posX: i32,
    posY: i32,
    posZ: i32,
    soundVolume: f32,
    soundPitch: f32,
}

impl SPacketSoundEffect {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let soundId = read_var_i32(&mut input)?;
        let sound = SoundEvent::getById(soundId).ok_or_else(|| {
            CodecError::InvalidData(format!("unknown SoundEvent registry id {soundId}"))
        })?;
        let category = read_sound_category(&mut input)?;
        let posX = read_i32_be(&mut input)?;
        let posY = read_i32_be(&mut input)?;
        let posZ = read_i32_be(&mut input)?;
        let soundVolume = read_f32_be(&mut input)?;
        let soundPitch = read_f32_be(&mut input)?;
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} trailing SPacketSoundEffect bytes",
                input.len()
            )));
        }
        Ok(Self {
            sound,
            category,
            posX,
            posY,
            posZ,
            soundVolume,
            soundPitch,
        })
    }

    pub fn getSound(&self) -> &ResourceLocation {
        &self.sound
    }
    pub const fn getCategory(&self) -> SoundCategory {
        self.category
    }
    pub fn getX(&self) -> f64 {
        self.posX as f64 / 8.0
    }
    pub fn getY(&self) -> f64 {
        self.posY as f64 / 8.0
    }
    pub fn getZ(&self) -> f64 {
        self.posZ as f64 / 8.0
    }
    pub const fn getVolume(&self) -> f32 {
        self.soundVolume
    }
    pub const fn getPitch(&self) -> f32 {
        self.soundPitch
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{write_f32_be, write_i32_be, write_var_i32};

    #[test]
    fn resolves_numeric_sound_registry() {
        let sound = ResourceLocation::parse("minecraft:ui.button.click");
        let mut payload = Vec::new();
        write_var_i32(SoundEvent::getId(&sound).unwrap(), &mut payload);
        write_var_i32(0, &mut payload);
        write_i32_be(0, &mut payload);
        write_i32_be(8, &mut payload);
        write_i32_be(16, &mut payload);
        write_f32_be(1.0, &mut payload);
        write_f32_be(1.0, &mut payload);
        let packet = SPacketSoundEffect::readPacketData(&RawPacket::new(0x49, payload)).unwrap();
        assert_eq!(packet.getSound(), &sound);
        assert_eq!(packet.getCategory(), SoundCategory::Master);
    }
}
