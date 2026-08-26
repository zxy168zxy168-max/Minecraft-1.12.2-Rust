use crate::net::minecraft::client::gui::inventory::GuiContainer::{GuiContainer, GuiSlot};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `GuiChest` geometry for `textures/gui/container/generic_54.png`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiChest {
    pub container: GuiContainer,
    pub inventoryRows: i32,
}

impl GuiChest {
    pub const X_SIZE: i32 = 176;

    pub fn new(inventoryRows: i32) -> Self {
        // MCP stores `lowerInv.getSizeInventory() / 9` directly. Vanilla
        // does not clamp this value in `GuiChest`.
        let inventoryRows = inventoryRows.max(0);
        let ySize = 114 + inventoryRows * 18;
        let offset = (inventoryRows - 4) * 18;
        let lowerSlots = inventoryRows * 9;
        let mut slots = Vec::with_capacity((lowerSlots + 36) as usize);
        for row in 0..inventoryRows {
            for column in 0..9 {
                slots.push(GuiSlot {
                    slotNumber: column + row * 9,
                    xPos: 8 + column * 18,
                    yPos: 18 + row * 18,
                });
            }
        }
        for row in 0..3 {
            for column in 0..9 {
                slots.push(GuiSlot {
                    slotNumber: lowerSlots + column + row * 9,
                    xPos: 8 + column * 18,
                    yPos: 103 + row * 18 + offset,
                });
            }
        }
        for column in 0..9 {
            slots.push(GuiSlot {
                slotNumber: lowerSlots + 27 + column,
                xPos: 8 + column * 18,
                yPos: 161 + offset,
            });
        }
        Self {
            container: GuiContainer::new(Self::X_SIZE, ySize, slots),
            inventoryRows,
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.container.initGui(width, height);
    }

    pub fn slotAt(&self, mouseX: i32, mouseY: i32) -> Option<i32> {
        self.container.slotAt(mouseX, mouseY)
    }

    pub fn slotPosition(&self, slotNumber: i32) -> Option<(i32, i32)> {
        self.container.slotPosition(slotNumber)
    }

    pub fn chestBackground() -> ResourceLocation {
        ResourceLocation::parse("textures/gui/container/generic_54.png")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_row_chest_matches_mcp_geometry() {
        let mut gui = GuiChest::new(3);
        gui.initGui(320, 240);
        assert_eq!(gui.container.ySize, 168);
        assert_eq!(
            gui.slotPosition(0),
            Some((gui.container.guiLeft + 8, gui.container.guiTop + 18))
        );
        assert_eq!(
            gui.slotPosition(27),
            Some((gui.container.guiLeft + 8, gui.container.guiTop + 103 - 18))
        );
        assert_eq!(
            gui.slotPosition(54),
            Some((gui.container.guiLeft + 8, gui.container.guiTop + 161 - 18))
        );
    }
    #[test]
    fn six_row_double_chest_matches_mcp_geometry() {
        let mut gui = GuiChest::new(6);
        gui.initGui(426, 240);
        assert_eq!(gui.container.ySize, 222);
        assert_eq!(
            gui.slotPosition(53),
            Some((gui.container.guiLeft + 152, gui.container.guiTop + 108))
        );
        assert_eq!(
            gui.slotPosition(54),
            Some((gui.container.guiLeft + 8, gui.container.guiTop + 139))
        );
        assert_eq!(
            gui.slotPosition(81),
            Some((gui.container.guiLeft + 8, gui.container.guiTop + 197))
        );
    }
}
