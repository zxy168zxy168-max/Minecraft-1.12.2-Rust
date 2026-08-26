use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_f32_be, read_i32_be, read_string, read_var_i32, CodecError,
};
use crate::net::minecraft::util::SoundCategory::SoundCategory;

/// MCP 1.12.2 `SPacketCustomSound` (clientbound play packet 0x19).
#[derive(Debug, Clone, PartialEq)]
pub struct SPacketCustomSound {
    soundName: String,
    category: SoundCategory,
    x: i32,
    y: i32,
    z: i32,
    volume: f32,
    pitch: f32,
}

impl SPacketCustomSound {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let soundName = read_string(&mut input, 256)?;
        let category = read_sound_category(&mut input)?;
        let x = read_i32_be(&mut input)?;
        let y = read_i32_be(&mut input)?;
        let z = read_i32_be(&mut input)?;
        let volume = read_f32_be(&mut input)?;
        let pitch = read_f32_be(&mut input)?;
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} trailing SPacketCustomSound bytes",
                input.len()
            )));
        }
        Ok(Self {
            soundName,
            category,
            x,
            y,
            z,
            volume,
            pitch,
        })
    }

    pub fn getSoundName(&self) -> &str {
        &self.soundName
    }
    pub const fn getCategory(&self) -> SoundCategory {
        self.category
    }
    pub fn getX(&self) -> f64 {
        self.x as f64 / 8.0
    }
    pub fn getY(&self) -> f64 {
        self.y as f64 / 8.0
    }
    pub fn getZ(&self) -> f64 {
        self.z as f64 / 8.0
    }
    pub const fn getVolume(&self) -> f32 {
        self.volume
    }
    pub const fn getPitch(&self) -> f32 {
        self.pitch
    }
}

pub(super) fn read_sound_category(input: &mut &[u8]) -> Result<SoundCategory, CodecError> {
    match read_var_i32(input)? {
        0 => Ok(SoundCategory::Master),
        1 => Ok(SoundCategory::Music),
        2 => Ok(SoundCategory::Records),
        3 => Ok(SoundCategory::Weather),
        4 => Ok(SoundCategory::Blocks),
        5 => Ok(SoundCategory::Hostile),
        6 => Ok(SoundCategory::Neutral),
        7 => Ok(SoundCategory::Players),
        8 => Ok(SoundCategory::Ambient),
        9 => Ok(SoundCategory::Voice),
        ordinal => Err(CodecError::InvalidData(format!(
            "invalid SoundCategory ordinal {ordinal}",
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{
        write_f32_be, write_i32_be, write_string, write_var_i32,
    };

    #[test]
    fn reads_fixed_point_position_and_enum_ordinal() {
        let mut payload = Vec::new();
        write_string("minecraft:block.note.harp", 256, &mut payload).unwrap();
        write_var_i32(4, &mut payload);
        write_i32_be(12, &mut payload);
        write_i32_be(-20, &mut payload);
        write_i32_be(32, &mut payload);
        write_f32_be(0.75, &mut payload);
        write_f32_be(1.25, &mut payload);
        let packet = SPacketCustomSound::readPacketData(&RawPacket::new(0x19, payload)).unwrap();
        assert_eq!(packet.getCategory(), SoundCategory::Blocks);
        assert_eq!(
            (packet.getX(), packet.getY(), packet.getZ()),
            (1.5, -2.5, 4.0)
        );
    }
}
