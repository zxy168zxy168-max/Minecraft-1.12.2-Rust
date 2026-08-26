use crate::net::minecraft::item::crafting::CraftingManager::CraftingManager;
use crate::net::minecraft::item::crafting::IRecipe::IRecipe;
use std::collections::BTreeSet;

/// MCP 1.12.2 `RecipeBook`: two recipe-ID bit sets plus the open/filter flags.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecipeBook {
    knownRecipes: BTreeSet<i32>,
    recipesToBeDisplayed: BTreeSet<i32>,
    guiOpen: bool,
    filteringCraftable: bool,
}
impl RecipeBook {
    pub fn copyFrom(&mut self, other: &Self) {
        self.knownRecipes.clone_from(&other.knownRecipes);
        self.recipesToBeDisplayed
            .clone_from(&other.recipesToBeDisplayed);
    }
    pub fn unlock(&mut self, recipe: IRecipe) {
        if !recipe.isDynamic() {
            self.knownRecipes.insert(recipe.getId());
        }
    }
    pub fn unlockById(&mut self, recipeId: i32) -> bool {
        let Some(recipe) = CraftingManager::getRecipe(recipeId) else {
            return false;
        };
        self.unlock(recipe);
        true
    }
    pub fn isUnlocked(&self, recipe: IRecipe) -> bool {
        self.knownRecipes.contains(&recipe.getId())
    }
    pub fn isUnlockedById(&self, recipeId: i32) -> bool {
        self.knownRecipes.contains(&recipeId)
    }
    pub fn lock(&mut self, recipe: IRecipe) {
        self.knownRecipes.remove(&recipe.getId());
        self.recipesToBeDisplayed.remove(&recipe.getId());
    }
    pub fn lockById(&mut self, recipeId: i32) -> bool {
        let Some(recipe) = CraftingManager::getRecipe(recipeId) else {
            return false;
        };
        self.lock(recipe);
        true
    }
    pub fn isNew(&self, recipe: IRecipe) -> bool {
        self.recipesToBeDisplayed.contains(&recipe.getId())
    }
    pub fn markSeen(&mut self, recipe: IRecipe) {
        self.recipesToBeDisplayed.remove(&recipe.getId());
    }
    pub fn markNew(&mut self, recipe: IRecipe) {
        self.recipesToBeDisplayed.insert(recipe.getId());
    }
    pub fn markNewById(&mut self, recipeId: i32) -> bool {
        let Some(recipe) = CraftingManager::getRecipe(recipeId) else {
            return false;
        };
        self.markNew(recipe);
        true
    }
    pub fn clearRecipes(&mut self) {
        self.knownRecipes.clear();
        self.recipesToBeDisplayed.clear();
    }
    pub const fn isGuiOpen(&self) -> bool {
        self.guiOpen
    }
    pub fn setGuiOpen(&mut self, value: bool) {
        self.guiOpen = value;
    }
    pub const fn isFilteringCraftable(&self) -> bool {
        self.filteringCraftable
    }
    pub fn setFilteringCraftable(&mut self, value: bool) {
        self.filteringCraftable = value;
    }
    pub fn knownRecipeIds(&self) -> impl Iterator<Item = i32> + '_ {
        self.knownRecipes.iter().copied()
    }
    pub fn newRecipeIds(&self) -> impl Iterator<Item = i32> + '_ {
        self.recipesToBeDisplayed.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dynamic_recipes_are_never_unlocked() {
        let mut book = RecipeBook::default();
        book.unlock(CraftingManager::getRecipe(0).unwrap());
        assert!(!book.isUnlockedById(0));
        let torch = CraftingManager::getRecipeByName("minecraft:torch").unwrap();
        let torchId = torch.getId();
        book.unlock(torch);
        assert!(book.isUnlockedById(torchId));
        book.markNewById(torchId);
        assert!(book.isNew(torch));
        book.lockById(torchId);
        assert!(!book.isUnlockedById(torchId));
        assert!(!book.isNew(torch));
    }
}
