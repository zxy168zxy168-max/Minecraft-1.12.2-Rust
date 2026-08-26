use crate::net::minecraft::item::crafting::CraftingManager::CraftingManager;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_bool, read_var_i32, CodecError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Init,
    Add,
    Remove,
}
impl State {
    fn fromId(id: i32) -> Result<Self, CodecError> {
        match id {
            0 => Ok(Self::Init),
            1 => Ok(Self::Add),
            2 => Ok(Self::Remove),
            _ => Err(CodecError::InvalidData(format!(
                "invalid recipe book state {id}"
            ))),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketRecipeBook {
    state: State,
    recipes: Vec<i32>,
    displayedRecipes: Vec<i32>,
    guiOpen: bool,
    filteringCraftable: bool,
}
impl SPacketRecipeBook {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let state = State::fromId(read_var_i32(&mut input)?)?;
        let guiOpen = read_bool(&mut input)?;
        let filteringCraftable = read_bool(&mut input)?;
        let recipes = readRecipeIds(&mut input)?;
        let displayedRecipes = if state == State::Init {
            readRecipeIds(&mut input)?
        } else {
            Vec::new()
        };
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread recipe-book bytes",
                input.len()
            )));
        }
        Ok(Self {
            state,
            recipes,
            displayedRecipes,
            guiOpen,
            filteringCraftable,
        })
    }
    pub const fn getState(&self) -> State {
        self.state
    }
    pub fn getRecipes(&self) -> &[i32] {
        &self.recipes
    }
    pub fn getDisplayedRecipes(&self) -> &[i32] {
        &self.displayedRecipes
    }
    pub const fn isGuiOpen(&self) -> bool {
        self.guiOpen
    }
    pub const fn isFilteringCraftable(&self) -> bool {
        self.filteringCraftable
    }
}
fn readRecipeIds(input: &mut &[u8]) -> Result<Vec<i32>, CodecError> {
    let count = read_var_i32(input)?;
    if count < 0 {
        return Err(CodecError::NegativeLength(count));
    }
    let count = usize::try_from(count)
        .map_err(|_| CodecError::InvalidData("recipe count overflow".to_owned()))?;
    if count > 443 {
        return Err(CodecError::InvalidData(format!(
            "recipe count {count} exceeds vanilla registry"
        )));
    }
    let mut recipes = Vec::with_capacity(count);
    for _ in 0..count {
        let id = read_var_i32(input)?;
        if CraftingManager::getRecipe(id).is_none() {
            return Err(CodecError::InvalidData(format!("unknown recipe id {id}")));
        }
        recipes.push(id);
    }
    Ok(recipes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{write_bool, write_var_i32};
    #[test]
    fn init_reads_both_lists() {
        let mut payload = Vec::new();
        write_var_i32(0, &mut payload);
        write_bool(true, &mut payload);
        write_bool(false, &mut payload);
        write_var_i32(2, &mut payload);
        write_var_i32(11, &mut payload);
        write_var_i32(12, &mut payload);
        write_var_i32(1, &mut payload);
        write_var_i32(12, &mut payload);
        let packet = SPacketRecipeBook::readPacketData(&RawPacket::new(0x31, payload)).unwrap();
        assert_eq!(packet.getRecipes(), &[11, 12]);
        assert_eq!(packet.getDisplayedRecipes(), &[12]);
    }
}
