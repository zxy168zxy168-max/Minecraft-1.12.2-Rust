use crate::net::minecraft::item::crafting::CraftingManager::CraftingManager;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i8, read_var_i32, CodecError};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketPlaceGhostRecipe {
    windowId: i8,
    recipeId: i32,
}
impl SPacketPlaceGhostRecipe {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let windowId = read_i8(&mut input)?;
        let recipeId = read_var_i32(&mut input)?;
        if CraftingManager::getRecipe(recipeId).is_none() {
            return Err(CodecError::InvalidData(format!(
                "unknown recipe id {recipeId}"
            )));
        }
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread ghost-recipe bytes",
                input.len()
            )));
        }
        Ok(Self { windowId, recipeId })
    }
    pub const fn getWindowId(&self) -> i8 {
        self.windowId
    }
    pub const fn getRecipeId(&self) -> i32 {
        self.recipeId
    }
}
