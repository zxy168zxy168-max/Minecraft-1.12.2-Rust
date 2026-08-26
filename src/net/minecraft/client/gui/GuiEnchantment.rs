use crate::compat::Java::JavaRandom;
use crate::net::minecraft::client::gui::inventory::GuiContainer::{GuiContainer, GuiSlot};
use crate::net::minecraft::client::gui::inventory::GuiCrafting::append_player_slots;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnchantmentBookRenderState {
    pub ticks: f32,
    pub open: f32,
    pub pageFlipRight: f32,
    pub pageFlipLeft: f32,
}

/// MCP 1.12.2 `GuiEnchantment`/`ContainerEnchantment` fixed slot and option geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct GuiEnchantment {
    pub container: GuiContainer,
    pub ticks: i32,
    pub flip: f32,
    pub oFlip: f32,
    pub flipT: f32,
    pub flipA: f32,
    pub open: f32,
    pub oOpen: f32,
    last: ItemStack,
    random: JavaRandom,
}

impl GuiEnchantment {
    pub const X_SIZE: i32 = 176;
    pub const Y_SIZE: i32 = 166;

    pub fn new() -> Self {
        let mut slots = vec![
            GuiSlot {
                slotNumber: 0,
                xPos: 15,
                yPos: 47,
            },
            GuiSlot {
                slotNumber: 1,
                xPos: 35,
                yPos: 47,
            },
        ];
        append_player_slots(&mut slots, 2);
        Self {
            container: GuiContainer::new(Self::X_SIZE, Self::Y_SIZE, slots),
            ticks: 0,
            flip: 0.0,
            oFlip: 0.0,
            flipT: 0.0,
            flipA: 0.0,
            open: 0.0,
            oOpen: 0.0,
            last: ItemStack::EMPTY,
            random: JavaRandom::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |duration| duration.as_nanos() as i64),
            ),
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.container.initGui(width, height);
    }

    pub fn enchantingBackground() -> ResourceLocation {
        ResourceLocation::parse("textures/gui/container/enchanting_table.png")
    }

    pub fn optionAt(&self, mouseX: i32, mouseY: i32) -> Option<i32> {
        (0..3).find(|option| {
            self.container
                .isPointInRegion(60, 14 + 19 * option, 108, 19, mouseX, mouseY)
        })
    }

    /// Exact state update from MCP `GuiEnchantment#tickBook`.
    pub fn tickBook(&mut self, inputStack: &ItemStack, enchantLevels: &[i32]) {
        if inputStack != &self.last {
            self.last = inputStack.clone();
            loop {
                self.flipT +=
                    (self.random.next_i32_bound(4) - self.random.next_i32_bound(4)) as f32;
                if self.flip > self.flipT + 1.0 || self.flip < self.flipT - 1.0 {
                    break;
                }
            }
        }

        self.ticks += 1;
        self.oFlip = self.flip;
        self.oOpen = self.open;
        if enchantLevels.iter().take(3).any(|level| *level != 0) {
            self.open += 0.2;
        } else {
            self.open -= 0.2;
        }
        self.open = self.open.clamp(0.0, 1.0);
        let targetAcceleration = ((self.flipT - self.flip) * 0.4).clamp(-0.2, 0.2);
        self.flipA += (targetAcceleration - self.flipA) * 0.9;
        self.flip += self.flipA;
    }

    pub fn bookRenderState(&self, partialTicks: f32) -> EnchantmentBookRenderState {
        let partial = partialTicks.clamp(0.0, 1.0);
        let open = self.oOpen + (self.open - self.oOpen) * partial;
        let flip = self.oFlip + (self.flip - self.oFlip) * partial;
        let page =
            |offset: f32| (((flip + offset) - (flip + offset).floor()) * 1.6 - 0.3).clamp(0.0, 1.0);
        EnchantmentBookRenderState {
            ticks: self.ticks as f32 + partial,
            open,
            pageFlipRight: page(0.25),
            pageFlipLeft: page(0.75),
        }
    }
}

impl Default for GuiEnchantment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_and_options_match_mcp() {
        let mut gui = GuiEnchantment::new();
        gui.initGui(320, 240);
        assert_eq!(gui.container.inventorySlots.len(), 38);
        assert_eq!(gui.container.slotPosition(0), Some((87, 84)));
        assert_eq!(
            gui.optionAt(gui.container.guiLeft + 60, gui.container.guiTop + 14),
            Some(0)
        );
    }

    #[test]
    fn book_opens_and_closes_by_point_two_per_tick() {
        let mut gui = GuiEnchantment::new();
        let stack = ItemStack {
            itemId: 1,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        };
        gui.tickBook(&stack, &[1, 0, 0]);
        assert!((gui.open - 0.2).abs() < 1.0e-6);
        for _ in 0..4 {
            gui.tickBook(&stack, &[1, 0, 0]);
        }
        assert!((gui.open - 1.0).abs() < 1.0e-6);
        gui.tickBook(&stack, &[0, 0, 0]);
        assert!((gui.open - 0.8).abs() < 1.0e-6);
    }
}
