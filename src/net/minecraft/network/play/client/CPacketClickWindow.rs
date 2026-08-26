use crate::net::minecraft::inventory::ClickType::ClickType;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{write_i16_be, write_var_i32, CodecError};

/// Protocol-340 port of MCP 1.12.2 `CPacketClickWindow`.
#[derive(Debug, Clone, PartialEq)]
pub struct CPacketClickWindow {
    pub windowId: i8,
    pub slotId: i16,
    pub usedButton: i8,
    pub actionNumber: i16,
    pub clickedItem: ItemStack,
    pub mode: ClickType,
}

impl CPacketClickWindow {
    pub fn new(
        windowIdIn: i32,
        slotIdIn: i32,
        usedButtonIn: i32,
        modeIn: ClickType,
        clickedItemIn: &ItemStack,
        actionNumberIn: i16,
    ) -> Self {
        Self {
            windowId: windowIdIn as i8,
            slotId: slotIdIn as i16,
            usedButton: usedButtonIn as i8,
            actionNumber: actionNumberIn,
            clickedItem: clickedItemIn.copy(),
            mode: modeIn,
        }
    }

    pub fn writePacketData(&self) -> Result<RawPacket, CodecError> {
        let mut payload = Vec::new();
        payload.push(self.windowId as u8);
        write_i16_be(self.slotId, &mut payload);
        payload.push(self.usedButton as u8);
        write_i16_be(self.actionNumber, &mut payload);
        write_var_i32(self.mode.ordinal(), &mut payload);
        self.clickedItem.writeToBuffer(&mut payload)?;
        Ok(RawPacket::new(0x07, payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_340_layout_and_id_match() {
        let packet = CPacketClickWindow::new(
            0,
            36,
            0,
            ClickType::Pickup,
            &ItemStack {
                itemId: 339,
                count: 1,
                itemDamage: 0,
                tagCompound: None,
            },
            7,
        )
        .writePacketData()
        .unwrap();
        assert_eq!(packet.id, 0x07);
        assert_eq!(packet.payload[0], 0);
        assert_eq!(&packet.payload[1..3], &36_i16.to_be_bytes());
    }
}
