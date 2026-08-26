use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::inventory::ContainerBeacon::ContainerBeacon;
use crate::net::minecraft::inventory::ContainerBrewingStand::ContainerBrewingStand;
use crate::net::minecraft::inventory::ContainerChest::ContainerChest;
use crate::net::minecraft::inventory::ContainerDispenser::ContainerDispenser;
use crate::net::minecraft::inventory::ContainerEnchantment::ContainerEnchantment;
use crate::net::minecraft::inventory::ContainerFurnace::ContainerFurnace;
use crate::net::minecraft::inventory::ContainerHopper::ContainerHopper;
use crate::net::minecraft::inventory::ContainerHorseInventory::{
    ContainerHorseInventory, HorseInventorySpec,
};
use crate::net::minecraft::inventory::ContainerMerchant::ContainerMerchant;
use crate::net::minecraft::inventory::ContainerRepair::ContainerRepair;
use crate::net::minecraft::inventory::ContainerShulkerBox::ContainerShulkerBox;
use crate::net::minecraft::inventory::ContainerWindow::{ContainerWindow, ContainerWindowKind};
use crate::net::minecraft::inventory::ContainerWorkbench::ContainerWorkbench;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::PacketBuffer::CodecError;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;
use crate::net::minecraft::village::MerchantRecipeList::MerchantRecipeList;

/// Rust sum type for the concrete MCP `EntityPlayer.openContainer` subclasses
/// migrated by the client. Each variant retains its source class identity;
/// `ContainerWindow` is only an internal shared-state helper for fixed-layout
/// slot/property mechanics.
#[derive(Debug, Clone, PartialEq)]
pub enum OpenContainer {
    Chest(ContainerChest),
    ShulkerBox(ContainerShulkerBox),
    Horse(ContainerHorseInventory),
    Workbench(ContainerWorkbench),
    Furnace(ContainerFurnace),
    Repair(ContainerRepair),
    Enchantment(ContainerEnchantment),
    Hopper(ContainerHopper),
    BrewingStand(ContainerBrewingStand),
    Dispenser(ContainerDispenser),
    Dropper(ContainerDispenser),
    Beacon(ContainerBeacon),
    Merchant(ContainerMerchant),
}

impl OpenContainer {
    fn fixed(&self) -> Option<&ContainerWindow> {
        match self {
            Self::Workbench(container) => Some(container.state()),
            Self::Furnace(container) => Some(container.state()),
            Self::Repair(container) => Some(container.state()),
            Self::Enchantment(container) => Some(container.state()),
            Self::Hopper(container) => Some(container.state()),
            Self::BrewingStand(container) => Some(container.state()),
            Self::Dispenser(container) => Some(container.state()),
            Self::Dropper(container) => Some(container.state()),
            Self::Beacon(container) => Some(container.state()),
            Self::Merchant(container) => Some(container.state()),
            Self::Chest(_) | Self::ShulkerBox(_) | Self::Horse(_) => None,
        }
    }

    pub const fn isShulkerBox(&self) -> bool {
        matches!(self, Self::ShulkerBox(_))
    }

    pub const fn isHorseInventory(&self) -> bool {
        matches!(self, Self::Horse(_))
    }

    pub fn horseInventorySpec(&self) -> Option<HorseInventorySpec> {
        match self {
            Self::Horse(container) => Some(container.spec()),
            _ => None,
        }
    }

    pub const fn windowKind(&self) -> Option<ContainerWindowKind> {
        match self {
            Self::Workbench(_) => Some(ContainerWindowKind::Workbench),
            Self::Furnace(_) => Some(ContainerWindowKind::Furnace),
            Self::Repair(_) => Some(ContainerWindowKind::Repair),
            Self::Enchantment(_) => Some(ContainerWindowKind::Enchantment),
            Self::Hopper(_) => Some(ContainerWindowKind::Hopper),
            Self::BrewingStand(_) => Some(ContainerWindowKind::BrewingStand),
            Self::Dispenser(_) => Some(ContainerWindowKind::Dispenser),
            Self::Dropper(_) => Some(ContainerWindowKind::Dropper),
            Self::Beacon(_) => Some(ContainerWindowKind::Beacon),
            Self::Merchant(_) => Some(ContainerWindowKind::Merchant),
            Self::Chest(_) | Self::ShulkerBox(_) | Self::Horse(_) => None,
        }
    }

