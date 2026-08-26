use crate::net::minecraft::client::gui::inventory::GuiContainer::{GuiContainer, GuiSlot};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// Exact MCP 1.12.2 `GuiShulkerBox` geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiShulkerBox {
    pub container: GuiContainer,
}

impl GuiShulkerBox {
    pub const X_SIZE: i32 = 176;
    pub const Y_SIZE: i32 = 167;

    pub fn new() -> Self {
        let mut slots = Vec::with_capacity(63);
        for row in 0..3 {
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
                    slotNumber: 27 + column + row * 9,
                    xPos: 8 + column * 18,
                    yPos: 84 + row * 18,
                });
            }
        }
        for column in 0..9 {
            slots.push(GuiSlot {
                slotNumber: 54 + column,
                xPos: 8 + column * 18,
                yPos: 142,
            });
        }
        Self {
            // GuiContainer defaults to 166 high; GuiShulkerBox increments it.
            container: GuiContainer::new(Self::X_SIZE, Self::Y_SIZE, slots),
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.container.initGui(width, height);
    }

    pub fn shulkerBackground() -> ResourceLocation {
        ResourceLocation::parse("textures/gui/container/shulker_box.png")
    }
}

impl Default for GuiShulkerBox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_matches_gui_shulker_box_and_container_shulker_box() {
        let mut gui = GuiShulkerBox::new();
        gui.initGui(320, 240);
        assert_eq!(gui.container.xSize, 176);
        assert_eq!(gui.container.ySize, 167);
        assert_eq!(gui.container.slotPosition(0), Some((80, 54)));
        assert_eq!(gui.container.slotPosition(26), Some((224, 90)));
        assert_eq!(gui.container.slotPosition(27), Some((80, 120)));
        assert_eq!(gui.container.slotPosition(54), Some((80, 178)));
    }
}
