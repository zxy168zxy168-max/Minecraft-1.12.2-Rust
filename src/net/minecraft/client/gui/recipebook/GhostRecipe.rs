use crate::net::minecraft::item::crafting::Ingredient::Ingredient;
use crate::net::minecraft::item::ItemStack::ItemStack;
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GhostIngredient {
    pub ingredient: Ingredient,
    pub x: i32,
    pub y: i32,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GhostRecipe {
    recipeId: Option<i32>,
    ingredients: Vec<GhostIngredient>,
    time: f32,
}
impl GhostRecipe {
    pub fn clear(&mut self) {
        self.recipeId = None;
        self.ingredients.clear();
        self.time = 0.0;
    }
    pub fn addIngredient(&mut self, ingredient: Ingredient, x: i32, y: i32) {
        self.ingredients.push(GhostIngredient { ingredient, x, y });
    }
    pub fn setRecipe(&mut self, recipeId: i32) {
        self.recipeId = Some(recipeId);
    }
    pub const fn recipeId(&self) -> Option<i32> {
        self.recipeId
    }
    pub fn ingredients(&self) -> &[GhostIngredient] {
        &self.ingredients
    }
    pub fn tick(&mut self, partialTicks: f32, controlHeld: bool) {
        if !controlHeld {
            self.time += partialTicks;
        }
    }
    pub fn displayedStack(&self, index: usize) -> ItemStack {
        let Some(entry) = self.ingredients.get(index) else {
            return ItemStack::EMPTY;
        };
        let stacks = entry.ingredient.getMatchingStacks();
        if stacks.is_empty() {
            return ItemStack::EMPTY;
        }
        stacks[((self.time / 30.0).floor() as usize) % stacks.len()].clone()
    }
}