    pub fn properties(&self) -> &[i32] {
        if let Some(container) = self.fixed() {
            container.properties()
        } else {
            &[]
        }
    }

    pub fn updateProgressBar(&mut self, property: i32, value: i32) -> Result<(), CodecError> {
        match self {
            Self::Workbench(container) => container.stateMut().updateProgressBar(property, value),
            Self::Furnace(container) => container.stateMut().updateProgressBar(property, value),
            Self::Repair(container) => container.stateMut().updateProgressBar(property, value),
            Self::Enchantment(container) => container.stateMut().updateProgressBar(property, value),
            Self::Hopper(container) => container.stateMut().updateProgressBar(property, value),
            Self::BrewingStand(container) => {
                container.stateMut().updateProgressBar(property, value)
            }
            Self::Dispenser(container) => container.stateMut().updateProgressBar(property, value),
            Self::Dropper(container) => container.stateMut().updateProgressBar(property, value),
            Self::Beacon(container) => container.stateMut().updateProgressBar(property, value),
            Self::Merchant(container) => container.stateMut().updateProgressBar(property, value),
            Self::Chest(container) => Err(CodecError::InvalidData(format!(
                "window {} does not expose integer properties",
                container.windowId
            ))),
            Self::ShulkerBox(container) => Err(CodecError::InvalidData(format!(
                "window {} does not expose integer properties",
                container.windowId()
            ))),
            Self::Horse(container) => Err(CodecError::InvalidData(format!(
                "window {} does not expose integer properties",
                container.windowId()
            ))),
        }
    }

    pub fn windowId(&self) -> i32 {
        if let Some(container) = self.fixed() {
            return container.windowId;
        }
        match self {
            Self::Chest(container) => container.windowId,
            Self::ShulkerBox(container) => container.windowId(),
            Self::Horse(container) => container.windowId(),
            _ => unreachable!("fixed container handled above"),
        }
    }

    pub fn guiId(&self) -> &str {
        if let Some(container) = self.fixed() {
            return container.kind.guiId();
        }
        match self {
            Self::Chest(container) => &container.guiId,
            Self::ShulkerBox(_) => "minecraft:shulker_box",
            Self::Horse(_) => "EntityHorse",
            _ => unreachable!("fixed container handled above"),
        }
    }

    pub fn title(&self) -> &ITextComponent {
        if let Some(container) = self.fixed() {
            return &container.title;
        }
        match self {
            Self::Chest(container) => &container.title,
            Self::ShulkerBox(container) => container.title(),
            Self::Horse(container) => container.title(),
            _ => unreachable!("fixed container handled above"),
        }
    }

    pub fn getNumRows(&self) -> usize {
        match self {
            Self::Chest(container) => container.getNumRows(),
            Self::ShulkerBox(container) => container.getNumRows(),
            Self::Horse(container) => container.getNumRows(),
            Self::Workbench(_)
            | Self::Furnace(_)
            | Self::Repair(_)
            | Self::Enchantment(_)
            | Self::Hopper(_)
            | Self::BrewingStand(_)
            | Self::Dispenser(_)
            | Self::Dropper(_)
            | Self::Beacon(_)
            | Self::Merchant(_) => 0,
        }
    }

    pub fn lowerSlotCount(&self) -> usize {
        if let Some(container) = self.fixed() {
            return container.lowerSlotCount();
        }
        match self {
            Self::Chest(container) => container.lowerSlotCount(),
            Self::ShulkerBox(container) => container.lowerSlotCount(),
            Self::Horse(container) => container.lowerSlotCount(),
            _ => unreachable!("fixed container handled above"),
        }
    }

