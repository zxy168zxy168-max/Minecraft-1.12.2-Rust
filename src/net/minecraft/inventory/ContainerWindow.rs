use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::inventory::Container::Container;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::PacketBuffer::CodecError;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

/// Rust equivalent of the client-side state shared by the fixed-layout MCP
/// container subclasses opened through `SPacketOpenWindow`.
///
/// The concrete kind remains explicit so slot validity, slot limits, property
/// counts and shift-click routing stay at the same responsibility boundary as
/// `ContainerWorkbench`, `ContainerFurnace`, `ContainerRepair`, and
/// `ContainerEnchantment` rather than being treated as a generic chest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerWindowKind {
    Workbench,
    Furnace,
    Repair,
    Enchantment,
    Hopper,
    BrewingStand,
    Dispenser,
    Dropper,
    Beacon,
    Merchant,
}

impl ContainerWindowKind {
    pub const fn guiId(self) -> &'static str {
        match self {
            Self::Workbench => "minecraft:crafting_table",
            Self::Furnace => "minecraft:furnace",
            Self::Repair => "minecraft:anvil",
            Self::Enchantment => "minecraft:enchanting_table",
            Self::Hopper => "minecraft:hopper",
            Self::BrewingStand => "minecraft:brewing_stand",
            Self::Dispenser => "minecraft:dispenser",
            Self::Dropper => "minecraft:dropper",
            Self::Beacon => "minecraft:beacon",
            Self::Merchant => "minecraft:villager",
        }
    }

    pub const fn lowerSlotCount(self) -> usize {
        match self {
            Self::Workbench => 10,
            Self::Furnace | Self::Repair | Self::Merchant => 3,
            Self::Enchantment => 2,
            Self::Hopper | Self::BrewingStand => 5,
            Self::Dispenser | Self::Dropper => 9,
            Self::Beacon => 1,
        }
    }

    pub const fn propertyCount(self) -> usize {
        match self {
            Self::Furnace => 4,
            Self::Repair => 1,
            Self::Enchantment => 10,
            Self::BrewingStand => 2,
            Self::Beacon => 3,
            Self::Workbench | Self::Hopper | Self::Dispenser | Self::Dropper | Self::Merchant => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContainerWindow {
    pub windowId: i32,
    pub title: ITextComponent,
    pub kind: ContainerWindowKind,
    inventorySlots: Vec<ItemStack>,
    properties: Vec<i32>,
    base: Container,
}

impl ContainerWindow {
    pub fn new(
        windowId: i32,
        title: ITextComponent,
        slotCount: usize,
        playerInventory: &InventoryPlayer,
        kind: ContainerWindowKind,
    ) -> Result<Self, CodecError> {
        // MCP `SPacketOpenWindow` sends zero slots for the fixed-layout
        // `IInteractionObject` screens (crafting, anvil, enchanting), while
        // furnace is opened through `displayGUIChest` and reports its three
        // inventory slots. The concrete client container still allocates its
        // complete MCP slot layout locally.
        let reportedSlots = match kind {
            ContainerWindowKind::Workbench
            | ContainerWindowKind::Repair
            | ContainerWindowKind::Enchantment => 0,
            ContainerWindowKind::Furnace => 3,
            ContainerWindowKind::Hopper | ContainerWindowKind::BrewingStand => 5,
            ContainerWindowKind::Dispenser | ContainerWindowKind::Dropper => 9,
            ContainerWindowKind::Beacon => 1,
            ContainerWindowKind::Merchant => 3,
        };
        if slotCount != reportedSlots {
            return Err(CodecError::InvalidData(format!(
                "{} reports {slotCount} protocol slots; MCP 1.12.2 requires {reportedSlots}",
                kind.guiId()
            )));
        }
        let expected = kind.lowerSlotCount();
        let mut inventorySlots = vec![ItemStack::EMPTY; expected + 36];
        for playerIndex in 9..36 {
            inventorySlots[expected + playerIndex - 9] = playerInventory
                .mainInventory
                .get(playerIndex)
                .cloned()
                .unwrap_or(ItemStack::EMPTY);
        }
        for hotbarIndex in 0..9 {
            inventorySlots[expected + 27 + hotbarIndex] = playerInventory
                .mainInventory
                .get(hotbarIndex)
                .cloned()
                .unwrap_or(ItemStack::EMPTY);
        }
        let mut properties = vec![0; kind.propertyCount()];
        if kind == ContainerWindowKind::Enchantment {
            // ContainerEnchantment initializes enchantClue/worldClue to -1;
            // properties 4..=9 carry those arrays over SPacketWindowProperty.
            properties[4..].fill(-1);
        }
        Ok(Self {
            windowId,
            title,
            kind,
            inventorySlots,
            properties,
            base: Container::default(),
        })
    }

    pub const fn lowerSlotCount(&self) -> usize {
        self.kind.lowerSlotCount()
    }
    pub fn slotCount(&self) -> usize {
        self.inventorySlots.len()
    }
    pub fn slots(&self) -> &[ItemStack] {
        &self.inventorySlots
    }
    pub fn getSlot(&self, slotId: usize) -> Option<&ItemStack> {
        self.inventorySlots.get(slotId)
    }
    pub fn properties(&self) -> &[i32] {
        &self.properties
    }
    pub fn getProperty(&self, property: usize) -> i32 {
        self.properties.get(property).copied().unwrap_or(0)
    }

    pub fn updateProgressBar(&mut self, property: i32, value: i32) -> Result<(), CodecError> {
        let index = usize::try_from(property).map_err(|_| {
            CodecError::InvalidData(format!("negative container property {property}"))
        })?;
        let maximum = self.properties.len().saturating_sub(1);
        let target = self.properties.get_mut(index).ok_or_else(|| {
            CodecError::InvalidData(format!(
                "{} property {property} outside 0..{maximum}",
                self.kind.guiId()
            ))
        })?;
        *target = value;
        Ok(())
    }

    pub fn putStackInSlot(&mut self, slotId: i32, stack: ItemStack) -> Result<(), CodecError> {
        let index = usize::try_from(slotId).map_err(|_| {
            CodecError::InvalidData(format!("negative {} slot {slotId}", self.kind.guiId()))
        })?;
        let maximum = self.inventorySlots.len().saturating_sub(1);
        let slot = self.inventorySlots.get_mut(index).ok_or_else(|| {
            CodecError::InvalidData(format!(
                "{} slot {slotId} outside 0..{maximum}",
                self.kind.guiId()
            ))
        })?;
        *slot = stack;
        Ok(())
    }

    pub fn setAll(&mut self, stacks: &[ItemStack]) -> Result<(), CodecError> {
        if stacks.len() != self.inventorySlots.len() {
            return Err(CodecError::InvalidData(format!(
                "{} stacks for {}-slot {}",
                stacks.len(),
                self.inventorySlots.len(),
                self.kind.guiId()
            )));
        }
        self.inventorySlots.clone_from_slice(stacks);
        Ok(())
    }

    pub fn syncFromPlayerInventory(&mut self, playerInventory: &InventoryPlayer) {
        let lower = self.lowerSlotCount();
        for playerIndex in 9..36 {
            if let (Some(source), Some(target)) = (
                playerInventory.mainInventory.get(playerIndex),
                self.inventorySlots.get_mut(lower + playerIndex - 9),
            ) {
                *target = source.clone();
            }
        }
        for hotbarIndex in 0..9 {
            if let (Some(source), Some(target)) = (
                playerInventory.mainInventory.get(hotbarIndex),
                self.inventorySlots.get_mut(lower + 27 + hotbarIndex),
            ) {
                *target = source.clone();
            }
        }
    }

    pub fn syncToPlayerInventory(&self, playerInventory: &mut InventoryPlayer) {
        let lower = self.lowerSlotCount();
        for playerIndex in 9..36 {
            if let (Some(stack), Some(target)) = (
                self.inventorySlots.get(lower + playerIndex - 9),
                playerInventory.mainInventory.get_mut(playerIndex),
            ) {
                *target = stack.clone();
            }
        }
        for hotbarIndex in 0..9 {
            if let (Some(stack), Some(target)) = (
                self.inventorySlots.get(lower + 27 + hotbarIndex),
                playerInventory.mainInventory.get_mut(hotbarIndex),
            ) {
                *target = stack.clone();
            }
        }
    }

    pub fn isItemValidForSlot(&self, slotId: i32, stack: &ItemStack) -> bool {
        if stack.isEmpty() || !(0..self.slotCount() as i32).contains(&slotId) {
            return false;
        }
        if slotId as usize >= self.lowerSlotCount() {
            return true;
        }
        match self.kind {
            ContainerWindowKind::Workbench => slotId != 0,
            ContainerWindowKind::Furnace => match slotId {
                0 => true,
                1 => isFurnaceFuel(stack),
                2 => false,
                _ => false,
            },
            ContainerWindowKind::Repair => slotId != 2,
            ContainerWindowKind::Enchantment => match slotId {
                0 => true,
                1 => stack.itemId == 351 && stack.itemDamage == 4,
                _ => false,
            },
            ContainerWindowKind::Hopper
            | ContainerWindowKind::Dispenser
            | ContainerWindowKind::Dropper => true,
            ContainerWindowKind::BrewingStand => match slotId {
                0..=2 => canHoldBrewingPotion(stack),
                3 => isBrewingReagent(stack),
                4 => stack.itemId == 377,
                _ => false,
            },
            ContainerWindowKind::Beacon => matches!(stack.itemId, 264 | 265 | 266 | 388),
            ContainerWindowKind::Merchant => slotId != 2,
        }
    }

    pub fn slotLimit(&self, slotId: i32, stack: &ItemStack) -> i32 {
        if (self.kind == ContainerWindowKind::Enchantment && slotId == 0)
            || (self.kind == ContainerWindowKind::BrewingStand && (0..=2).contains(&slotId))
            || (self.kind == ContainerWindowKind::Beacon && slotId == 0)
        {
            1
        } else {
            stack.getMaxStackSize()
        }
    }

    pub fn getNextTransactionID(&mut self) -> i16 {
        self.base.getNextTransactionID()
    }
    pub fn resetQuickCraft(&mut self) {
        self.base.resetDrag();
    }

    pub fn quickCraft(
        &mut self,
        slotId: i32,
        dragType: i32,
        cursor: &mut ItemStack,
        creative: bool,
    ) -> bool {
        let previousEvent = self.base.dragEvent;
        self.base.dragEvent = Container::getDragEvent(dragType);
        if (previousEvent != 1 || self.base.dragEvent != 2) && previousEvent != self.base.dragEvent
        {
            self.resetQuickCraft();
            return false;
        }
        if cursor.isEmpty() {
            self.resetQuickCraft();
            return false;
        }
        match self.base.dragEvent {
            0 => {
                self.base.dragMode = Container::extractDragMode(dragType);
                if Container::isValidDragMode(self.base.dragMode, creative) {
                    self.base.dragEvent = 1;
                    self.base.dragSlots.clear();
                    true
                } else {
                    self.resetQuickCraft();
                    false
                }
            }
            1 => {
                let Ok(index) = usize::try_from(slotId) else {
                    return false;
                };
                let Some(slotStack) = self.inventorySlots.get(index) else {
                    return false;
                };
                if self.isItemValidForSlot(slotId, cursor)
                    && Container::canAddItemToSlot(slotStack, cursor, true)
                    && (self.base.dragMode == 2
                        || cursor.getCount() > self.base.dragSlots.len() as i32)
                {
                    self.base.dragSlots.insert(index)
                } else {
                    false
                }
            }
            2 => {
                let mut changed = false;
                if !self.base.dragSlots.is_empty() {
                    let source = cursor.clone();
                    let mut remaining = cursor.getCount();
                    let slotCount = self.base.dragSlots.len();
                    let selected = self.base.dragSlots.iter().copied().collect::<Vec<_>>();
                    for index in selected {
                        let slotId = index as i32;
                        let existing = self.inventorySlots[index].clone();
                        if !self.isItemValidForSlot(slotId, cursor)
                            || !Container::canAddItemToSlot(&existing, cursor, true)
                            || (self.base.dragMode != 2 && cursor.getCount() < slotCount as i32)
                        {
                            continue;
                        }
                        let oldCount = if existing.isEmpty() {
                            0
                        } else {
                            existing.getCount()
                        };
                        let mut placed = source.clone();
                        Container::computeStackSize(
                            slotCount,
                            self.base.dragMode,
                            &mut placed,
                            oldCount,
                        );
                        let limit = placed
                            .getMaxStackSize()
                            .min(self.slotLimit(slotId, &placed));
                        if placed.getCount() > limit {
                            placed.setCount(limit);
                        }
                        remaining -= placed.getCount() - oldCount;
                        self.inventorySlots[index] = placed;
                        changed = true;
                    }
                    cursor.setCount(remaining);
                }
                self.resetQuickCraft();
                changed
            }
            _ => {
                self.resetQuickCraft();
                false
            }
        }
    }

    fn mergeItemStack(
        &mut self,
        stack: &mut ItemStack,
        startIndex: usize,
        endIndex: usize,
        reverseDirection: bool,
    ) -> bool {
        if stack.isEmpty() || startIndex >= endIndex || endIndex > self.inventorySlots.len() {
            return false;
        }
        let indices: Vec<usize> = if reverseDirection {
            (startIndex..endIndex).rev().collect()
        } else {
            (startIndex..endIndex).collect()
        };
        let mut changed = false;
        if stack.getMaxStackSize() > 1 {
            for &index in &indices {
                if stack.isEmpty() {
                    break;
                }
                if !self.isItemValidForSlot(index as i32, stack) {
                    continue;
                }
                let existing = self.inventorySlots[index].clone();
                if existing.isEmpty() || !existing.canStackWith(stack) {
                    continue;
                }
                let limit = existing
                    .getMaxStackSize()
                    .min(self.slotLimit(index as i32, &existing));
                let capacity = limit - existing.getCount();
                if capacity <= 0 {
                    continue;
                }
                let moved = capacity.min(stack.getCount());
                let mut merged = existing;
                merged.grow(moved);
                stack.shrink(moved);
                self.inventorySlots[index] = merged;
                changed = true;
            }
        }
        for &index in &indices {
            if stack.isEmpty() {
                break;
            }
            if !self.inventorySlots[index].isEmpty()
                || !self.isItemValidForSlot(index as i32, stack)
            {
                continue;
            }
            let moved = stack
                .getMaxStackSize()
                .min(self.slotLimit(index as i32, stack))
                .min(stack.getCount());
            self.inventorySlots[index] = stack.splitStack(moved);
            changed = true;
        }
        changed
    }

    pub fn transferStackInSlot(&mut self, index: usize) -> ItemStack {
        if index >= self.inventorySlots.len() {
            return ItemStack::EMPTY;
        }
        let original = self.inventorySlots[index].clone();
        if original.isEmpty() {
            return ItemStack::EMPTY;
        }
        let mut moving = original.clone();
        let total = self.inventorySlots.len();
        let merged = match self.kind {
            ContainerWindowKind::Workbench => {
                if index == 0 {
                    self.mergeItemStack(&mut moving, 10, 46, true)
                } else if (10..37).contains(&index) {
                    self.mergeItemStack(&mut moving, 37, 46, false)
                } else if (37..46).contains(&index) {
                    self.mergeItemStack(&mut moving, 10, 37, false)
                } else {
                    self.mergeItemStack(&mut moving, 10, 46, false)
                }
            }
            ContainerWindowKind::Furnace => {
                if index == 2 {
                    self.mergeItemStack(&mut moving, 3, 39, true)
                } else if index == 0 || index == 1 {
                    self.mergeItemStack(&mut moving, 3, 39, false)
                } else if isSmeltingInput(&moving) {
                    self.mergeItemStack(&mut moving, 0, 1, false)
                } else if isFurnaceFuel(&moving) {
                    self.mergeItemStack(&mut moving, 1, 2, false)
                } else if (3..30).contains(&index) {
                    self.mergeItemStack(&mut moving, 30, 39, false)
                } else {
                    self.mergeItemStack(&mut moving, 3, 30, false)
                }
            }
            ContainerWindowKind::Repair => {
                if index == 2 {
                    self.mergeItemStack(&mut moving, 3, 39, true)
                } else if index == 0 || index == 1 {
                    self.mergeItemStack(&mut moving, 3, 39, false)
                } else {
                    self.mergeItemStack(&mut moving, 0, 2, false)
                }
            }
            ContainerWindowKind::Enchantment => {
                if index == 0 || index == 1 {
                    self.mergeItemStack(&mut moving, 2, 38, true)
                } else if moving.itemId == 351 && moving.itemDamage == 4 {
                    self.mergeItemStack(&mut moving, 1, 2, true)
                } else if self.inventorySlots[0].isEmpty() {
                    let moved = if moving.tagCompound.is_some() && moving.getCount() == 1 {
                        let copy = moving.clone();
                        moving = ItemStack::EMPTY;
                        copy
                    } else {
                        let copy = ItemStack {
                            itemId: moving.itemId,
                            count: 1,
                            itemDamage: moving.itemDamage,
                            tagCompound: None,
                        };
                        moving.shrink(1);
                        copy
                    };
                    self.inventorySlots[0] = moved;
                    true
                } else {
                    false
                }
            }
            ContainerWindowKind::Hopper => {
                if index < 5 {
                    self.mergeItemStack(&mut moving, 5, 41, true)
                } else {
                    self.mergeItemStack(&mut moving, 0, 5, false)
                }
            }
            ContainerWindowKind::Dispenser | ContainerWindowKind::Dropper => {
                if index < 9 {
                    self.mergeItemStack(&mut moving, 9, 45, true)
                } else {
                    self.mergeItemStack(&mut moving, 0, 9, false)
                }
            }
            ContainerWindowKind::BrewingStand => {
                if index <= 4 {
                    self.mergeItemStack(&mut moving, 5, 41, true)
                } else if isBrewingReagent(&moving) {
                    self.mergeItemStack(&mut moving, 3, 4, false)
                } else if canHoldBrewingPotion(&moving) && moving.getCount() == 1 {
                    self.mergeItemStack(&mut moving, 0, 3, false)
                } else if moving.itemId == 377 {
                    self.mergeItemStack(&mut moving, 4, 5, false)
                } else if (5..32).contains(&index) {
                    self.mergeItemStack(&mut moving, 32, 41, false)
                } else if (32..41).contains(&index) {
                    self.mergeItemStack(&mut moving, 5, 32, false)
                } else {
                    self.mergeItemStack(&mut moving, 5, 41, false)
                }
            }
            ContainerWindowKind::Beacon => {
                if index == 0 {
                    self.mergeItemStack(&mut moving, 1, 37, true)
                } else if self.inventorySlots[0].isEmpty()
                    && self.isItemValidForSlot(0, &moving)
                    && moving.getCount() == 1
                {
                    self.mergeItemStack(&mut moving, 0, 1, false)
                } else if (1..28).contains(&index) {
                    self.mergeItemStack(&mut moving, 28, 37, false)
                } else if (28..37).contains(&index) {
                    self.mergeItemStack(&mut moving, 1, 28, false)
                } else {
                    self.mergeItemStack(&mut moving, 1, 37, false)
                }
            }
            ContainerWindowKind::Merchant => {
                if index == 2 {
                    self.mergeItemStack(&mut moving, 3, 39, true)
                } else if index == 0 || index == 1 {
                    self.mergeItemStack(&mut moving, 3, 39, false)
                } else if (3..30).contains(&index) {
                    self.mergeItemStack(&mut moving, 30, 39, false)
                } else if (30..39).contains(&index) {
                    self.mergeItemStack(&mut moving, 3, 30, false)
                } else {
                    self.mergeItemStack(&mut moving, 3, 39, false)
                }
            }
        };
        if !merged || moving.getCount() == original.getCount() {
            return ItemStack::EMPTY;
        }
        self.inventorySlots[index] = moving;
        debug_assert_eq!(total, self.inventorySlots.len());
        original
    }

    pub fn swapWithHotbar(&mut self, slotId: usize, hotbarIndex: usize) -> bool {
        if slotId >= self.inventorySlots.len() || hotbarIndex >= 9 {
            return false;
        }
        let hotbarSlot = self.lowerSlotCount() + 27 + hotbarIndex;
        if slotId == hotbarSlot {
            return false;
        }
        let hotbarStack = self.inventorySlots[hotbarSlot].clone();
        if !hotbarStack.isEmpty() && !self.isItemValidForSlot(slotId as i32, &hotbarStack) {
            return false;
        }
        self.inventorySlots.swap(slotId, hotbarSlot);
        true
    }

    pub fn throwFromSlot(&mut self, slotId: usize, wholeStack: bool) -> bool {
        let Some(stack) = self.inventorySlots.get_mut(slotId) else {
            return false;
        };
        if stack.isEmpty() {
            return false;
        }
        let amount = if wholeStack { stack.getCount() } else { 1 };
        !stack.splitStack(amount).isEmpty()
    }

    pub fn pickupAll(&mut self, cursor: &mut ItemStack, reverse: bool) -> bool {
        if cursor.isEmpty() || cursor.getCount() >= cursor.getMaxStackSize() {
            return false;
        }
        let indices: Vec<usize> = if reverse {
            (0..self.inventorySlots.len()).rev().collect()
        } else {
            (0..self.inventorySlots.len()).collect()
        };
        let mut changed = false;
        for pass in 0..2 {
            for &index in &indices {
                if cursor.getCount() >= cursor.getMaxStackSize() {
                    break;
                }
                let stack = self.inventorySlots[index].clone();
                if stack.isEmpty() || !stack.canStackWith(cursor) {
                    continue;
                }
                if pass == 0 && stack.getCount() == stack.getMaxStackSize() {
                    continue;
                }
                let moved = (cursor.getMaxStackSize() - cursor.getCount()).min(stack.getCount());
                if moved <= 0 {
                    continue;
                }
                let mut remaining = stack;
                remaining.shrink(moved);
                cursor.grow(moved);
                self.inventorySlots[index] = remaining;
                changed = true;
            }
        }
        changed
    }
}

/// MCP `ContainerBrewingStand.Potion#canHoldPotion`.
pub const fn canHoldBrewingPotion(stack: &ItemStack) -> bool {
    matches!(stack.itemId, 373 | 374 | 438 | 441)
}

/// MCP `PotionHelper#isReagent` for the vanilla 1.12.2 registry.
pub const fn isBrewingReagent(stack: &ItemStack) -> bool {
    matches!(
        stack.itemId,
        289 | 331 | 348 | 353 | 370 | 372 | 375 | 376 | 377 | 378 | 382 | 396 | 414 | 437
    ) || (stack.itemId == 349 && stack.itemDamage == 3)
}

/// Exact vanilla smelting-input set registered by MCP 1.12.2
/// `FurnaceRecipes`, expressed in protocol-340 numeric IDs.
fn isSmeltingInput(stack: &ItemStack) -> bool {
    if stack.isEmpty() {
        return false;
    }
    match stack.itemId {
        14 | 15 | 16 | 21 | 56 | 73 | 81 | 82 | 87 | 129 | 153 | 17 | 162 | 319 | 337 | 363
        | 365 | 392 | 411 | 423 | 432 | 256 | 257 | 258 | 267 | 292 | 302 | 303 | 304 | 305
        | 306 | 307 | 308 | 309 | 417 | 283 | 284 | 285 | 286 | 294 | 314 | 315 | 316 | 317
        | 418 => true,
        4 | 12 => true,
        19 => stack.itemDamage == 1,
        98 => stack.itemDamage == 0,
        159 => (0..=15).contains(&stack.itemDamage),
        349 => matches!(stack.itemDamage, 0 | 1),
        _ => false,
    }
}

/// MCP `TileEntityFurnace.isItemFuel` for the vanilla item registry. This is
/// used only for client-side slot validity and click prediction; the server
/// remains authoritative for burn time and inventory synchronization.
pub fn isFurnaceFuel(stack: &ItemStack) -> bool {
    if stack.isEmpty() {
        return false;
    }
    matches!(
        stack.itemId,
        5 | 6
            | 17
            | 25
            | 35
            | 47
            | 53
            | 54
            | 58
            | 65
            | 72
            | 84
            | 85
            | 96
            | 99
            | 100
            | 107
            | 126
            | 134
            | 135
            | 136
            | 143
            | 146
            | 151
            | 162
            | 163
            | 164
            | 171
            | 173
            | 183
            | 184
            | 185
            | 186
            | 187
            | 188
            | 189
            | 190
            | 191
            | 192
            | 261
            | 263
            | 268
            | 269
            | 270
            | 271
            | 280
            | 281
            | 290
            | 323
            | 324
            | 327
            | 333
            | 346
            | 369
            | 427
            | 428
            | 429
            | 430
            | 431
            | 444
            | 445
            | 446
            | 447
            | 448
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(id: i16, count: u8, damage: i16) -> ItemStack {
        ItemStack {
            itemId: id,
            count,
            itemDamage: damage,
            tagCompound: None,
        }
    }

    #[test]
    fn fixed_layouts_match_mcp_slot_counts() {
        let player = InventoryPlayer::default();
        for (kind, reported, lower) in [
            (ContainerWindowKind::Workbench, 0, 10),
            (ContainerWindowKind::Furnace, 3, 3),
            (ContainerWindowKind::Repair, 0, 3),
            (ContainerWindowKind::Enchantment, 0, 2),
        ] {
            let window = ContainerWindow::new(
                2,
                ITextComponent::fromPlainText(kind.guiId()),
                reported,
                &player,
                kind,
            )
            .unwrap();
            assert_eq!(window.slotCount(), lower + 36);
        }
    }

    #[test]
    fn special_slot_rules_are_not_chest_rules() {
        let player = InventoryPlayer::default();
        let workbench = ContainerWindow::new(
            1,
            ITextComponent::fromPlainText("Crafting"),
            0,
            &player,
            ContainerWindowKind::Workbench,
        )
        .unwrap();
        assert!(!workbench.isItemValidForSlot(0, &stack(1, 1, 0)));

        let enchantment = ContainerWindow::new(
            2,
            ITextComponent::fromPlainText("Enchant"),
            0,
            &player,
            ContainerWindowKind::Enchantment,
        )
        .unwrap();
        assert!(enchantment.isItemValidForSlot(1, &stack(351, 1, 4)));
        assert!(!enchantment.isItemValidForSlot(1, &stack(351, 1, 1)));
        assert_eq!(enchantment.slotLimit(0, &stack(276, 1, 0)), 1);
        assert_eq!(&enchantment.properties()[4..], &[-1, -1, -1, -1, -1, -1]);
    }

    #[test]
    fn furnace_properties_and_slot_rules_match_container_furnace() {
        let mut furnace = ContainerWindow::new(
            3,
            ITextComponent::fromPlainText("Furnace"),
            3,
            &InventoryPlayer::default(),
            ContainerWindowKind::Furnace,
        )
        .unwrap();
        furnace.updateProgressBar(2, 100).unwrap();
        assert_eq!(furnace.getProperty(2), 100);
        assert!(furnace.isItemValidForSlot(1, &stack(263, 1, 0)));
        assert!(!furnace.isItemValidForSlot(2, &stack(1, 1, 0)));
        assert!(isSmeltingInput(&stack(15, 1, 0)));
        assert!(!isSmeltingInput(&stack(280, 1, 0)));
    }
}
