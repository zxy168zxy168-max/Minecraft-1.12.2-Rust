use crate::net::minecraft::network::Packet::RawPacket;

/// Protocol-340 port of MCP 1.12.2 `CPacketEnchantItem`
/// (serverbound 0x06).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPacketEnchantItem {
    pub windowId: i8,
    pub button: i8,
}

impl CPacketEnchantItem {
    pub const fn new(windowIdIn: i32, buttonIn: i32) -> Self {
        Self {
            windowId: windowIdIn as i8,
            button: buttonIn as i8,
        }
    }

    pub fn writePacketData(self) -> RawPacket {
        RawPacket::new(0x06, vec![self.windowId as u8, self.button as u8])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_340_layout_and_id_match() {
        let packet = CPacketEnchantItem::new(4, 2).writePacketData();
        assert_eq!(packet.id, 0x06);
        assert_eq!(packet.payload, vec![4, 2]);
    }
}
