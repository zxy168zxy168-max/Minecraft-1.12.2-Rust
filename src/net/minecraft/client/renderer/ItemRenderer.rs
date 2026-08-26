use crate::net::minecraft::item::EnumAction::EnumAction;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::util::EnumHand::EnumHand;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// Interpolated state consumed by the first-person renderer for one frame.
///
/// The values match the arguments passed by MCP `ItemRenderer` to its two
/// `renderItemInFirstPerson` hand branches. Geometry submission remains a
/// Vulkan concern, but stack/equip transition ownership stays in this class.
#[derive(Debug, Clone, PartialEq)]
pub struct FirstPersonItemRenderState {
    pub itemStackMainHand: ItemStack,
    pub itemStackOffHand: ItemStack,
    /// MCP passes `1 - interpolated equippedProgress` to the transform.
    pub equipOffsetMainHand: f32,
    pub equipOffsetOffHand: f32,
    pub handActive: bool,
    pub activeHand: EnumHand,
    pub activeUseAction: EnumAction,
    pub itemInUseCount: i32,
    pub activeMaxUseDuration: i32,
}

impl Default for FirstPersonItemRenderState {
    fn default() -> Self {
        Self {
            itemStackMainHand: ItemStack::EMPTY,
            itemStackOffHand: ItemStack::EMPTY,
            equipOffsetMainHand: 1.0,
            equipOffsetOffHand: 1.0,
            handActive: false,
            activeHand: EnumHand::MainHand,
            activeUseAction: EnumAction::None,
            itemInUseCount: 0,
            activeMaxUseDuration: 0,
        }
    }
}

/// State-bearing Rust equivalent of MCP 1.12.2 `ItemRenderer`.
///
/// This type deliberately owns only the persistent equipped-stack transition
/// state. The baked-model resolver remains `RenderItem`, mirroring the
/// original class split instead of folding item semantics into Vulkan code.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemRenderer {
    itemStackMainHand: ItemStack,
    itemStackOffHand: ItemStack,
    equippedProgressMainHand: f32,
    prevEquippedProgressMainHand: f32,
    equippedProgressOffHand: f32,
    prevEquippedProgressOffHand: f32,
    handActive: bool,
    activeHand: EnumHand,
    activeUseAction: EnumAction,
    itemInUseCount: i32,
    activeMaxUseDuration: i32,
}

