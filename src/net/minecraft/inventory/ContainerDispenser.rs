use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::inventory::ContainerWindow::{ContainerWindow, ContainerWindowKind};
use crate::net::minecraft::network::PacketBuffer::CodecError;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

/// MCP 1.12.2 `ContainerDispenser` client-side container owner.
///
/// The concrete class identity is retained while `ContainerWindow` provides
/// shared protocol slot, property, and desktop click mechanics.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerDispenser {
    state: ContainerWindow,
}

impl ContainerDispenser {
    pub fn new(
        windowId: i32,
        title: ITextComponent,
        reportedSlotCount: usize,
        playerInventory: &InventoryPlayer,
    ) -> Result<Self, CodecError> {
        Self::newForKind(
            windowId,
            title,
            reportedSlotCount,
            playerInventory,
            ContainerWindowKind::Dispenser,
        )
    }

    /// Minecraft 1.12.2 uses the same `ContainerDispenser` class for a
    /// dropper window. Only the GUI id/title differ on the client.
    pub fn newDropper(
        windowId: i32,
        title: ITextComponent,
        reportedSlotCount: usize,
        playerInventory: &InventoryPlayer,
    ) -> Result<Self, CodecError> {
        Self::newForKind(
            windowId,
            title,
            reportedSlotCount,
            playerInventory,
            ContainerWindowKind::Dropper,
        )
    }

    fn newForKind(
        windowId: i32,
        title: ITextComponent,
        reportedSlotCount: usize,
        playerInventory: &InventoryPlayer,
        kind: ContainerWindowKind,
    ) -> Result<Self, CodecError> {
        ContainerWindow::new(windowId, title, reportedSlotCount, playerInventory, kind)
            .map(|state| Self { state })
    }

    pub const fn state(&self) -> &ContainerWindow {
        &self.state
    }
    pub fn stateMut(&mut self) -> &mut ContainerWindow {
        &mut self.state
    }
}
