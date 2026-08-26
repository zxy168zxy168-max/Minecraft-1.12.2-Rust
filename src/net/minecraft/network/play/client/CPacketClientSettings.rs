use crate::net::minecraft::entity::player::EntityPlayer::EnumChatVisibility;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    write_bool, write_string, write_var_i32, CodecError,
};
use crate::net::minecraft::util::EnumHandSide::EnumHandSide;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CPacketClientSettings {
    lang: String,
    view: i8,
    chatVisibility: EnumChatVisibility,
    enableColors: bool,
    modelPartFlags: u8,
    mainHand: EnumHandSide,
}

impl CPacketClientSettings {
    pub fn new(
        langIn: impl Into<String>,
        renderDistanceIn: i32,
        chatVisibilityIn: EnumChatVisibility,
        chatColorsIn: bool,
        modelPartsIn: u8,
        mainHandIn: EnumHandSide,
    ) -> Self {
        Self {
            lang: langIn.into(),
            view: renderDistanceIn.clamp(i8::MIN as i32, i8::MAX as i32) as i8,
            chatVisibility: chatVisibilityIn,
            enableColors: chatColorsIn,
            modelPartFlags: modelPartsIn,
            mainHand: mainHandIn,
        }
    }

    pub fn writePacketData(&self) -> Result<RawPacket, CodecError> {
        let mut payload = Vec::new();
        write_string(&self.lang, 16, &mut payload)?;
        payload.push(self.view as u8);
        write_var_i32(self.chatVisibility.getChatVisibilityId(), &mut payload);
        write_bool(self.enableColors, &mut payload);
        payload.push(self.modelPartFlags);
        write_var_i32(self.mainHand.getId(), &mut payload);
        Ok(RawPacket::new(0x04, payload))
    }

    pub fn getLang(&self) -> &str {
        &self.lang
    }
    pub const fn getChatVisibility(&self) -> EnumChatVisibility {
        self.chatVisibility
    }
    pub const fn isColorsEnabled(&self) -> bool {
        self.enableColors
    }
    pub const fn getModelPartFlags(&self) -> u8 {
        self.modelPartFlags
    }
    pub const fn getMainHand(&self) -> EnumHandSide {
        self.mainHand
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_340_payload_matches_mcp_field_order() {
        let packet = CPacketClientSettings::new(
            "en_us",
            8,
            EnumChatVisibility::Full,
            true,
            0x7F,
            EnumHandSide::Right,
        )
        .writePacketData()
        .unwrap();
        assert_eq!(packet.id, 0x04);
        assert_eq!(
            packet.payload,
            vec![5, b'e', b'n', b'_', b'u', b's', 8, 0, 1, 0x7F, 1]
        );
    }
}
