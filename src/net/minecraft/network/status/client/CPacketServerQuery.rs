use crate::net::minecraft::network::Packet::RawPacket;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CPacketServerQuery;
impl CPacketServerQuery {
    pub fn writePacketData(self) -> RawPacket {
        RawPacket::new(0, [])
    }
}
