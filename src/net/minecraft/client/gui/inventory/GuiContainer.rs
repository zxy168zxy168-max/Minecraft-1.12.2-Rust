use std::collections::BTreeSet;

use crate::net::minecraft::inventory::Container::Container;
use crate::net::minecraft::inventory::ContainerPlayer::{
    playerContainerSlotAccepts, playerContainerSlotLimit,
};
use crate::net::minecraft::item::ItemStack::ItemStack;

/// Geometry, slot hit-testing, and desktop drag-splitting state owned by MCP
/// 1.12.2 `GuiContainer`.
///
/// Rendering remains in the Vulkan GUI pass, but the container screen owns the
/// same `xSize`, `ySize`, `guiLeft`, `guiTop`, slot regions, and QUICK_CRAFT
/// interaction fields as the Java class instead of duplicating those rules in
/// the renderer and input loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuiSlot {
    pub slotNumber: i32,
    pub xPos: i32,
    pub yPos: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiContainer {
    pub xSize: i32,
    pub ySize: i32,
    pub guiLeft: i32,
    pub guiTop: i32,
    pub inventorySlots: Vec<GuiSlot>,
    pub dragSplittingSlots: BTreeSet<i32>,
    pub dragSplitting: bool,
    pub dragSplittingLimit: i32,
    pub dragSplittingButton: i32,
    pub dragSplittingRemnant: i32,
    pub ignoreMouseUp: bool,
    pub doubleClick: bool,
}

impl GuiContainer {
    pub fn new(xSize: i32, ySize: i32, inventorySlots: Vec<GuiSlot>) -> Self {
        Self {
            xSize,
            ySize,
            guiLeft: 0,
            guiTop: 0,
            inventorySlots,
            dragSplittingSlots: BTreeSet::new(),
            dragSplitting: false,
            dragSplittingLimit: 0,
            dragSplittingButton: -1,
            dragSplittingRemnant: 0,
            ignoreMouseUp: false,
            doubleClick: false,
        }
    }

    /// Port of `GuiContainer.initGui`'s centered layout calculation.
    pub fn initGui(&mut self, width: i32, height: i32) {
        self.guiLeft = (width - self.xSize) / 2;
        self.guiTop = (height - self.ySize) / 2;
    }

    /// Port of `GuiContainer.isMouseOverSlot` plus `isPointInRegion`.
    /// Vanilla includes a one-pixel border around the nominal 16x16 slot.
    pub fn slotAt(&self, mouseX: i32, mouseY: i32) -> Option<i32> {
        self.inventorySlots
            .iter()
            .find(|slot| self.isPointInRegion(slot.xPos, slot.yPos, 16, 16, mouseX, mouseY))
            .map(|slot| slot.slotNumber)
    }

    pub fn slotPosition(&self, slotNumber: i32) -> Option<(i32, i32)> {
        self.inventorySlots
            .iter()
            .find(|slot| slot.slotNumber == slotNumber)
            .map(|slot| (self.guiLeft + slot.xPos, self.guiTop + slot.yPos))
    }

    pub fn isPointInRegion(
        &self,
        rectX: i32,
        rectY: i32,
        rectWidth: i32,
        rectHeight: i32,
        pointX: i32,
        pointY: i32,
    ) -> bool {
        let relativeX = pointX - self.guiLeft;
        let relativeY = pointY - self.guiTop;
        relativeX >= rectX - 1
            && relativeX < rectX + rectWidth + 1
            && relativeY >= rectY - 1
            && relativeY < rectY + rectHeight + 1
    }

    /// MCP `GuiContainer.func_193983_c`: only points outside the complete GUI
    /// rectangle are represented as protocol slot `-999`. Empty space inside
    /// the window is slot `-1` and must not drop the cursor stack.
    pub fn isOutsideGui(&self, mouseX: i32, mouseY: i32) -> bool {
        mouseX < self.guiLeft
            || mouseY < self.guiTop
            || mouseX >= self.guiLeft + self.xSize
            || mouseY >= self.guiTop + self.ySize
    }

    pub fn protocolSlotAt(&self, mouseX: i32, mouseY: i32) -> i32 {
        if self.isOutsideGui(mouseX, mouseY) {
            -999
        } else {
            self.slotAt(mouseX, mouseY).unwrap_or(-1)
        }
    }

    pub fn beginDragSplitting(&mut self, mouseButton: i32) -> bool {
        let mode = match mouseButton {
            0 => 0,
            1 => 1,
            2 => 2,
            _ => return false,
        };
        self.dragSplitting = true;
        self.dragSplittingButton = mouseButton;
        self.dragSplittingLimit = mode;
        self.dragSplittingRemnant = 0;
        self.dragSplittingSlots.clear();
        true
    }

    pub fn tryAddDragSplittingSlot(
        &mut self,
        slotId: i32,
        cursor: &ItemStack,
        slots: &[ItemStack],
    ) -> bool {
        self.tryAddDragSplittingSlotWithRules(
            slotId,
            cursor,
            slots,
            playerContainerSlotAccepts,
            playerContainerSlotLimit,
        )
    }

