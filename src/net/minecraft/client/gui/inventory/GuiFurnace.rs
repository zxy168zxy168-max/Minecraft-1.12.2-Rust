use crate::net::minecraft::client::gui::inventory::GuiContainer::{GuiContainer, GuiSlot};
use crate::net::minecraft::client::gui::inventory::GuiCrafting::append_player_slots;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `GuiFurnace`/`ContainerFurnace` fixed slot geometry and progress scaling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiFurnace {
    pub container: GuiContainer,
}

impl GuiFurnace {
    pub const X_SIZE: i32 = 176;
    pub const Y_SIZE: i32 = 166;

    pub fn new() -> Self {
        let mut slots = vec![
            GuiSlot {
                slotNumber: 0,
                xPos: 56,
                yPos: 17,
            },
            GuiSlot {
                slotNumber: 1,
                xPos: 56,
                yPos: 53,
            },
            GuiSlot {
                slotNumber: 2,
                xPos: 116,
                yPos: 35,
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

    pub fn furnaceBackground() -> ResourceLocation {
        ResourceLocation::parse("textures/gui/container/furnace.png")
    }

    pub fn cookProgressScaled(properties: &[i32], pixels: i32) -> i32 {
        let cook = properties.get(2).copied().unwrap_or(0);
        let total = properties.get(3).copied().unwrap_or(0);
        if total != 0 && cook != 0 {
            cook * pixels / total
        } else {
            0
        }
    }

    pub fn burnLeftScaled(properties: &[i32], pixels: i32) -> i32 {
        let burn = properties.first().copied().unwrap_or(0);
        let mut current = properties.get(1).copied().unwrap_or(0);
        if current == 0 {
            current = 200;
        }
        burn * pixels / current
    }

    pub fn isBurning(properties: &[i32]) -> bool {
        properties.first().copied().unwrap_or(0) > 0
    }
}

impl Default for GuiFurnace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_and_progress_match_mcp() {
        let mut gui = GuiFurnace::new();
        gui.initGui(320, 240);
        assert_eq!(gui.container.inventorySlots.len(), 39);
        assert_eq!(gui.container.slotPosition(0), Some((128, 54)));
        assert_eq!(gui.container.slotPosition(2), Some((188, 72)));
        assert_eq!(GuiFurnace::cookProgressScaled(&[0, 0, 100, 200], 24), 12);
        assert_eq!(GuiFurnace::burnLeftScaled(&[50, 100, 0, 0], 13), 6);
    }
}
