use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i16_be, read_u8, CodecError};

/// Protocol-340 port of MCP 1.12.2 `SPacketWindowProperty`
/// (clientbound 0x15). Property and value are signed shorts on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketWindowProperty {
    windowId: u8,
    property: i16,
    value: i16,
}

impl SPacketWindowProperty {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let windowId = read_u8(&mut input)?;
        let property = read_i16_be(&mut input)?;
        let value = read_i16_be(&mut input)?;
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread window-property bytes",
                input.len()
            )));
        }
        Ok(Self {
            windowId,
            property,
            value,
        })
    }

    pub const fn getWindowId(&self) -> u8 {
        self.windowId
    }
    pub const fn getProperty(&self) -> i16 {
        self.property
    }
    pub const fn getValue(&self) -> i16 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::write_i16_be;

    #[test]
    fn protocol_340_layout_matches_mcp() {
        let mut payload = vec![7];
        write_i16_be(2, &mut payload);
        write_i16_be(160, &mut payload);
        let packet = SPacketWindowProperty::readPacketData(&RawPacket::new(0x15, payload)).unwrap();
        assert_eq!(packet.getWindowId(), 7);
        assert_eq!(packet.getProperty(), 2);
        assert_eq!(packet.getValue(), 160);
    }
}
