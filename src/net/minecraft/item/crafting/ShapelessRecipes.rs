use crate::net::minecraft::item::crafting::IRecipe::IRecipe;
use crate::net::minecraft::item::crafting::RecipeRegistryData::RecipeKind;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShapelessRecipes(pub IRecipe);
impl ShapelessRecipes {
    pub fn new(recipe: IRecipe) -> Option<Self> {
        (recipe.getKind() == RecipeKind::Shapeless).then_some(Self(recipe))
    }
}