    pub fn slotCount(&self) -> usize {
        if let Some(container) = self.fixed() {
            return container.slotCount();
        }
        match self {
            Self::Chest(container) => container.slotCount(),
            Self::ShulkerBox(container) => container.slotCount(),
            Self::Horse(container) => container.slotCount(),
            _ => unreachable!("fixed container handled above"),
        }
    }

    pub fn slots(&self) -> &[ItemStack] {
        if let Some(container) = self.fixed() {
            return container.slots();
        }
        match self {
            Self::Chest(container) => container.slots(),
            Self::ShulkerBox(container) => container.slots(),
            Self::Horse(container) => container.slots(),
            _ => unreachable!("fixed container handled above"),
        }
    }

    pub fn getSlot(&self, slotId: usize) -> Option<&ItemStack> {
        if let Some(container) = self.fixed() {
            return container.getSlot(slotId);
        }
        match self {
            Self::Chest(container) => container.getSlot(slotId),
            Self::ShulkerBox(container) => container.getSlot(slotId),
            Self::Horse(container) => container.getSlot(slotId),
            _ => unreachable!("fixed container handled above"),
        }
    }

    pub fn isItemValidForSlot(&self, slotId: i32, stack: &ItemStack) -> bool {
        if let Some(container) = self.fixed() {
            return container.isItemValidForSlot(slotId, stack);
        }
        match self {
            Self::Chest(_) => true,
            Self::ShulkerBox(container) => container.isItemValidForSlot(slotId, stack),
            Self::Horse(container) => container.isItemValidForSlot(slotId, stack),
            _ => unreachable!("fixed container handled above"),
        }
    }

    pub fn slotLimit(&self, slotId: i32, stack: &ItemStack) -> i32 {
        if let Some(container) = self.fixed() {
            container.slotLimit(slotId, stack)
        } else {
            stack.getMaxStackSize()
        }
    }

    pub fn putStackInSlot(&mut self, slotId: i32, stack: ItemStack) -> Result<(), CodecError> {
        match self {
            Self::Chest(container) => container.putStackInSlot(slotId, stack),
            Self::ShulkerBox(container) => container.putStackInSlot(slotId, stack),
            Self::Horse(container) => container.putStackInSlot(slotId, stack),
            Self::Workbench(container) => container.stateMut().putStackInSlot(slotId, stack),
            Self::Furnace(container) => container.stateMut().putStackInSlot(slotId, stack),
            Self::Repair(container) => container.stateMut().putStackInSlot(slotId, stack),
            Self::Enchantment(container) => container.stateMut().putStackInSlot(slotId, stack),
            Self::Hopper(container) => container.stateMut().putStackInSlot(slotId, stack),
            Self::BrewingStand(container) => container.stateMut().putStackInSlot(slotId, stack),
            Self::Dispenser(container) => container.stateMut().putStackInSlot(slotId, stack),
            Self::Dropper(container) => container.stateMut().putStackInSlot(slotId, stack),
            Self::Beacon(container) => container.stateMut().putStackInSlot(slotId, stack),
            Self::Merchant(container) => {
                let result = container.stateMut().putStackInSlot(slotId, stack);
                if matches!(slotId, 0 | 1) {
                    container.resetRecipeAndSlots();
                }
                result
            }
        }
    }

    pub fn setAll(&mut self, stacks: &[ItemStack]) -> Result<(), CodecError> {
        match self {
            Self::Chest(container) => container.setAll(stacks),
            Self::ShulkerBox(container) => container.setAll(stacks),
            Self::Horse(container) => container.setAll(stacks),
            Self::Workbench(container) => container.stateMut().setAll(stacks),
            Self::Furnace(container) => container.stateMut().setAll(stacks),
            Self::Repair(container) => container.stateMut().setAll(stacks),
            Self::Enchantment(container) => container.stateMut().setAll(stacks),
            Self::Hopper(container) => container.stateMut().setAll(stacks),
            Self::BrewingStand(container) => container.stateMut().setAll(stacks),
            Self::Dispenser(container) => container.stateMut().setAll(stacks),
            Self::Dropper(container) => container.stateMut().setAll(stacks),
            Self::Beacon(container) => container.stateMut().setAll(stacks),
            Self::Merchant(container) => {
                let result = container.stateMut().setAll(stacks);
                container.resetRecipeAndSlots();
                result
            }
        }
    }

