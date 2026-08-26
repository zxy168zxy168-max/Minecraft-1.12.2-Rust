use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::inventory::ContainerWindow::{ContainerWindow, ContainerWindowKind};
use crate::net::minecraft::network::PacketBuffer::CodecError;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

/// MCP 1.12.2 `ContainerRepair` client-side container owner.
/// Repair-cost calculation and output generation remain server-authoritative;
/// property 0 mirrors `maximumCost` for GUI rendering and take validation.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerRepair {
    state: ContainerWindow,
}

impl ContainerRepair {
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
            ContainerWindowKind::Repair,
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
