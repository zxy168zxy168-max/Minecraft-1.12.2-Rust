use crate::net::minecraft::item::crafting::CraftingManager::CraftingManager;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    write_bool, write_i32_be, write_var_i32, CodecError,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    Shown,
    Settings,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPacketRecipeInfo {
    purpose: Purpose,
    recipeId: Option<i32>,
    guiOpen: bool,
    filteringCraftable: bool,
}
impl CPacketRecipeInfo {
    pub fn shown(recipeId: i32) -> Result<Self, CodecError> {
        if CraftingManager::getRecipe(recipeId).is_none() {
            return Err(CodecError::InvalidData(format!(
                "unknown recipe id {recipeId}"
            )));
        }
        Ok(Self {
            purpose: Purpose::Shown,
            recipeId: Some(recipeId),
            guiOpen: false,
            filteringCraftable: false,
        })
    }
    pub const fn settings(guiOpen: bool, filteringCraftable: bool) -> Self {
        Self {
            purpose: Purpose::Settings,
            recipeId: None,
            guiOpen,
            filteringCraftable,
        }
    }
    pub fn writePacketData(&self) -> RawPacket {
        let mut payload = Vec::new();
        match self.purpose {
            Purpose::Shown => {
                write_var_i32(0, &mut payload);
                write_i32_be(
                    self.recipeId.expect("SHOWN always has recipe id"),
                    &mut payload,
                );
            }
            Purpose::Settings => {
                write_var_i32(1, &mut payload);
                write_bool(self.guiOpen, &mut payload);
                write_bool(self.filteringCraftable, &mut payload);
            }
        }
        RawPacket::new(0x17, payload)
    }
    pub const fn getPurpose(&self) -> Purpose {
        self.purpose
    }
    pub const fn getRecipeId(&self) -> Option<i32> {
        self.recipeId
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shown_uses_fixed_big_endian_i32() {
        let p = CPacketRecipeInfo::shown(300).unwrap().writePacketData();
        assert_eq!(p.payload, vec![0, 0, 0, 1, 44]);
    }
}