    pub fn syncFromPlayerInventory(&mut self, inventory: &InventoryPlayer) {
        match self {
            Self::Chest(container) => container.syncFromPlayerInventory(inventory),
            Self::ShulkerBox(container) => container.syncFromPlayerInventory(inventory),
            Self::Horse(container) => container.syncFromPlayerInventory(inventory),
            Self::Workbench(container) => container.stateMut().syncFromPlayerInventory(inventory),
            Self::Furnace(container) => container.stateMut().syncFromPlayerInventory(inventory),
            Self::Repair(container) => container.stateMut().syncFromPlayerInventory(inventory),
            Self::Enchantment(container) => container.stateMut().syncFromPlayerInventory(inventory),
            Self::Hopper(container) => container.stateMut().syncFromPlayerInventory(inventory),
            Self::BrewingStand(container) => {
                container.stateMut().syncFromPlayerInventory(inventory)
            }
            Self::Dispenser(container) => container.stateMut().syncFromPlayerInventory(inventory),
            Self::Dropper(container) => container.stateMut().syncFromPlayerInventory(inventory),
            Self::Beacon(container) => container.stateMut().syncFromPlayerInventory(inventory),
            Self::Merchant(container) => container.stateMut().syncFromPlayerInventory(inventory),
        }
    }

    pub fn syncToPlayerInventory(&self, inventory: &mut InventoryPlayer) {
        if let Some(container) = self.fixed() {
            container.syncToPlayerInventory(inventory);
            return;
        }
        match self {
            Self::Chest(container) => container.syncToPlayerInventory(inventory),
            Self::ShulkerBox(container) => container.syncToPlayerInventory(inventory),
            Self::Horse(container) => container.syncToPlayerInventory(inventory),
            _ => unreachable!("fixed container handled above"),
        }
    }

    pub fn getNextTransactionID(&mut self) -> i16 {
        match self {
            Self::Chest(container) => container.getNextTransactionID(),
            Self::ShulkerBox(container) => container.getNextTransactionID(),
            Self::Horse(container) => container.getNextTransactionID(),
            Self::Workbench(container) => container.stateMut().getNextTransactionID(),
            Self::Furnace(container) => container.stateMut().getNextTransactionID(),
            Self::Repair(container) => container.stateMut().getNextTransactionID(),
            Self::Enchantment(container) => container.stateMut().getNextTransactionID(),
            Self::Hopper(container) => container.stateMut().getNextTransactionID(),
            Self::BrewingStand(container) => container.stateMut().getNextTransactionID(),
            Self::Dispenser(container) => container.stateMut().getNextTransactionID(),
            Self::Dropper(container) => container.stateMut().getNextTransactionID(),
            Self::Beacon(container) => container.stateMut().getNextTransactionID(),
            Self::Merchant(container) => container.stateMut().getNextTransactionID(),
        }
    }

    pub fn resetQuickCraft(&mut self) {
        match self {
            Self::Chest(container) => container.resetQuickCraft(),
            Self::ShulkerBox(container) => container.resetQuickCraft(),
            Self::Horse(container) => container.resetQuickCraft(),
            Self::Workbench(container) => container.stateMut().resetQuickCraft(),
            Self::Furnace(container) => container.stateMut().resetQuickCraft(),
            Self::Repair(container) => container.stateMut().resetQuickCraft(),
            Self::Enchantment(container) => container.stateMut().resetQuickCraft(),
            Self::Hopper(container) => container.stateMut().resetQuickCraft(),
            Self::BrewingStand(container) => container.stateMut().resetQuickCraft(),
            Self::Dispenser(container) => container.stateMut().resetQuickCraft(),
            Self::Dropper(container) => container.stateMut().resetQuickCraft(),
            Self::Beacon(container) => container.stateMut().resetQuickCraft(),
            Self::Merchant(container) => container.stateMut().resetQuickCraft(),
        }
    }