    pub fn tryAddDragSplittingSlotWithRules<A, L>(
        &mut self,
        slotId: i32,
        cursor: &ItemStack,
        slots: &[ItemStack],
        slotAccepts: A,
        slotLimit: L,
    ) -> bool
    where
        A: Fn(i32, &ItemStack) -> bool,
        L: Fn(i32, &ItemStack) -> i32,
    {
        if !self.dragSplitting || cursor.isEmpty() || !(0..slots.len() as i32).contains(&slotId) {
            return false;
        }
        let slotStack = &slots[slotId as usize];
        if (cursor.getCount() > self.dragSplittingSlots.len() as i32
            || self.dragSplittingLimit == 2)
            && Container::canAddItemToSlot(slotStack, cursor, true)
            && slotAccepts(slotId, cursor)
        {
            let inserted = self.dragSplittingSlots.insert(slotId);
            if inserted {
                self.updateDragSplittingWithRules(cursor, slots, slotAccepts, slotLimit);
            }
            inserted
        } else {
            false
        }
    }

    /// MCP `GuiContainer.updateDragSplitting` for the player container.
    pub fn updateDragSplitting(&mut self, cursor: &ItemStack, slots: &[ItemStack]) {
        self.updateDragSplittingWithRules(
            cursor,
            slots,
            playerContainerSlotAccepts,
            playerContainerSlotLimit,
        );
    }

    pub fn updateDragSplittingWithRules<A, L>(
        &mut self,
        cursor: &ItemStack,
        slots: &[ItemStack],
        slotAccepts: A,
        slotLimit: L,
    ) where
        A: Fn(i32, &ItemStack) -> bool,
        L: Fn(i32, &ItemStack) -> i32,
    {
        if cursor.isEmpty() || !self.dragSplitting {
            self.dragSplittingRemnant = 0;
            return;
        }
        if self.dragSplittingLimit == 2 {
            self.dragSplittingRemnant = cursor.getMaxStackSize();
            return;
        }
        self.dragSplittingRemnant = cursor.getCount();
        let selectedCount = self.dragSplittingSlots.len();
        if selectedCount == 0 {
            return;
        }
        for &slotId in &self.dragSplittingSlots {
            let Some(existing) = slots.get(slotId as usize) else {
                continue;
            };
            if !slotAccepts(slotId, cursor) {
                continue;
            }
            let oldCount = if existing.isEmpty() {
                0
            } else {
                existing.getCount()
            };
            let mut preview = cursor.clone();
            Container::computeStackSize(
                selectedCount,
                self.dragSplittingLimit,
                &mut preview,
                oldCount,
            );
            let limit = preview.getMaxStackSize().min(slotLimit(slotId, &preview));
            if preview.getCount() > limit {
                preview.setCount(limit);
            }
            self.dragSplittingRemnant -= preview.getCount() - oldCount;
        }
    }

    pub fn dragPreviewStack(
        &self,
        slotId: i32,
        cursor: &ItemStack,
        slots: &[ItemStack],
    ) -> Option<ItemStack> {
        self.dragPreviewStackWithRules(
            slotId,
            cursor,
            slots,
            playerContainerSlotAccepts,
            playerContainerSlotLimit,
        )
    }

    pub fn dragPreviewStackWithRules<A, L>(
        &self,
        slotId: i32,
        cursor: &ItemStack,
        slots: &[ItemStack],
        slotAccepts: A,
        slotLimit: L,
    ) -> Option<ItemStack>
    where
        A: Fn(i32, &ItemStack) -> bool,
        L: Fn(i32, &ItemStack) -> i32,
    {
        if !self.dragSplitting
            || self.dragSplittingSlots.len() <= 1
            || !self.dragSplittingSlots.contains(&slotId)
            || cursor.isEmpty()
        {
            return None;
        }
        let existing = slots.get(slotId as usize)?;
        if !Container::canAddItemToSlot(existing, cursor, true) || !slotAccepts(slotId, cursor) {
            return None;
        }
        let oldCount = if existing.isEmpty() {
            0
        } else {
            existing.getCount()
        };
        let mut preview = cursor.clone();
        Container::computeStackSize(
            self.dragSplittingSlots.len(),
            self.dragSplittingLimit,
            &mut preview,
            oldCount,
        );
        let limit = preview.getMaxStackSize().min(slotLimit(slotId, &preview));
        if preview.getCount() > limit {
            preview.setCount(limit);
        }
        Some(preview)
    }

    pub fn cancelDragSplitting(&mut self) {
        self.dragSplitting = false;
        self.dragSplittingSlots.clear();
        self.dragSplittingRemnant = 0;
    }

    pub fn resetInteraction(&mut self) {
        self.cancelDragSplitting();
        self.dragSplittingButton = -1;
        self.ignoreMouseUp = false;
        self.doubleClick = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_space_inside_gui_is_not_outside_slot() {
        let mut gui = GuiContainer::new(
            176,
            166,
            vec![GuiSlot {
                slotNumber: 0,
                xPos: 8,
                yPos: 8,
            }],
        );
        gui.initGui(320, 240);
        assert_eq!(gui.protocolSlotAt(gui.guiLeft + 50, gui.guiTop + 50), -1);
        assert_eq!(gui.protocolSlotAt(gui.guiLeft - 1, gui.guiTop + 50), -999);
    }

    #[test]
    fn middle_drag_uses_mode_two_and_container_validates_creative() {
        let mut gui = GuiContainer::new(176, 166, Vec::new());
        assert!(gui.beginDragSplitting(2));
        assert_eq!(gui.dragSplittingLimit, 2);
    }
}
