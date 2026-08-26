use crate::net::minecraft::client::gui::inventory::GuiContainer::{GuiContainer, GuiSlot};
use crate::net::minecraft::client::gui::inventory::GuiCrafting::append_player_slots;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `GuiBrewingStand` geometry and property scaling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiBrewingStand {
    pub container: GuiContainer,
}

impl GuiBrewingStand {
    pub const X_SIZE: i32 = 176;
    pub const Y_SIZE: i32 = 166;
    pub const BUBBLE_LENGTHS: [i32; 7] = [29, 24, 20, 16, 11, 6, 0];

    pub fn new() -> Self {
        let mut slots = vec![
            GuiSlot {
                slotNumber: 0,
                xPos: 56,
                yPos: 51,
            },
            GuiSlot {
                slotNumber: 1,
                xPos: 79,
                yPos: 58,
            },
            GuiSlot {
                slotNumber: 2,
                xPos: 102,
                yPos: 51,
            },
            GuiSlot {
                slotNumber: 3,
                xPos: 79,
                yPos: 17,
            },
            GuiSlot {
                slotNumber: 4,
                xPos: 17,
                yPos: 17,
            },
        ];
        append_player_slots(&mut slots, 5);
        Self {
            container: GuiContainer::new(Self::X_SIZE, Self::Y_SIZE, slots),
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.container.initGui(width, height);
    }

    pub fn brewingStandBackground() -> ResourceLocation {
        ResourceLocation::parse("textures/gui/container/brewing_stand.png")
    }

    pub fn fuelWidth(properties: &[i32]) -> i32 {
        let fuel = properties.get(1).copied().unwrap_or(0);
        ((18 * fuel + 19) / 20).clamp(0, 18)
    }

    pub fn brewHeight(properties: &[i32]) -> i32 {
        let brewTime = properties.first().copied().unwrap_or(0);
        if brewTime > 0 {
            (28.0 * (1.0 - brewTime as f32 / 400.0)) as i32
        } else {
            0
        }
    }

    pub fn bubbleHeight(properties: &[i32]) -> i32 {
        let brewTime = properties.first().copied().unwrap_or(0);
        if brewTime > 0 {
            Self::BUBBLE_LENGTHS[(brewTime / 2 % 7) as usize]
        } else {
            0
        }
    }
}

impl Default for GuiBrewingStand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_and_progress_match_mcp() {
        let mut gui = GuiBrewingStand::new();
        gui.initGui(320, 240);
        assert_eq!(gui.container.inventorySlots.len(), 41);
        assert_eq!(gui.container.slotPosition(0), Some((128, 88)));
        assert_eq!(gui.container.slotPosition(4), Some((89, 54)));
        assert_eq!(GuiBrewingStand::fuelWidth(&[0, 20]), 18);
        assert_eq!(GuiBrewingStand::brewHeight(&[200, 0]), 14);
        assert_eq!(GuiBrewingStand::bubbleHeight(&[12, 0]), 0);
    }
}