    pub fn quickCraft(
        &mut self,
        slotId: i32,
        dragType: i32,
        cursor: &mut ItemStack,
        creative: bool,
    ) -> bool {
        match self {
            Self::Chest(container) => container.quickCraft(slotId, dragType, cursor, creative),
            Self::ShulkerBox(container) => container.quickCraft(slotId, dragType, cursor, creative),
            Self::Horse(container) => container.quickCraft(slotId, dragType, cursor, creative),
            Self::Workbench(container) => container
                .stateMut()
                .quickCraft(slotId, dragType, cursor, creative),
            Self::Furnace(container) => container
                .stateMut()
                .quickCraft(slotId, dragType, cursor, creative),
            Self::Repair(container) => container
                .stateMut()
                .quickCraft(slotId, dragType, cursor, creative),
            Self::Enchantment(container) => container
                .stateMut()
                .quickCraft(slotId, dragType, cursor, creative),
            Self::Hopper(container) => container
                .stateMut()
                .quickCraft(slotId, dragType, cursor, creative),
            Self::BrewingStand(container) => container
                .stateMut()
                .quickCraft(slotId, dragType, cursor, creative),
            Self::Dispenser(container) => container
                .stateMut()
                .quickCraft(slotId, dragType, cursor, creative),
            Self::Dropper(container) => container
                .stateMut()
                .quickCraft(slotId, dragType, cursor, creative),
            Self::Beacon(container) => container
                .stateMut()
                .quickCraft(slotId, dragType, cursor, creative),
            Self::Merchant(container) => container
                .stateMut()
                .quickCraft(slotId, dragType, cursor, creative),
        }
    }

    pub fn transferStackInSlot(&mut self, index: usize) -> ItemStack {
        match self {
            Self::Chest(container) => container.transferStackInSlot(index),
            Self::ShulkerBox(container) => container.transferStackInSlot(index),
            Self::Horse(container) => container.transferStackInSlot(index),
            Self::Workbench(container) => container.stateMut().transferStackInSlot(index),
            Self::Furnace(container) => container.stateMut().transferStackInSlot(index),
            Self::Repair(container) => container.stateMut().transferStackInSlot(index),
            Self::Enchantment(container) => container.stateMut().transferStackInSlot(index),
            Self::Hopper(container) => container.stateMut().transferStackInSlot(index),
            Self::BrewingStand(container) => container.stateMut().transferStackInSlot(index),
            Self::Dispenser(container) => container.stateMut().transferStackInSlot(index),
            Self::Dropper(container) => container.stateMut().transferStackInSlot(index),
            Self::Beacon(container) => container.stateMut().transferStackInSlot(index),
            Self::Merchant(container) => container.stateMut().transferStackInSlot(index),
        }
    }

    pub fn swapWithHotbar(&mut self, slotId: usize, hotbarIndex: usize) -> bool {
        match self {
            Self::Chest(container) => container.swapWithHotbar(slotId, hotbarIndex),
            Self::ShulkerBox(container) => container.swapWithHotbar(slotId, hotbarIndex),
            Self::Horse(container) => container.swapWithHotbar(slotId, hotbarIndex),
            Self::Workbench(container) => container.stateMut().swapWithHotbar(slotId, hotbarIndex),
            Self::Furnace(container) => container.stateMut().swapWithHotbar(slotId, hotbarIndex),
            Self::Repair(container) => container.stateMut().swapWithHotbar(slotId, hotbarIndex),
            Self::Enchantment(container) => {
                container.stateMut().swapWithHotbar(slotId, hotbarIndex)
            }
            Self::Hopper(container) => container.stateMut().swapWithHotbar(slotId, hotbarIndex),
            Self::BrewingStand(container) => {
                container.stateMut().swapWithHotbar(slotId, hotbarIndex)
            }
            Self::Dispenser(container) => container.stateMut().swapWithHotbar(slotId, hotbarIndex),
            Self::Dropper(container) => container.stateMut().swapWithHotbar(slotId, hotbarIndex),
            Self::Beacon(container) => container.stateMut().swapWithHotbar(slotId, hotbarIndex),
            Self::Merchant(container) => container.stateMut().swapWithHotbar(slotId, hotbarIndex),
        }
    }

