use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i64_be, CodecError};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketKeepAlive {
    id: i64,
}
impl SPacketKeepAlive {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        Ok(Self {
            id: read_i64_be(&mut input)?,
        })
    }
    pub const fn getId(&self) -> i64 {
        self.id
    }
}
