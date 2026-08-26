use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::inventory::ContainerWindow::{ContainerWindow, ContainerWindowKind};
use crate::net::minecraft::network::PacketBuffer::CodecError;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

/// MCP 1.12.2 `ContainerWorkbench` client-side container owner.
///
/// Crafting-result calculation remains server-authoritative. The shared Rust
/// state stores the exact 10 local slots plus 36 player slots and applies the
/// source output/input/shift-click rules for this concrete class.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerWorkbench {
    state: ContainerWindow,
}

impl ContainerWorkbench {
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
            ContainerWindowKind::Workbench,
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
