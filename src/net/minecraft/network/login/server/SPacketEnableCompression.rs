use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_var_i32, CodecError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketEnableCompression {
    compressionThreshold: i32,
}
impl SPacketEnableCompression {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        Ok(Self {
            compressionThreshold: read_var_i32(&mut input)?,
        })
    }
    pub const fn getCompressionThreshold(&self) -> i32 {
        self.compressionThreshold
    }
}
