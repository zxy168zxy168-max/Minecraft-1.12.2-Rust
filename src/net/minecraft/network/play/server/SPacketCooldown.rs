use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_var_i32, CodecError};

/// MCP 1.12.2 `SPacketCooldown` (clientbound 0x17): item registry id + ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketCooldown {
    itemId: i16,
    ticks: i32,
}

impl SPacketCooldown {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let rawItemId = read_var_i32(&mut input)?;
        let ticks = read_var_i32(&mut input)?;
        if !(0..=i16::MAX as i32).contains(&rawItemId) {
            return Err(CodecError::InvalidData(format!("invalid cooldown item id {rawItemId}")));
        }
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread cooldown bytes",
                input.len()
            )));
        }
        Ok(Self { itemId: rawItemId as i16, ticks })
    }

    pub const fn getItemId(&self) -> i16 { self.itemId }
    pub const fn getTicks(&self) -> i32 { self.ticks }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::write_var_i32;

    #[test]
    fn reads_item_and_ticks_as_varints() {
        let mut payload = Vec::new();
        write_var_i32(368, &mut payload);
        write_var_i32(20, &mut payload);
        let packet = SPacketCooldown::readPacketData(&RawPacket::new(0x17, payload)).unwrap();
        assert_eq!(packet.getItemId(), 368);
        assert_eq!(packet.getTicks(), 20);
    }
}
