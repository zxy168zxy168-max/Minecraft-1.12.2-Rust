use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_i32_be, read_string, read_text_component, read_u8, CodecError,
};
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

/// Protocol-340 port of MCP 1.12.2 `SPacketOpenWindow` (clientbound 0x13).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketOpenWindow {
    windowId: u8,
    inventoryType: String,
    windowTitle: ITextComponent,
    slotCount: u8,
    entityId: i32,
}

impl SPacketOpenWindow {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let windowId = read_u8(&mut input)?;
        let inventoryType = read_string(&mut input, 32)?;
        let windowTitle = read_text_component(&mut input)?;
        let slotCount = read_u8(&mut input)?;
        let entityId = if inventoryType == "EntityHorse" {
            read_i32_be(&mut input)?
        } else {
            0
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread open-window bytes",
                input.len()
            )));
        }
        Ok(Self {
            windowId,
            inventoryType,
            windowTitle,
            slotCount,
            entityId,
        })
    }

    pub const fn getWindowId(&self) -> u8 {
        self.windowId
    }
    pub fn getGuiId(&self) -> &str {
        &self.inventoryType
    }
    pub fn getWindowTitle(&self) -> &ITextComponent {
        &self.windowTitle
    }
    pub const fn getSlotCount(&self) -> u8 {
        self.slotCount
    }
    pub const fn getEntityId(&self) -> i32 {
        self.entityId
    }
    pub const fn hasSlots(&self) -> bool {
        self.slotCount > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{write_string, write_var_i32};

    #[test]
    fn reads_vanilla_chest_window() {
        let mut payload = vec![3];
        write_string("minecraft:container", 32, &mut payload).unwrap();
        let title = br#"{"translate":"container.chest"}"#;
        write_var_i32(title.len() as i32, &mut payload);
        payload.extend_from_slice(title);
        payload.push(27);
        let packet = SPacketOpenWindow::readPacketData(&RawPacket::new(0x13, payload)).unwrap();
        assert_eq!(packet.getWindowId(), 3);
        assert_eq!(packet.getGuiId(), "minecraft:container");
        assert_eq!(
            packet.getWindowTitle().getUnformattedText(),
            "container.chest"
        );
        assert_eq!(packet.getSlotCount(), 27);
    }
    #[test]
    fn reads_entity_horse_extra_id() {
        use crate::net::minecraft::network::PacketBuffer::write_i32_be;
        let mut payload = vec![4];
        write_string("EntityHorse", 32, &mut payload).unwrap();
        let title = br#"{"text":"Horse"}"#;
        write_var_i32(title.len() as i32, &mut payload);
        payload.extend_from_slice(title);
        payload.push(17);
        write_i32_be(12345, &mut payload);
        let packet = SPacketOpenWindow::readPacketData(&RawPacket::new(0x13, payload)).unwrap();
        assert_eq!(packet.getWindowId(), 4);
        assert_eq!(packet.getGuiId(), "EntityHorse");
        assert_eq!(packet.getSlotCount(), 17);
        assert_eq!(packet.getEntityId(), 12345);
    }
}