    pub fn throwFromSlot(&mut self, slotId: usize, wholeStack: bool) -> bool {
        match self {
            Self::Chest(container) => container.throwFromSlot(slotId, wholeStack),
            Self::ShulkerBox(container) => container.throwFromSlot(slotId, wholeStack),
            Self::Horse(container) => container.throwFromSlot(slotId, wholeStack),
            Self::Workbench(container) => container.stateMut().throwFromSlot(slotId, wholeStack),
            Self::Furnace(container) => container.stateMut().throwFromSlot(slotId, wholeStack),
            Self::Repair(container) => container.stateMut().throwFromSlot(slotId, wholeStack),
            Self::Enchantment(container) => container.stateMut().throwFromSlot(slotId, wholeStack),
            Self::Hopper(container) => container.stateMut().throwFromSlot(slotId, wholeStack),
            Self::BrewingStand(container) => container.stateMut().throwFromSlot(slotId, wholeStack),
            Self::Dispenser(container) => container.stateMut().throwFromSlot(slotId, wholeStack),
            Self::Dropper(container) => container.stateMut().throwFromSlot(slotId, wholeStack),
            Self::Beacon(container) => container.stateMut().throwFromSlot(slotId, wholeStack),
            Self::Merchant(container) => container.stateMut().throwFromSlot(slotId, wholeStack),
        }
    }

    pub fn pickupAll(&mut self, cursor: &mut ItemStack, reverse: bool) -> bool {
        match self {
            Self::Chest(container) => container.pickupAll(cursor, reverse),
            Self::ShulkerBox(container) => container.pickupAll(cursor, reverse),
            Self::Horse(container) => container.pickupAll(cursor, reverse),
            Self::Workbench(container) => container.stateMut().pickupAll(cursor, reverse),
            Self::Furnace(container) => container.stateMut().pickupAll(cursor, reverse),
            Self::Repair(container) => container.stateMut().pickupAll(cursor, reverse),
            Self::Enchantment(container) => container.stateMut().pickupAll(cursor, reverse),
            Self::Hopper(container) => container.stateMut().pickupAll(cursor, reverse),
            Self::BrewingStand(container) => container.stateMut().pickupAll(cursor, reverse),
            Self::Dispenser(container) => container.stateMut().pickupAll(cursor, reverse),
            Self::Dropper(container) => container.stateMut().pickupAll(cursor, reverse),
            Self::Beacon(container) => container.stateMut().pickupAll(cursor, reverse),
            Self::Merchant(container) => container.stateMut().pickupAll(cursor, reverse),
        }
    }
    pub fn merchantRecipes(&self) -> Option<&MerchantRecipeList> {
        match self {
            Self::Merchant(container) => container.getRecipes(),
            _ => None,
        }
    }
    pub fn merchantRecipeIndex(&self) -> Option<i32> {
        match self {
            Self::Merchant(container) => Some(container.getCurrentRecipeIndex()),
            _ => None,
        }
    }
    pub fn setMerchantRecipes(&mut self, recipes: MerchantRecipeList) -> bool {
        match self {
            Self::Merchant(container) => {
                container.setRecipes(recipes);
                true
            }
            _ => false,
        }
    }
    pub fn setMerchantRecipeIndex(&mut self, index: i32) -> bool {
        match self {
            Self::Merchant(container) => {
                container.setCurrentRecipeIndex(index);
                true
            }
            _ => false,
        }
    }
}
