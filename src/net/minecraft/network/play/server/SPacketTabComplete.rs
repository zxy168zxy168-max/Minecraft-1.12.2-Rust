use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_string, read_var_i32, CodecError};

/// Protocol 340 clientbound 0x0E, matching MCP 1.12.2
/// `SPacketTabComplete`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketTabComplete {
    matches: Vec<String>,
}

impl SPacketTabComplete {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let count = read_var_i32(&mut input)?;
        if count < 0 || count > 32_767 {
            return Err(CodecError::InvalidData(format!(
                "invalid tab-completion count: {count}"
            )));
        }
        let mut matches = Vec::with_capacity(count as usize);
        for _ in 0..count {
            matches.push(read_string(&mut input, 32_767)?);
        }
        Ok(Self { matches })
    }

    pub fn getMatches(&self) -> &[String] {
        &self.matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{write_string, write_var_i32};

    #[test]
    fn reads_matches_in_server_order() {
        let mut payload = Vec::new();
        write_var_i32(3, &mut payload);
        for value in ["Player", "Player905", "Player906"] {
            write_string(value, 32_767, &mut payload).unwrap();
        }
        let packet = SPacketTabComplete::readPacketData(&RawPacket::new(0x0E, payload)).unwrap();
        assert_eq!(packet.getMatches(), &["Player", "Player905", "Player906"]);
    }
}
