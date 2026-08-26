use crate::net::minecraft::item::crafting::Ingredient::Ingredient;
use crate::net::minecraft::item::crafting::RecipeRegistryData::{RecipeDefinition, RecipeKind};
use crate::net::minecraft::item::ItemStack::ItemStack;

/// Lightweight registry-backed Rust equivalent of MCP `IRecipe` used by the
/// recipe-book and place-recipe protocol. The definition is immutable and
/// carries the exact vanilla numeric registry identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IRecipe {
    definition: &'static RecipeDefinition,
}
impl IRecipe {
    pub const fn new(definition: &'static RecipeDefinition) -> Self {
        Self { definition }
    }
    pub const fn getId(&self) -> i32 {
        self.definition.id
    }
    pub const fn getRegistryName(&self) -> &'static str {
        self.definition.registryName
    }
    pub const fn getGroup(&self) -> &'static str {
        self.definition.group
    }
    pub const fn getKind(&self) -> RecipeKind {
        self.definition.kind
    }
    pub const fn isDynamic(&self) -> bool {
        self.definition.isDynamic()
    }
    pub const fn fits(&self, width: usize, height: usize) -> bool {
        self.definition.fits(width, height)
    }
    pub fn getRecipeOutput(&self) -> ItemStack {
        self.definition.outputStack()
    }
    pub fn getIngredients(&self) -> Vec<Ingredient> {
        self.definition.ingredientList()
    }
    pub const fn definition(&self) -> &'static RecipeDefinition {
        self.definition
    }
}
