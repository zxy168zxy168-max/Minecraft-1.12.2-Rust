use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_string, CodecError};
use crate::net::minecraft::util::text::ITextComponent::{ITextComponent, TextComponentError};

/// Protocol 340 clientbound 0x4A, matching MCP 1.12.2
/// `SPacketPlayerListHeaderFooter`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketPlayerListHeaderFooter {
    header: ITextComponent,
    footer: ITextComponent,
}

impl SPacketPlayerListHeaderFooter {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, SPacketPlayerListHeaderFooterError> {
        let mut input = packet.payload.as_slice();
        let header = ITextComponent::fromJsonLenient(&read_string(&mut input, 32_767)?)?;
        let footer = ITextComponent::fromJsonLenient(&read_string(&mut input, 32_767)?)?;
        Ok(Self { header, footer })
    }

    pub fn getHeader(&self) -> &ITextComponent {
        &self.header
    }
    pub fn getFooter(&self) -> &ITextComponent {
        &self.footer
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SPacketPlayerListHeaderFooterError {
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error(transparent)]
    Text(#[from] TextComponentError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::write_string;

    #[test]
    fn reads_header_then_footer_in_protocol_order() {
        let mut payload = Vec::new();
        write_string(r#"{"text":"Header"}"#, 32_767, &mut payload).unwrap();
        write_string(r#"{"text":"Footer"}"#, 32_767, &mut payload).unwrap();
        let packet =
            SPacketPlayerListHeaderFooter::readPacketData(&RawPacket::new(0x4A, payload)).unwrap();
        assert_eq!(packet.getHeader().getUnformattedText(), "Header");
        assert_eq!(packet.getFooter().getUnformattedText(), "Footer");
    }
}
