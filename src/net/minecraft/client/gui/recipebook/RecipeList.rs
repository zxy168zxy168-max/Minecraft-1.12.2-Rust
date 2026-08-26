use crate::net::minecraft::client::util::RecipeItemHelper::RecipeItemHelper;
use crate::net::minecraft::item::crafting::CraftingManager::CraftingManager;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::stats::RecipeBook::RecipeBook;
use std::collections::BTreeSet;

/// MCP `RecipeList`: recipes grouped under one 25x25 recipe-book button and
/// three independent index sets for craftable, grid-fitting and unlocked.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecipeList {
    recipes: Vec<i32>,
    craftable: BTreeSet<usize>,
    fitsGrid: BTreeSet<usize>,
    unlocked: BTreeSet<usize>,
    allOutputsEqual: bool,
}
impl RecipeList {
    pub fn new() -> Self {
        Self {
            allOutputsEqual: true,
            ..Self::default()
        }
    }
    pub fn add(&mut self, recipeId: i32) {
        let Some(recipe) = CraftingManager::getRecipe(recipeId) else {
            return;
        };
        if let Some(&firstId) = self.recipes.first() {
            let first = CraftingManager::getRecipe(firstId)
                .expect("registered recipe")
                .getRecipeOutput();
            let output = recipe.getRecipeOutput();
            self.allOutputsEqual &= ItemStack::areItemsEqual(&first, &output)
                && ItemStack::areItemStackTagsEqual(&first, &output);
        }
        self.recipes.push(recipeId);
    }
    pub fn updateUnlocked(&mut self, book: &RecipeBook) {
        self.unlocked.clear();
        for (index, &recipeId) in self.recipes.iter().enumerate() {
            if book.isUnlockedById(recipeId) {
                self.unlocked.insert(index);
            }
        }
    }
    pub fn updateCraftable(
        &mut self,
        helper: &RecipeItemHelper,
        width: usize,
        height: usize,
        book: &RecipeBook,
    ) {
        self.craftable.clear();
        self.fitsGrid.clear();
        for (index, &recipeId) in self.recipes.iter().enumerate() {
            let Some(recipe) = CraftingManager::getRecipe(recipeId) else {
                continue;
            };
            let fits = recipe.fits(width, height) && book.isUnlockedById(recipeId);
            if fits {
                self.fitsGrid.insert(index);
                if helper.canCraft(recipe, None) {
                    self.craftable.insert(index);
                }
            }
        }
    }
    pub fn hasUnlocked(&self) -> bool {
        !self.unlocked.is_empty()
    }
    pub fn hasCraftable(&self) -> bool {
        !self.craftable.is_empty()
    }
    pub fn hasFitting(&self) -> bool {
        !self.fitsGrid.is_empty()
    }
    pub fn isCraftable(&self, recipeId: i32) -> bool {
        self.recipes
            .iter()
            .position(|&id| id == recipeId)
            .is_some_and(|i| self.craftable.contains(&i))
    }
    pub fn recipes(&self) -> &[i32] {
        &self.recipes
    }
    pub fn visibleRecipes(&self, craftableOnly: bool) -> Vec<i32> {
        self.unlocked
            .iter()
            .filter(|&&i| {
                if craftableOnly {
                    self.craftable.contains(&i)
                } else {
                    self.fitsGrid.contains(&i)
                }
            })
            .filter_map(|&i| self.recipes.get(i).copied())
            .collect()
    }
    pub fn recipesByCraftability(&self, craftable: bool) -> Vec<i32> {
        self.unlocked
            .iter()
            .filter(|&&i| self.fitsGrid.contains(&i) && self.craftable.contains(&i) == craftable)
            .filter_map(|&i| self.recipes.get(i).copied())
            .collect()
    }
    pub const fn allOutputsEqual(&self) -> bool {
        self.allOutputsEqual
    }
}
