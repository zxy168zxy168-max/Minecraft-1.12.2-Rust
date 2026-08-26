use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::inventory::ContainerWindow::{ContainerWindow, ContainerWindowKind};
use crate::net::minecraft::network::PacketBuffer::CodecError;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

/// MCP 1.12.2 `ContainerFurnace` client-side container owner.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerFurnace {
    state: ContainerWindow,
}

impl ContainerFurnace {
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
            ContainerWindowKind::Furnace,
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
