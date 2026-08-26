use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_string, CodecError};
use crate::net::minecraft::util::text::ITextComponent::{ITextComponent, TextComponentError};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketDisconnect {
    reason: ITextComponent,
}
impl SPacketDisconnect {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, SPacketDisconnectError> {
        let mut input = packet.payload.as_slice();
        let json = read_string(&mut input, 32767)?;
        Ok(Self {
            reason: ITextComponent::fromJsonLenient(&json)?,
        })
    }
    pub fn getReason(&self) -> &ITextComponent {
        &self.reason
    }
}
#[derive(Debug, thiserror::Error)]
pub enum SPacketDisconnectError {
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error(transparent)]
    Text(#[from] TextComponentError),
}
