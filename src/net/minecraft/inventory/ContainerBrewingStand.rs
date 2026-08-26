use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::inventory::ContainerWindow::{ContainerWindow, ContainerWindowKind};
use crate::net::minecraft::network::PacketBuffer::CodecError;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

/// MCP 1.12.2 `ContainerBrewingStand` client-side container owner.
///
/// The concrete class identity is retained while `ContainerWindow` provides
/// shared protocol slot, property, and desktop click mechanics.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerBrewingStand {
    state: ContainerWindow,
}

impl ContainerBrewingStand {
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
            ContainerWindowKind::BrewingStand,
        )
        .map(|state| Self { state })
    }

    pub const fn state(&self) -> &ContainerWindow {
        &self.state
    }
    pub fn stateMut(&mut self) -> &mut ContainerWindow {
        &mut self.state
    }
}
