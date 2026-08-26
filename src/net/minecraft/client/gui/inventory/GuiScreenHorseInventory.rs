use crate::net::minecraft::client::gui::inventory::GuiContainer::{GuiContainer, GuiSlot};
use crate::net::minecraft::inventory::ContainerHorseInventory::HorseInventorySpec;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `GuiScreenHorseInventory` geometry derived from the concrete
/// horse inventory announced by SPacketOpenWindow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiScreenHorseInventory {
    pub container: GuiContainer,
    pub spec: HorseInventorySpec,
}

impl GuiScreenHorseInventory {
    pub const X_SIZE: i32 = 176;
    pub const Y_SIZE: i32 = 166;

    pub fn new(spec: HorseInventorySpec) -> Self {
        let lower = spec.lowerSlotCount() as i32;
        let mut slots = Vec::with_capacity(lower as usize + 36);
        slots.push(GuiSlot {
            slotNumber: 0,
            xPos: 8,
            yPos: 18,
        });
        slots.push(GuiSlot {
            slotNumber: 1,
            xPos: 8,
            yPos: 36,
        });
        if spec.chested {
            let columns = spec.chestColumns.clamp(1, 5);
            for row in 0..3 {
                for column in 0..columns {
                    slots.push(GuiSlot {
                        slotNumber: 2 + column + row * columns,
                        xPos: 80 + column * 18,
                        yPos: 18 + row * 18,
                    });
                }
            }
        }
        for row in 0..3 {
            for column in 0..9 {
                slots.push(GuiSlot {
                    slotNumber: lower + column + row * 9,
                    xPos: 8 + column * 18,
                    yPos: 84 + row * 18,
                });
            }
        }
        for column in 0..9 {
            slots.push(GuiSlot {
                slotNumber: lower + 27 + column,
                xPos: 8 + column * 18,
                yPos: 142,
            });
        }
        Self {
            container: GuiContainer::new(Self::X_SIZE, Self::Y_SIZE, slots),
            spec,
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.container.initGui(width, height);
    }

    pub fn horseBackground() -> ResourceLocation {
        ResourceLocation::parse("textures/gui/container/horse.png")
    }

    pub const fn hasSaddleSlot(&self) -> bool {
        self.spec.kind.canUseSaddleSlot()
    }

    pub const fn hasEquipmentSlot(&self) -> bool {
        self.spec.kind.hasEquipmentSlot()
    }

    pub const fn isLlama(&self) -> bool {
        self.spec.kind.isLlama()
    }

    pub fn chestOverlayWidth(&self) -> i32 {
        if self.spec.chested {
            self.spec.chestColumns.clamp(1, 5) * 18
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::inventory::ContainerHorseInventory::HorseInventoryKind;

    #[test]
    fn chested_donkey_geometry_matches_container_horse_inventory() {
        let mut gui = GuiScreenHorseInventory::new(HorseInventorySpec {
            entityId: 12,
            kind: HorseInventoryKind::Donkey,
            chested: true,
            chestColumns: 5,
        });
        gui.initGui(320, 240);
        assert_eq!(gui.container.inventorySlots.len(), 53);
        assert_eq!(gui.container.slotPosition(0), Some((80, 55)));
        assert_eq!(gui.container.slotPosition(2), Some((152, 55)));
        assert_eq!(gui.container.slotPosition(16), Some((224, 91)));
        assert_eq!(gui.container.slotPosition(17), Some((80, 121)));
        assert_eq!(gui.chestOverlayWidth(), 90);
    }

    #[test]
    fn llama_uses_carpet_slot_without_saddle_overlay() {
        let gui = GuiScreenHorseInventory::new(HorseInventorySpec {
            entityId: 13,
            kind: HorseInventoryKind::Llama,
            chested: false,
            chestColumns: 3,
        });
        assert!(!gui.hasSaddleSlot());
        assert!(gui.hasEquipmentSlot());
        assert!(gui.isLlama());
    }
}
