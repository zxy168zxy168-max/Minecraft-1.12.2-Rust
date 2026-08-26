use crate::net::minecraft::client::gui::inventory::GuiContainer::{GuiContainer, GuiSlot};
use crate::net::minecraft::client::gui::inventory::GuiCrafting::append_player_slots;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::village::MerchantRecipeList::MerchantRecipeList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MerchantButtonState {
    pub forward: bool,
    pub x: i32,
    pub y: i32,
    pub enabled: bool,
    pub hovered: bool,
}
impl MerchantButtonState {
    /// Exact `GuiMerchant.MerchantButton#drawButton` source coordinates.
    pub const fn source(self) -> (i32, i32) {
        let mut u = 176;
        if !self.enabled {
            u += 24;
        } else if self.hovered {
            u += 12;
        }
        let v = if self.forward { 0 } else { 19 };
        (u, v)
    }
}

/// MCP 1.12.2 `GuiMerchant` geometry and selected recipe state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiMerchant {
    pub container: GuiContainer,
    selectedMerchantRecipe: i32,
}
impl GuiMerchant {
    pub const X_SIZE: i32 = 176;
    pub const Y_SIZE: i32 = 166;
    pub fn new() -> Self {
        let mut slots = vec![
            GuiSlot {
                slotNumber: 0,
                xPos: 36,
                yPos: 53,
            },
            GuiSlot {
                slotNumber: 1,
                xPos: 62,
                yPos: 53,
            },
            GuiSlot {
                slotNumber: 2,
                xPos: 120,
                yPos: 53,
            },
        ];
        append_player_slots(&mut slots, 3);
        Self {
            container: GuiContainer::new(Self::X_SIZE, Self::Y_SIZE, slots),
            selectedMerchantRecipe: 0,
        }
    }
    pub fn initGui(&mut self, width: i32, height: i32) {
        self.container.initGui(width, height);
    }
    pub fn merchantBackground() -> ResourceLocation {
        ResourceLocation::parse("textures/gui/container/villager.png")
    }
    pub const fn selectedMerchantRecipe(&self) -> i32 {
        self.selectedMerchantRecipe
    }
    pub fn setSelectedMerchantRecipe(&mut self, index: i32, recipes: Option<&MerchantRecipeList>) {
        let maximum = recipes.map_or(0, |r| r.len().saturating_sub(1) as i32);
        self.selectedMerchantRecipe = index.clamp(0, maximum);
    }
    pub fn previousButton(
        &self,
        mouseX: i32,
        mouseY: i32,
        recipes: Option<&MerchantRecipeList>,
    ) -> MerchantButtonState {
        let x = self.container.guiLeft + 17;
        let y = self.container.guiTop + 23;
        MerchantButtonState {
            forward: false,
            x,
            y,
            enabled: self.selectedMerchantRecipe > 0 && recipes.is_some_and(|r| !r.isEmpty()),
            hovered: mouseX >= x && mouseX < x + 12 && mouseY >= y && mouseY < y + 19,
        }
    }
    pub fn nextButton(
        &self,
        mouseX: i32,
        mouseY: i32,
        recipes: Option<&MerchantRecipeList>,
    ) -> MerchantButtonState {
        let x = self.container.guiLeft + 147;
        let y = self.container.guiTop + 23;
        MerchantButtonState {
            forward: true,
            x,
            y,
            enabled: recipes.is_some_and(|r| self.selectedMerchantRecipe < (r.len() as i32 - 1)),
            hovered: mouseX >= x && mouseX < x + 12 && mouseY >= y && mouseY < y + 19,
        }
    }
    pub fn buttonDeltaAt(
        &self,
        mouseX: i32,
        mouseY: i32,
        recipes: Option<&MerchantRecipeList>,
    ) -> Option<i32> {
        let previous = self.previousButton(mouseX, mouseY, recipes);
        if previous.enabled && previous.hovered {
            return Some(-1);
        }
        let next = self.nextButton(mouseX, mouseY, recipes);
        if next.enabled && next.hovered {
            return Some(1);
        }
        None
    }
    pub fn previewRegionAt(&self, mouseX: i32, mouseY: i32) -> Option<MerchantPreviewRegion> {
        let region = |x, y, w, h| self.container.isPointInRegion(x, y, w, h, mouseX, mouseY);
        if region(36, 24, 16, 16) {
            Some(MerchantPreviewRegion::FirstBuy)
        } else if region(62, 24, 16, 16) {
            Some(MerchantPreviewRegion::SecondBuy)
        } else if region(120, 24, 16, 16) {
            Some(MerchantPreviewRegion::Sell)
        } else if region(83, 21, 28, 21) || region(83, 51, 28, 21) {
            Some(MerchantPreviewRegion::Disabled)
        } else {
            None
        }
    }
}
impl Default for GuiMerchant {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MerchantPreviewRegion {
    FirstBuy,
    SecondBuy,
    Sell,
    Disabled,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_slot_and_button_geometry() {
        let mut g = GuiMerchant::new();
        g.initGui(320, 240);
        assert_eq!(g.container.slotPosition(0), Some((108, 90)));
        assert_eq!(g.container.slotPosition(2), Some((192, 90)));
        let p = g.previousButton(89, 60, None);
        assert_eq!(p.source(), (200, 19));
    }
}
