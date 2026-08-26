use crate::net::minecraft::item::crafting::IRecipe::IRecipe;
use crate::net::minecraft::item::crafting::RecipeRegistryData::RecipeKind;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShapedRecipes(pub IRecipe);
impl ShapedRecipes {
    pub fn new(recipe: IRecipe) -> Option<Self> {
        (recipe.getKind() == RecipeKind::Shaped).then_some(Self(recipe))
    }
    pub const fn getRecipeWidth(&self) -> u8 {
        self.0.definition().width
    }
    pub const fn getRecipeHeight(&self) -> u8 {
        self.0.definition().height
    }
}
