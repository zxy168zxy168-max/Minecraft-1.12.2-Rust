use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{write_i16_be, CodecError};

/// Protocol-340 port of MCP 1.12.2 `CPacketCreativeInventoryAction`.
#[derive(Debug, Clone, PartialEq)]
pub struct CPacketCreativeInventoryAction {
    pub slotId: i16,
    pub stack: ItemStack,
}

impl CPacketCreativeInventoryAction {
    pub fn new(slotIdIn: i32, stackIn: &ItemStack) -> Self {
        Self {
            slotId: slotIdIn as i16,
            stack: stackIn.copy(),
        }
    }

    pub fn writePacketData(&self) -> Result<RawPacket, CodecError> {
        let mut payload = Vec::new();
        write_i16_be(self.slotId, &mut payload);
        self.stack.writeToBuffer(&mut payload)?;
        Ok(RawPacket::new(0x1B, payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_340_layout_and_id_match() {
        let packet = CPacketCreativeInventoryAction::new(
            36,
            &ItemStack {
                itemId: 1,
                count: 64,
                itemDamage: 0,
                tagCompound: None,
            },
        )
        .writePacketData()
        .unwrap();
        assert_eq!(packet.id, 0x1B);
        assert_eq!(&packet.payload[0..2], &36_i16.to_be_bytes());
        assert_eq!(&packet.payload[2..4], &1_i16.to_be_bytes());
    }
}
