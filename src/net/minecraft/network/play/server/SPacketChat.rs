use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i8, read_text_component, CodecError};
use crate::net::minecraft::util::text::ChatType::ChatType;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

/// Protocol 340 clientbound 0x0F, matching MCP 1.12.2 `SPacketChat`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketChat {
    chatComponent: ITextComponent,
    type_: ChatType,
}

impl SPacketChat {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let chatComponent = read_text_component(&mut input)?;
        let type_ = ChatType::func_192582_a(read_i8(&mut input)?);
        Ok(Self {
            chatComponent,
            type_,
        })
    }

    pub fn getChatComponent(&self) -> &ITextComponent {
        &self.chatComponent
    }
    pub const fn isSystem(&self) -> bool {
        matches!(self.type_, ChatType::System | ChatType::GameInfo)
    }
    pub const fn func_192590_c(&self) -> ChatType {
        self.type_
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::write_string;

    #[test]
    fn reads_component_then_chat_type() {
        let mut payload = Vec::new();
        write_string(r#"{"text":"hello"}"#, 32_767, &mut payload).unwrap();
        payload.push(2);
        let packet = SPacketChat::readPacketData(&RawPacket::new(0x0F, payload)).unwrap();
        assert_eq!(packet.getChatComponent().getUnformattedText(), "hello");
        assert_eq!(packet.func_192590_c(), ChatType::GameInfo);
    }
}
