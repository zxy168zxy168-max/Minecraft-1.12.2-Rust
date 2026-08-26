use crate::net::minecraft::client::gui::inventory::GuiContainer::{GuiContainer, GuiSlot};
use crate::net::minecraft::client::gui::inventory::GuiCrafting::append_player_slots;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `GuiDispenser` geometry, shared by dispenser and dropper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiDispenser {
    pub container: GuiContainer,
}

impl GuiDispenser {
    pub const X_SIZE: i32 = 176;
    pub const Y_SIZE: i32 = 166;

    pub fn new() -> Self {
        let mut slots = Vec::with_capacity(45);
        for row in 0..3 {
            for column in 0..3 {
                slots.push(GuiSlot {
                    slotNumber: column + row * 3,
                    xPos: 62 + column * 18,
                    yPos: 17 + row * 18,
                });
            }
        }
        append_player_slots(&mut slots, 9);
        Self {
            container: GuiContainer::new(Self::X_SIZE, Self::Y_SIZE, slots),
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.container.initGui(width, height);
    }

    pub fn dispenserBackground() -> ResourceLocation {
        ResourceLocation::parse("textures/gui/container/dispenser.png")
    }
}

impl Default for GuiDispenser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_matches_gui_dispenser() {
        let mut gui = GuiDispenser::new();
        gui.initGui(320, 240);
        assert_eq!(gui.container.inventorySlots.len(), 45);
        assert_eq!(gui.container.slotPosition(0), Some((134, 54)));
        assert_eq!(gui.container.slotPosition(8), Some((170, 90)));
        assert_eq!(gui.container.slotPosition(9), Some((80, 121)));
    }
}
