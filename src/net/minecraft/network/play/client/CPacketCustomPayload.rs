use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{write_string, CodecError};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CPacketCustomPayload {
    channel: String,
    data: Vec<u8>,
}
impl CPacketCustomPayload {
    pub fn new(channelIn: impl Into<String>, data: Vec<u8>) -> Result<Self, CodecError> {
        if data.len() > 32767 {
            return Err(CodecError::PacketTooLarge {
                actual: data.len(),
                maximum: 32767,
            });
        }
        Ok(Self {
            channel: channelIn.into(),
            data,
        })
    }
    pub fn writePacketData(&self) -> Result<RawPacket, CodecError> {
        let mut payload = Vec::new();
        write_string(&self.channel, 20, &mut payload)?;
        payload.extend_from_slice(&self.data);
        Ok(RawPacket::new(0x09, payload))
    }
}
