use crate::net::minecraft::item::ItemStack::ItemStack;

/// Direct state port of MCP 1.12.2 `MerchantRecipe`.
#[derive(Debug, Clone, PartialEq)]
pub struct MerchantRecipe {
    itemToBuy: ItemStack,
    secondItemToBuy: ItemStack,
    itemToSell: ItemStack,
    toolUses: i32,
    maxTradeUses: i32,
    rewardsExp: bool,
}
impl MerchantRecipe {
    pub fn new(
        itemToBuy: ItemStack,
        secondItemToBuy: ItemStack,
        itemToSell: ItemStack,
        toolUses: i32,
        maxTradeUses: i32,
    ) -> Self {
        Self {
            itemToBuy,
            secondItemToBuy,
            itemToSell,
            toolUses,
            maxTradeUses,
            rewardsExp: true,
        }
    }
    pub fn newSingle(itemToBuy: ItemStack, itemToSell: ItemStack) -> Self {
        Self::new(itemToBuy, ItemStack::EMPTY, itemToSell, 0, 7)
    }
    pub const fn getItemToBuy(&self) -> &ItemStack {
        &self.itemToBuy
    }
    pub const fn getSecondItemToBuy(&self) -> &ItemStack {
        &self.secondItemToBuy
    }
    pub fn hasSecondItemToBuy(&self) -> bool {
        !self.secondItemToBuy.isEmpty()
    }
    pub const fn getItemToSell(&self) -> &ItemStack {
        &self.itemToSell
    }
    pub const fn getToolUses(&self) -> i32 {
        self.toolUses
    }
    pub const fn getMaxTradeUses(&self) -> i32 {
        self.maxTradeUses
    }
    pub fn incrementToolUses(&mut self) {
        self.toolUses += 1;
    }
    pub fn increaseMaxTradeUses(&mut self, increment: i32) {
        self.maxTradeUses += increment;
    }
    pub const fn isRecipeDisabled(&self) -> bool {
        self.toolUses >= self.maxTradeUses
    }
    pub fn compensateToolUses(&mut self) {
        self.toolUses = self.maxTradeUses;
    }
    pub const fn getRewardsExp(&self) -> bool {
        self.rewardsExp
    }
}
