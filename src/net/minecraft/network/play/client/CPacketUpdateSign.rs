use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{write_i64_be, write_string, CodecError};
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// Protocol 340 serverbound 0x1C, matching MCP 1.12.2
/// `CPacketUpdateSign`: packed BlockPos followed by four UTF-8 strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CPacketUpdateSign {
    position: BlockPos,
    lines: [String; 4],
}

impl CPacketUpdateSign {
    pub fn new(positionIn: BlockPos, linesIn: [String; 4]) -> Self {
        Self {
            position: positionIn,
            lines: linesIn.map(|line| truncate_utf16(&line, 384)),
        }
    }

    pub fn writePacketData(&self) -> Result<RawPacket, CodecError> {
        let mut payload = Vec::new();
        write_i64_be(self.position.to_long(), &mut payload);
        for line in &self.lines {
            write_string(line, 384, &mut payload)?;
        }
        Ok(RawPacket::new(0x1C, payload))
    }

    pub const fn getPosition(&self) -> BlockPos {
        self.position
    }
    pub const fn getLines(&self) -> &[String; 4] {
        &self.lines
    }
}

fn truncate_utf16(value: &str, maximum: usize) -> String {
    String::from_utf16_lossy(&value.encode_utf16().take(maximum).collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{read_i64_be, read_string};

    #[test]
    fn writes_protocol_340_position_and_four_lines() {
        let position = BlockPos::new(-17, 72, 300);
        let packet = CPacketUpdateSign::new(
            position,
            [
                "one".to_owned(),
                "two".to_owned(),
                "三".to_owned(),
                "".to_owned(),
            ],
        )
        .writePacketData()
        .unwrap();
        assert_eq!(packet.id, 0x1C);
        let mut input = packet.payload.as_slice();
        assert_eq!(
            BlockPos::from_long(read_i64_be(&mut input).unwrap()),
            position
        );
        assert_eq!(read_string(&mut input, 384).unwrap(), "one");
        assert_eq!(read_string(&mut input, 384).unwrap(), "two");
        assert_eq!(read_string(&mut input, 384).unwrap(), "三");
        assert_eq!(read_string(&mut input, 384).unwrap(), "");
        assert!(input.is_empty());
    }
}
