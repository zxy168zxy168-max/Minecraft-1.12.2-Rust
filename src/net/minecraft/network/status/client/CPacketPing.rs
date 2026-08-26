use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::write_i64_be;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPacketPing {
    clientTime: i64,
}
impl CPacketPing {
    pub const fn new(clientTimeIn: i64) -> Self {
        Self {
            clientTime: clientTimeIn,
        }
    }
    pub fn writePacketData(self) -> RawPacket {
        let mut payload = Vec::with_capacity(8);
        write_i64_be(self.clientTime, &mut payload);
        RawPacket::new(1, payload)
    }
    pub const fn getClientTime(self) -> i64 {
        self.clientTime
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ping_is_packet_one_with_big_endian_long() {
        let packet = CPacketPing::new(0x0102_0304_0506_0708).writePacketData();
        assert_eq!(packet.id, 1);
        assert_eq!(packet.payload, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    }
}
