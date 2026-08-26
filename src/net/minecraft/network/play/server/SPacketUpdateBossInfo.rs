use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_f32_be, read_string, read_u8, read_uuid, read_var_i32, CodecError,
};
use crate::net::minecraft::util::text::ITextComponent::{ITextComponent, TextComponentError};
use crate::net::minecraft::world::BossInfo::{Color, Overlay};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Add,
    Remove,
    UpdatePct,
    UpdateName,
    UpdateStyle,
    UpdateProperties,
}
impl Operation {
    pub const VALUES: [Self; 6] = [
        Self::Add,
        Self::Remove,
        Self::UpdatePct,
        Self::UpdateName,
        Self::UpdateStyle,
        Self::UpdateProperties,
    ];
    pub fn byOrdinal(value: i32) -> Option<Self> {
        Self::VALUES.get(value as usize).copied()
    }
}

/// Protocol 340 clientbound 0x0C. The bit-2 fog/music alias is deliberately
/// preserved because it is the behavior of the supplied 1.12.2 MCP source.
#[derive(Debug, Clone, PartialEq)]
pub struct SPacketUpdateBossInfo {
    uniqueId: Uuid,
    operation: Operation,
    name: Option<ITextComponent>,
    percent: f32,
    color: Option<Color>,
    overlay: Option<Overlay>,
    darkenSky: bool,
    playEndBossMusic: bool,
    createFog: bool,
}

impl SPacketUpdateBossInfo {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, SPacketUpdateBossInfoError> {
        let mut input = packet.payload.as_slice();
        let uniqueId = read_uuid(&mut input)?;
        let operationOrdinal = read_var_i32(&mut input)?;
        let operation = Operation::byOrdinal(operationOrdinal).ok_or_else(|| {
            CodecError::InvalidData(format!("invalid boss operation: {operationOrdinal}"))
        })?;
        let mut result = Self {
            uniqueId,
            operation,
            name: None,
            percent: 0.0,
            color: None,
            overlay: None,
            darkenSky: false,
            playEndBossMusic: false,
            createFog: false,
        };
        match operation {
            Operation::Add => {
                result.name = Some(ITextComponent::fromJsonLenient(&read_string(
                    &mut input, 32_767,
                )?)?);
                result.percent = read_f32_be(&mut input)?;
                result.color = Some(read_color(&mut input)?);
                result.overlay = Some(read_overlay(&mut input)?);
                result.setFlags(read_u8(&mut input)?);
            }
            Operation::Remove => {}
            Operation::UpdatePct => result.percent = read_f32_be(&mut input)?,
            Operation::UpdateName => {
                result.name = Some(ITextComponent::fromJsonLenient(&read_string(
                    &mut input, 32_767,
                )?)?)
            }
            Operation::UpdateStyle => {
                result.color = Some(read_color(&mut input)?);
                result.overlay = Some(read_overlay(&mut input)?);
            }
            Operation::UpdateProperties => result.setFlags(read_u8(&mut input)?),
        }
        Ok(result)
    }

    fn setFlags(&mut self, flags: u8) {
        self.darkenSky = flags & 1 != 0;
        self.playEndBossMusic = flags & 2 != 0;
        self.createFog = flags & 2 != 0;
    }
    pub const fn getUniqueId(&self) -> Uuid {
        self.uniqueId
    }
    pub const fn getOperation(&self) -> Operation {
        self.operation
    }
    pub fn getName(&self) -> Option<&ITextComponent> {
        self.name.as_ref()
    }
    pub const fn getPercent(&self) -> f32 {
        self.percent
    }
    pub const fn getColor(&self) -> Option<Color> {
        self.color
    }
    pub const fn getOverlay(&self) -> Option<Overlay> {
        self.overlay
    }
    pub const fn shouldDarkenSky(&self) -> bool {
        self.darkenSky
    }
    pub const fn shouldPlayEndBossMusic(&self) -> bool {
        self.playEndBossMusic
    }
    pub const fn shouldCreateFog(&self) -> bool {
        self.createFog
    }
}

fn read_color(input: &mut &[u8]) -> Result<Color, CodecError> {
    let value = read_var_i32(input)?;
    Color::byOrdinal(value)
        .ok_or_else(|| CodecError::InvalidData(format!("invalid boss color: {value}")))
}
fn read_overlay(input: &mut &[u8]) -> Result<Overlay, CodecError> {
    let value = read_var_i32(input)?;
    Overlay::byOrdinal(value)
        .ok_or_else(|| CodecError::InvalidData(format!("invalid boss overlay: {value}")))
}

#[derive(Debug, thiserror::Error)]
pub enum SPacketUpdateBossInfoError {
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error(transparent)]
    Text(#[from] TextComponentError),
}
