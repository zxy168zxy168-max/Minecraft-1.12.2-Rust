use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i64_be, write_i64_be, CodecError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketTimeUpdate {
    totalWorldTime: i64,
    worldTime: i64,
}

impl SPacketTimeUpdate {
    pub fn new(totalWorldTimeIn: i64, worldTimeIn: i64, doDaylightCycle: bool) -> Self {
        let worldTime = if doDaylightCycle {
            worldTimeIn
        } else {
            let value = -worldTimeIn;
            if value == 0 {
                -1
            } else {
                value
            }
        };
        Self {
            totalWorldTime: totalWorldTimeIn,
            worldTime,
        }
    }
    pub fn writePacketData(&self) -> RawPacket {
        let mut payload = Vec::new();
        write_i64_be(self.totalWorldTime, &mut payload);
        write_i64_be(self.worldTime, &mut payload);
        RawPacket::new(0x47, payload)
    }
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let totalWorldTime = read_i64_be(&mut input)?;
        let worldTime = read_i64_be(&mut input)?;
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread time-update bytes",
                input.len()
            )));
        }
        Ok(Self {
            totalWorldTime,
            worldTime,
        })
    }

    pub const fn getTotalWorldTime(&self) -> i64 {
        self.totalWorldTime
    }

    pub const fn getWorldTime(&self) -> i64 {
        self.worldTime
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::write_i64_be;

    #[test]
    fn reads_protocol_340_time_update() {
        let mut payload = Vec::new();
        write_i64_be(1234, &mut payload);
        write_i64_be(-6000, &mut payload);
        let packet = SPacketTimeUpdate::readPacketData(&RawPacket::new(0x47, payload)).unwrap();
        assert_eq!(packet.getTotalWorldTime(), 1234);
        assert_eq!(packet.getWorldTime(), -6000);
    }
}
