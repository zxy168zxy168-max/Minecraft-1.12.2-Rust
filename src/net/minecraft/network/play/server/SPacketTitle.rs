use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_i32_be, read_string, read_var_i32, CodecError,
};
use crate::net::minecraft::util::text::ITextComponent::{ITextComponent, TextComponentError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Title,
    Subtitle,
    Actionbar,
    Times,
    Clear,
    Reset,
}
impl Type {
    pub const VALUES: [Self; 6] = [
        Self::Title,
        Self::Subtitle,
        Self::Actionbar,
        Self::Times,
        Self::Clear,
        Self::Reset,
    ];
    pub fn byOrdinal(value: i32) -> Option<Self> {
        Self::VALUES.get(value as usize).copied()
    }
}

/// Protocol 340 clientbound 0x48, matching MCP 1.12.2 `SPacketTitle`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketTitle {
    type_: Type,
    message: Option<ITextComponent>,
    fadeInTime: i32,
    displayTime: i32,
    fadeOutTime: i32,
}

impl SPacketTitle {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, SPacketTitleError> {
        let mut input = packet.payload.as_slice();
        let ordinal = read_var_i32(&mut input)?;
        let type_ = Type::byOrdinal(ordinal)
            .ok_or_else(|| CodecError::InvalidData(format!("invalid title type: {ordinal}")))?;
        let message = if matches!(type_, Type::Title | Type::Subtitle | Type::Actionbar) {
            Some(ITextComponent::fromJsonLenient(&read_string(
                &mut input, 32_767,
            )?)?)
        } else {
            None
        };
        let (fadeInTime, displayTime, fadeOutTime) = if type_ == Type::Times {
            (
                read_i32_be(&mut input)?,
                read_i32_be(&mut input)?,
                read_i32_be(&mut input)?,
            )
        } else {
            (-1, -1, -1)
        };
        Ok(Self {
            type_,
            message,
            fadeInTime,
            displayTime,
            fadeOutTime,
        })
    }
    pub const fn getType(&self) -> Type {
        self.type_
    }
    pub fn getMessage(&self) -> Option<&ITextComponent> {
        self.message.as_ref()
    }
    pub const fn getFadeInTime(&self) -> i32 {
        self.fadeInTime
    }
    pub const fn getDisplayTime(&self) -> i32 {
        self.displayTime
    }
    pub const fn getFadeOutTime(&self) -> i32 {
        self.fadeOutTime
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SPacketTitleError {
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error(transparent)]
    Text(#[from] TextComponentError),
}
