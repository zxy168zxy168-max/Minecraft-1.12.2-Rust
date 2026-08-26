use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::inventory::ContainerWindow::{ContainerWindow, ContainerWindowKind};
use crate::net::minecraft::inventory::InventoryMerchant::InventoryMerchant;
use crate::net::minecraft::network::PacketBuffer::CodecError;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;
use crate::net::minecraft::village::MerchantRecipeList::MerchantRecipeList;

/// MCP 1.12.2 `ContainerMerchant`: three merchant slots followed by the normal
/// 27-slot inventory and 9-slot hotbar.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerMerchant {
    state: ContainerWindow,
    merchantInventory: InventoryMerchant,
}
impl ContainerMerchant {
    pub fn new(
        windowId: i32,
        title: ITextComponent,
        reportedSlotCount: usize,
        playerInventory: &InventoryPlayer,
    ) -> Result<Self, CodecError> {
        ContainerWindow::new(
            windowId,
            title,
            reportedSlotCount,
            playerInventory,
            ContainerWindowKind::Merchant,
        )
        .map(|state| Self {
            state,
            merchantInventory: InventoryMerchant::default(),
        })
    }
    pub const fn state(&self) -> &ContainerWindow {
        &self.state
    }
    pub fn stateMut(&mut self) -> &mut ContainerWindow {
        &mut self.state
    }
    pub fn getRecipes(&self) -> Option<&MerchantRecipeList> {
        self.merchantInventory.getRecipes()
    }
    pub fn setRecipes(&mut self, recipes: MerchantRecipeList) {
        self.merchantInventory.setRecipes(recipes);
        self.resetRecipeAndSlots();
    }
    pub const fn getCurrentRecipeIndex(&self) -> i32 {
        self.merchantInventory.getCurrentRecipeIndex()
    }
    pub fn setCurrentRecipeIndex(&mut self, index: i32) {
        self.merchantInventory.setCurrentRecipeIndex(index);
        self.resetRecipeAndSlots();
    }
    pub fn resetRecipeAndSlots(&mut self) {
        let first = self.state.getSlot(0).cloned().unwrap_or_default();
        let second = self.state.getSlot(1).cloned().unwrap_or_default();
        let preview = self.merchantInventory.resetRecipeAndSlots(&first, &second);
        let _ = self.state.putStackInSlot(2, preview);
    }
}
