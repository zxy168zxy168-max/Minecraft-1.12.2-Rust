use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::nbt::NBTUtil::areNBTEquals;
use crate::net::minecraft::network::PacketBuffer::{
    read_bool, read_i32_be, read_u8, write_bool, write_i32_be, CodecError,
};
use crate::net::minecraft::village::MerchantRecipe::MerchantRecipe;

/// MCP 1.12.2 `MerchantRecipeList`, including the legacy selected-index rule
/// where index zero deliberately falls back to a complete scan.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MerchantRecipeList(pub Vec<MerchantRecipe>);
impl MerchantRecipeList {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn isEmpty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn get(&self, index: usize) -> Option<&MerchantRecipe> {
        self.0.get(index)
    }
    pub fn iter(&self) -> impl Iterator<Item = &MerchantRecipe> {
        self.0.iter()
    }
    pub fn push(&mut self, recipe: MerchantRecipe) {
        self.0.push(recipe);
    }

    fn areItemStacksExactlyEqual(stack1: &ItemStack, stack2: &ItemStack) -> bool {
        ItemStack::areItemsEqual(stack1, stack2)
            && match &stack2.tagCompound {
                None => true,
                Some(expected) => stack1
                    .tagCompound
                    .as_ref()
                    .is_some_and(|candidate| areNBTEquals(expected, candidate, false)),
            }
    }
    pub fn canRecipeBeUsed(
        &self,
        first: &ItemStack,
        second: &ItemStack,
        selected: i32,
    ) -> Option<&MerchantRecipe> {
        if selected > 0 && (selected as usize) < self.0.len() {
            let recipe = &self.0[selected as usize];
            let matches = Self::areItemStacksExactlyEqual(first, recipe.getItemToBuy())
                && first.getCount() >= recipe.getItemToBuy().getCount()
                && ((!recipe.hasSecondItemToBuy() && second.isEmpty())
                    || (recipe.hasSecondItemToBuy()
                        && Self::areItemStacksExactlyEqual(second, recipe.getSecondItemToBuy())
                        && second.getCount() >= recipe.getSecondItemToBuy().getCount()));
            return matches.then_some(recipe);
        }
        self.0.iter().find(|recipe| {
            Self::areItemStacksExactlyEqual(first, recipe.getItemToBuy())
                && first.getCount() >= recipe.getItemToBuy().getCount()
                && ((!recipe.hasSecondItemToBuy() && second.isEmpty())
                    || (recipe.hasSecondItemToBuy()
                        && Self::areItemStacksExactlyEqual(second, recipe.getSecondItemToBuy())
                        && second.getCount() >= recipe.getSecondItemToBuy().getCount()))
        })
    }
    pub fn writeToBuf(&self, output: &mut Vec<u8>) -> Result<(), CodecError> {
        if self.0.len() > u8::MAX as usize {
            return Err(CodecError::InvalidData(
                "more than 255 merchant recipes".to_owned(),
            ));
        }
        output.push(self.0.len() as u8);
        for recipe in &self.0 {
            recipe.getItemToBuy().writeToBuffer(output)?;
            recipe.getItemToSell().writeToBuffer(output)?;
            write_bool(recipe.hasSecondItemToBuy(), output);
            if recipe.hasSecondItemToBuy() {
                recipe.getSecondItemToBuy().writeToBuffer(output)?;
            }
            write_bool(recipe.isRecipeDisabled(), output);
            write_i32_be(recipe.getToolUses(), output);
            write_i32_be(recipe.getMaxTradeUses(), output);
        }
        Ok(())
    }
    pub fn readFromBuf(input: &mut &[u8]) -> Result<Self, CodecError> {
        let count = read_u8(input)? as usize;
        let mut result = Self::new();
        for _ in 0..count {
            let buy = ItemStack::readFromBuffer(input)?;
            let sell = ItemStack::readFromBuffer(input)?;
            let second = if read_bool(input)? {
                ItemStack::readFromBuffer(input)?
            } else {
                ItemStack::EMPTY
            };
            let disabled = read_bool(input)?;
            let uses = read_i32_be(input)?;
            let maxUses = read_i32_be(input)?;
            let mut recipe = MerchantRecipe::new(buy, second, sell, uses, maxUses);
            if disabled {
                recipe.compensateToolUses();
            }
            result.push(recipe);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn stack(id: i16, count: u8) -> ItemStack {
        ItemStack {
            itemId: id,
            count,
            itemDamage: 0,
            tagCompound: None,
        }
    }
    #[test]
    fn packet_round_trip_and_selected_zero_scan() {
        let mut list = MerchantRecipeList::new();
        list.push(MerchantRecipe::new(
            stack(388, 3),
            ItemStack::EMPTY,
            stack(297, 1),
            2,
            7,
        ));
        let mut bytes = Vec::new();
        list.writeToBuf(&mut bytes).unwrap();
        let mut input = bytes.as_slice();
        let decoded = MerchantRecipeList::readFromBuf(&mut input).unwrap();
        assert!(input.is_empty());
        assert!(decoded
            .canRecipeBeUsed(&stack(388, 3), &ItemStack::EMPTY, 0)
            .is_some());
    }
}
