use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::village::MerchantRecipe::MerchantRecipe;
use crate::net::minecraft::village::MerchantRecipeList::MerchantRecipeList;

/// Client-side state owner matching MCP 1.12.2 `InventoryMerchant`. The result
/// is only a preview; server `SPacketSetSlot`/`SPacketWindowItems` remain
/// authoritative for trade execution and input consumption.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InventoryMerchant {
    currentRecipeIndex: i32,
    currentRecipe: Option<MerchantRecipe>,
    recipes: Option<MerchantRecipeList>,
}
impl InventoryMerchant {
    pub fn getCurrentRecipe(&self) -> Option<&MerchantRecipe> {
        self.currentRecipe.as_ref()
    }
    pub const fn getCurrentRecipeIndex(&self) -> i32 {
        self.currentRecipeIndex
    }
    pub fn setCurrentRecipeIndex(&mut self, index: i32) {
        self.currentRecipeIndex = index.max(0);
    }
    pub fn getRecipes(&self) -> Option<&MerchantRecipeList> {
        self.recipes.as_ref()
    }
    pub fn setRecipes(&mut self, recipes: MerchantRecipeList) {
        self.recipes = Some(recipes);
    }
    pub fn resetRecipeAndSlots(&mut self, first: &ItemStack, second: &ItemStack) -> ItemStack {
        self.currentRecipe = None;
        let (primary, secondary) = if first.isEmpty() && !second.isEmpty() {
            (second, first)
        } else {
            (first, second)
        };
        let found = self
            .recipes
            .as_ref()
            .and_then(|list| {
                list.canRecipeBeUsed(primary, secondary, self.currentRecipeIndex)
                    .or_else(|| list.canRecipeBeUsed(secondary, primary, self.currentRecipeIndex))
            })
            .filter(|recipe| !recipe.isRecipeDisabled())
            .cloned();
        self.currentRecipe = found.clone();
        found
            .map(|r| r.getItemToSell().clone())
            .unwrap_or(ItemStack::EMPTY)
    }
}
