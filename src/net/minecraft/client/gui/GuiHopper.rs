use crate::net::minecraft::client::gui::inventory::GuiContainer::{GuiContainer, GuiSlot};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `GuiHopper` fixed geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiHopper {
    pub container: GuiContainer,
}

impl GuiHopper {
    pub const X_SIZE: i32 = 176;
    pub const Y_SIZE: i32 = 133;

    pub fn new() -> Self {
        let mut slots = Vec::with_capacity(41);
        for index in 0..5 {
            slots.push(GuiSlot {
                slotNumber: index,
                xPos: 44 + index * 18,
                yPos: 20,
            });
        }
        for row in 0..3 {
            for column in 0..9 {
                slots.push(GuiSlot {
                    slotNumber: 5 + column + row * 9,
                    xPos: 8 + column * 18,
                    yPos: 51 + row * 18,
                });
            }
        }
        for column in 0..9 {
            slots.push(GuiSlot {
                slotNumber: 32 + column,
                xPos: 8 + column * 18,
                yPos: 109,
            });
        }
        Self {
            container: GuiContainer::new(Self::X_SIZE, Self::Y_SIZE, slots),
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.container.initGui(width, height);
    }

    pub fn hopperBackground() -> ResourceLocation {
        ResourceLocation::parse("textures/gui/container/hopper.png")
    }
}

impl Default for GuiHopper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_matches_gui_hopper_and_container_hopper() {
        let mut gui = GuiHopper::new();
        gui.initGui(320, 240);
        assert_eq!(gui.container.inventorySlots.len(), 41);
        assert_eq!(gui.container.slotPosition(0), Some((116, 73)));
        assert_eq!(gui.container.slotPosition(4), Some((188, 73)));
        assert_eq!(gui.container.slotPosition(5), Some((80, 104)));
        assert_eq!(gui.container.slotPosition(32), Some((80, 162)));
    }
}
