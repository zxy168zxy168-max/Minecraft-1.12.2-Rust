use crate::net::minecraft::item::crafting::CraftingManager::CraftingManager;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{write_bool, write_var_i32, CodecError};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPacketPlaceRecipe {
    windowId: i8,
    recipeId: i32,
    placeAll: bool,
}
impl CPacketPlaceRecipe {
    pub fn new(windowId: i32, recipeId: i32, placeAll: bool) -> Result<Self, CodecError> {
        if !(-128..=127).contains(&windowId) {
            return Err(CodecError::InvalidData(format!(
                "window id {windowId} does not fit signed byte"
            )));
        }
        if CraftingManager::getRecipe(recipeId).is_none() {
            return Err(CodecError::InvalidData(format!(
                "unknown recipe id {recipeId}"
            )));
        }
        Ok(Self {
            windowId: windowId as i8,
            recipeId,
            placeAll,
        })
    }
    pub fn writePacketData(&self) -> RawPacket {
        let mut payload = vec![self.windowId as u8];
        write_var_i32(self.recipeId, &mut payload);
        write_bool(self.placeAll, &mut payload);
        RawPacket::new(0x12, payload)
    }
    pub const fn getWindowId(&self) -> i8 {
        self.windowId
    }
    pub const fn getRecipeId(&self) -> i32 {
        self.recipeId
    }
    pub const fn isPlaceAll(&self) -> bool {
        self.placeAll
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wooden_pickaxe_uses_official_1122_registry_id() {
        let recipe = CraftingManager::getRecipeByName("minecraft:wooden_pickaxe").unwrap();
        assert_eq!(recipe.getId(), 26);
        let packet = CPacketPlaceRecipe::new(1, recipe.getId(), false)
            .unwrap()
            .writePacketData();
        assert_eq!(packet.id, 0x12);
        assert_eq!(packet.payload, vec![1, 26, 0]);
    }

    #[test]
    fn id_427_is_birch_door_not_wooden_pickaxe() {
        assert_eq!(
            CraftingManager::getRecipe(427).unwrap().getRegistryName(),
            "minecraft:birch_door"
        );
    }
}
