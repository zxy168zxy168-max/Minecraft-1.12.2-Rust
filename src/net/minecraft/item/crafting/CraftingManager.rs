use crate::net::minecraft::item::crafting::IRecipe::IRecipe;
use crate::net::minecraft::item::crafting::RecipeRegistryData::{
    getRecipeById, getRecipeByName, RECIPES,
};

pub struct CraftingManager;
impl CraftingManager {
    pub fn getRecipe(id: i32) -> Option<IRecipe> {
        getRecipeById(id).map(IRecipe::new)
    }
    pub fn getRecipeByName(name: &str) -> Option<IRecipe> {
        getRecipeByName(name).map(IRecipe::new)
    }
    pub fn getIdForRecipe(recipe: IRecipe) -> i32 {
        recipe.getId()
    }
    pub fn recipes() -> impl ExactSizeIterator<Item = IRecipe> {
        RECIPES.iter().map(IRecipe::new)
    }
}
