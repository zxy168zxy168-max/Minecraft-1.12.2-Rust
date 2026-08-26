use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i64_be, CodecError};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketPong {
    clientTime: i64,
}
impl SPacketPong {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        Ok(Self {
            clientTime: read_i64_be(&mut input)?,
        })
    }
    pub const fn getClientTime(self) -> i64 {
        self.clientTime
    }
}
