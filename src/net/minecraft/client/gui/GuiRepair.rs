use crate::net::minecraft::client::gui::inventory::GuiContainer::{GuiContainer, GuiSlot};
use crate::net::minecraft::client::gui::inventory::GuiCrafting::append_player_slots;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `GuiRepair`/`ContainerRepair` fixed slot and text-field geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiRepair {
    pub container: GuiContainer,
}

impl GuiRepair {
    pub const X_SIZE: i32 = 176;
    pub const Y_SIZE: i32 = 166;
    pub const NAME_X: i32 = 62;
    pub const NAME_Y: i32 = 24;
    pub const NAME_WIDTH: i32 = 103;
    pub const NAME_HEIGHT: i32 = 12;
    pub const NAME_MAX_LENGTH: usize = 35;

    pub fn new() -> Self {
        let mut slots = vec![
            GuiSlot {
                slotNumber: 0,
                xPos: 27,
                yPos: 47,
            },
            GuiSlot {
                slotNumber: 1,
                xPos: 76,
                yPos: 47,
            },
            GuiSlot {
                slotNumber: 2,
                xPos: 134,
                yPos: 47,
            },
        ];
        append_player_slots(&mut slots, 3);
        Self {
            container: GuiContainer::new(Self::X_SIZE, Self::Y_SIZE, slots),
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.container.initGui(width, height);
    }

    pub fn anvilBackground() -> ResourceLocation {
        ResourceLocation::parse("textures/gui/container/anvil.png")
    }
}

impl Default for GuiRepair {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_match_container_repair() {
        let mut gui = GuiRepair::new();
        gui.initGui(320, 240);
        assert_eq!(gui.container.inventorySlots.len(), 39);
        assert_eq!(gui.container.slotPosition(0), Some((99, 84)));
        assert_eq!(gui.container.slotPosition(2), Some((206, 84)));
    }
}