impl Default for ItemRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemRenderer {
    /// MCP 1.12.2 `ItemRenderer.RES_MAP_BACKGROUND`.
    pub fn mapBackgroundTexture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/map/map_background.png")
    }

    /// Exact port of `ItemRenderer#getMapAngleFromPitch`.
    pub fn getMapAngleFromPitch(pitch: f32) -> f32 {
        let value = (1.0 - pitch / 45.0 + 0.1).clamp(0.0, 1.0);
        -(value * std::f32::consts::PI).cos() * 0.5 + 0.5
    }

    pub const fn new() -> Self {
        Self {
            itemStackMainHand: ItemStack::EMPTY,
            itemStackOffHand: ItemStack::EMPTY,
            equippedProgressMainHand: 0.0,
            prevEquippedProgressMainHand: 0.0,
            equippedProgressOffHand: 0.0,
            prevEquippedProgressOffHand: 0.0,
            handActive: false,
            activeHand: EnumHand::MainHand,
            activeUseAction: EnumAction::None,
            itemInUseCount: 0,
            activeMaxUseDuration: 0,
        }
    }

    /// Port of `ItemRenderer.updateEquippedItem`.
    ///
    /// `cooledAttackStrength` is supplied by the concrete player state. Until
    /// the generic attribute system supplies per-item attack speed, callers use
    /// the player's current concrete value rather than inventing a render-only
    /// timer here.
    pub fn updateEquippedItem(
        &mut self,
        mainHand: &ItemStack,
        offHand: &ItemStack,
        rowingBoat: bool,
        cooledAttackStrength: f32,
    ) {
        self.prevEquippedProgressMainHand = self.equippedProgressMainHand;
        self.prevEquippedProgressOffHand = self.equippedProgressOffHand;

        if rowingBoat {
            self.equippedProgressMainHand = (self.equippedProgressMainHand - 0.4).clamp(0.0, 1.0);
            self.equippedProgressOffHand = (self.equippedProgressOffHand - 0.4).clamp(0.0, 1.0);
        } else {
            let mainTarget = if self.itemStackMainHand == *mainHand {
                let strength = cooledAttackStrength.clamp(0.0, 1.0);
                strength * strength * strength
            } else {
                0.0
            };
            let offTarget = if self.itemStackOffHand == *offHand {
                1.0
            } else {
                0.0
            };
            self.equippedProgressMainHand +=
                (mainTarget - self.equippedProgressMainHand).clamp(-0.4, 0.4);
            self.equippedProgressOffHand +=
                (offTarget - self.equippedProgressOffHand).clamp(-0.4, 0.4);
        }

        if self.equippedProgressMainHand < 0.1 {
            self.itemStackMainHand = mainHand.clone();
        }
        if self.equippedProgressOffHand < 0.1 {
            self.itemStackOffHand = offHand.clone();
        }
    }

    pub fn renderState(&self, partialTicks: f32) -> FirstPersonItemRenderState {
        let partial = partialTicks.clamp(0.0, 1.0);
        let main = self.prevEquippedProgressMainHand
            + (self.equippedProgressMainHand - self.prevEquippedProgressMainHand) * partial;
        let off = self.prevEquippedProgressOffHand
            + (self.equippedProgressOffHand - self.prevEquippedProgressOffHand) * partial;
        FirstPersonItemRenderState {
            itemStackMainHand: self.itemStackMainHand.clone(),
            itemStackOffHand: self.itemStackOffHand.clone(),
            equipOffsetMainHand: 1.0 - main,
            equipOffsetOffHand: 1.0 - off,
            handActive: self.handActive,
            activeHand: self.activeHand,
            activeUseAction: self.activeUseAction,
            itemInUseCount: self.itemInUseCount,
            activeMaxUseDuration: self.activeMaxUseDuration,
        }
    }

    pub fn setActiveItemState(
        &mut self,
        active: bool,
        hand: EnumHand,
        stack: &ItemStack,
        itemInUseCount: i32,
    ) {
        self.handActive = active;
        self.activeHand = hand;
        self.activeUseAction = if active {
            stack.getItemUseAction()
        } else {
            EnumAction::None
        };
        self.itemInUseCount = if active { itemInUseCount.max(0) } else { 0 };
        self.activeMaxUseDuration = if active {
            stack.getMaxItemUseDuration()
        } else {
            0
        };
    }

    pub fn resetEquippedProgressMainHand(&mut self) {
        self.equippedProgressMainHand = 0.0;
    }

    pub fn resetEquippedProgressOffHand(&mut self) {
        self.equippedProgressOffHand = 0.0;
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(id: i16) -> ItemStack {
        ItemStack {
            itemId: id,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        }
    }

    #[test]
    fn map_pitch_angle_matches_vanilla_endpoints() {
        assert!((ItemRenderer::getMapAngleFromPitch(-90.0) - 1.0).abs() < 1.0e-6);
        assert!((ItemRenderer::getMapAngleFromPitch(90.0) - 0.0).abs() < 1.0e-6);
        assert!((ItemRenderer::getMapAngleFromPitch(0.0) - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn changed_main_hand_drops_before_replacing_and_raises_again() {
        let mut renderer = ItemRenderer::new();
        let paper = stack(339);
        renderer.updateEquippedItem(&paper, &ItemStack::EMPTY, false, 1.0);
        assert_eq!(renderer.renderState(1.0).itemStackMainHand, paper);
        renderer.updateEquippedItem(&paper, &ItemStack::EMPTY, false, 1.0);
        assert!((renderer.renderState(1.0).equipOffsetMainHand - 0.6).abs() < 1.0e-6);
        renderer.updateEquippedItem(&paper, &ItemStack::EMPTY, false, 1.0);
        renderer.updateEquippedItem(&paper, &ItemStack::EMPTY, false, 1.0);
        assert!((renderer.renderState(1.0).equipOffsetMainHand).abs() < 1.0e-6);

        let slime = stack(341);
        renderer.updateEquippedItem(&slime, &ItemStack::EMPTY, false, 1.0);
        assert_ne!(renderer.renderState(1.0).itemStackMainHand, slime);
        renderer.updateEquippedItem(&slime, &ItemStack::EMPTY, false, 1.0);
        assert_ne!(renderer.renderState(1.0).itemStackMainHand, slime);
        renderer.updateEquippedItem(&slime, &ItemStack::EMPTY, false, 1.0);
        assert_eq!(renderer.renderState(1.0).itemStackMainHand, slime);
    }
}
