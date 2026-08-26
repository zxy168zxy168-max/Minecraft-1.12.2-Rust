use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::write_i64_be;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPacketKeepAlive {
    key: i64,
}
impl CPacketKeepAlive {
    pub const fn new(idIn: i64) -> Self {
        Self { key: idIn }
    }
    pub fn writePacketData(&self) -> RawPacket {
        let mut payload = Vec::with_capacity(8);
        write_i64_be(self.key, &mut payload);
        RawPacket::new(0x0B, payload)
    }
    pub const fn getKey(&self) -> i64 {
        self.key
    }
}
