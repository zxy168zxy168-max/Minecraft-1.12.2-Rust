use crate::net::minecraft::client::gui::inventory::GuiContainer::{GuiContainer, GuiSlot};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `GuiBeacon` fixed container geometry and effect-button layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiBeacon {
    pub container: GuiContainer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeaconPowerButton {
    pub effectId: i32,
    pub tier: i32,
    pub x: i32,
    pub y: i32,
}

impl GuiBeacon {
    pub const X_SIZE: i32 = 230;
    pub const Y_SIZE: i32 = 219;
    pub const CONFIRM_X: i32 = 164;
    pub const CANCEL_X: i32 = 190;
    pub const ACTION_Y: i32 = 107;

    pub fn new() -> Self {
        let mut slots = Vec::with_capacity(37);
        slots.push(GuiSlot {
            slotNumber: 0,
            xPos: 136,
            yPos: 110,
        });
        for row in 0..3 {
            for column in 0..9 {
                slots.push(GuiSlot {
                    slotNumber: 1 + column + row * 9,
                    xPos: 36 + column * 18,
                    yPos: 137 + row * 18,
                });
            }
        }
        for column in 0..9 {
            slots.push(GuiSlot {
                slotNumber: 28 + column,
                xPos: 36 + column * 18,
                yPos: 195,
            });
        }
        Self {
            container: GuiContainer::new(Self::X_SIZE, Self::Y_SIZE, slots),
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.container.initGui(width, height);
    }

    pub fn beaconBackground() -> ResourceLocation {
        ResourceLocation::parse("textures/gui/container/beacon.png")
    }

    pub fn powerButtons(&self, primaryEffect: i32) -> Vec<BeaconPowerButton> {
        let tiers: [&[i32]; 4] = [&[1, 3], &[11, 8], &[5], &[10]];
        let mut buttons = Vec::new();
        for tier in 0..=2_i32 {
            let effects = tiers[tier as usize];
            let width = effects.len() as i32 * 22 + (effects.len() as i32 - 1) * 2;
            for (index, effectId) in effects.iter().copied().enumerate() {
                buttons.push(BeaconPowerButton {
                    effectId,
                    tier,
                    x: self.container.guiLeft + 76 + index as i32 * 24 - width / 2,
                    y: self.container.guiTop + 22 + tier * 25,
                });
            }
        }
        // TileEntityBeacon.EFFECTS_LIST[3].length + 1 is always two slots.
        // The second button is only instantiated when a primary potion exists,
        // but the centering width still reserves both positions in vanilla.
        let secondaryCount = 2;
        let width = secondaryCount * 22 + (secondaryCount - 1) * 2;
        buttons.push(BeaconPowerButton {
            effectId: 10,
            tier: 3,
            x: self.container.guiLeft + 167 - width / 2,
            y: self.container.guiTop + 47,
        });
        if primaryEffect > 0 {
            buttons.push(BeaconPowerButton {
                effectId: primaryEffect,
                tier: 3,
                x: self.container.guiLeft + 167 + 24 - width / 2,
                y: self.container.guiTop + 47,
            });
        }
        buttons
    }

    pub fn powerButtonAt(
        &self,
        mouseX: i32,
        mouseY: i32,
        levels: i32,
        primaryEffect: i32,
    ) -> Option<BeaconPowerButton> {
        self.powerButtons(primaryEffect).into_iter().find(|button| {
            mouseX >= button.x
                && mouseX < button.x + 22
                && mouseY >= button.y
                && mouseY < button.y + 22
                && button.tier < levels
        })
    }

    /// MCP `Potion#getStatusIconIndex` for the six effects available to a
    /// 1.12.2 beacon. The atlas coordinates are consumed by `GuiBeacon.Button`.
    pub const fn effectIconIndex(effectId: i32) -> Option<i32> {
        match effectId {
            1 => Some(0),   // speed: (0, 0)
            3 => Some(2),   // haste: (2, 0)
            5 => Some(4),   // strength: (4, 0)
            8 => Some(10),  // jump boost: (2, 1)
            10 => Some(7),  // regeneration: (7, 0)
            11 => Some(14), // resistance: (6, 1)
            _ => None,
        }
    }

    /// Source X in `beacon.png` for MCP `GuiBeacon.Button#drawButton`.
    pub const fn buttonSourceX(enabled: bool, selected: bool, hovered: bool) -> i32 {
        if !enabled {
            44
        } else if selected {
            22
        } else if hovered {
            66
        } else {
            0
        }
    }

    pub fn confirmEnabled(
        payment: Option<&crate::net::minecraft::item::ItemStack::ItemStack>,
        primaryEffect: i32,
    ) -> bool {
        payment.is_some_and(|stack| !stack.isEmpty()) && primaryEffect > 0
    }

    pub const fn effectNameKey(effectId: i32) -> Option<&'static str> {
        match effectId {
            1 => Some("effect.moveSpeed"),
            3 => Some("effect.digSpeed"),
            5 => Some("effect.damageBoost"),
            8 => Some("effect.jump"),
            10 => Some("effect.regeneration"),
            11 => Some("effect.resistance"),
            _ => None,
        }
    }

    pub fn confirmAt(&self, mouseX: i32, mouseY: i32) -> bool {
        self.container
            .isPointInRegion(Self::CONFIRM_X, Self::ACTION_Y, 22, 22, mouseX, mouseY)
    }

    pub fn cancelAt(&self, mouseX: i32, mouseY: i32) -> bool {
        self.container
            .isPointInRegion(Self::CANCEL_X, Self::ACTION_Y, 22, 22, mouseX, mouseY)
    }
}

impl Default for GuiBeacon {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_matches_gui_beacon_and_container_beacon() {
        let mut gui = GuiBeacon::new();
        gui.initGui(400, 300);
        assert_eq!(gui.container.inventorySlots.len(), 37);
        assert_eq!(gui.container.slotPosition(0), Some((221, 150)));
        assert_eq!(gui.container.slotPosition(1), Some((121, 177)));
        assert_eq!(gui.container.slotPosition(28), Some((121, 235)));
        assert!(gui
            .powerButtons(1)
            .iter()
            .any(|button| button.effectId == 10));
    }

    #[test]
    fn button_texture_states_match_gui_beacon_button() {
        assert_eq!(GuiBeacon::buttonSourceX(true, false, false), 0);
        assert_eq!(GuiBeacon::buttonSourceX(true, true, false), 22);
        assert_eq!(GuiBeacon::buttonSourceX(false, false, true), 44);
        assert_eq!(GuiBeacon::buttonSourceX(true, false, true), 66);
    }
}
