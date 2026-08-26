use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i8, read_string, CodecError};

/// Protocol 340 clientbound 0x3B, matching MCP 1.12.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketDisplayObjective {
    position: i32,
    scoreName: String,
}

impl SPacketDisplayObjective {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        Ok(Self {
            position: read_i8(&mut input)? as i32,
            scoreName: read_string(&mut input, 16)?,
        })
    }
    pub const fn getPosition(&self) -> i32 {
        self.position
    }
    pub fn getName(&self) -> &str {
        &self.scoreName
    }
}
