use crate::net::minecraft::client::gui::recipebook::RecipeList::RecipeList;
use crate::net::minecraft::item::crafting::CraftingManager::CraftingManager;
use crate::net::minecraft::item::crafting::RecipeRegistryData::RecipeCategory;
use std::collections::HashMap;

/// Exact grouping constructed by MCP `RecipeBookClient` after
/// `CraftingManager` registration. Empty groups get one button per recipe;
/// named groups share a button only within the same creative category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeBookClient {
    byCategory: HashMap<RecipeCategory, Vec<usize>>,
    lists: Vec<RecipeList>,
}
impl Default for RecipeBookClient {
    fn default() -> Self {
        Self::new()
    }
}
impl RecipeBookClient {
    pub fn new() -> Self {
        let mut client = Self {
            byCategory: HashMap::new(),
            lists: Vec::new(),
        };
        let mut grouped = HashMap::<(RecipeCategory, &'static str), usize>::new();
        for recipe in CraftingManager::recipes() {
            if recipe.isDynamic() {
                continue;
            }
            let definition = recipe.definition();
            let listIndex = if definition.group.is_empty() {
                client.createList(definition.category)
            } else if let Some(&index) = grouped.get(&(definition.category, definition.group)) {
                index
            } else {
                let index = client.createList(definition.category);
                grouped.insert((definition.category, definition.group), index);
                index
            };
            client.lists[listIndex].add(recipe.getId());
        }
        client
    }
    fn createList(&mut self, category: RecipeCategory) -> usize {
        let index = self.lists.len();
        self.lists.push(RecipeList::new());
        self.byCategory.entry(category).or_default().push(index);
        self.byCategory
            .entry(RecipeCategory::Search)
            .or_default()
            .push(index);
        index
    }
    pub fn listIndices(&self, category: RecipeCategory) -> &[usize] {
        self.byCategory
            .get(&category)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
    pub fn list(&self, index: usize) -> Option<&RecipeList> {
        self.lists.get(index)
    }
    pub fn listMut(&mut self, index: usize) -> Option<&mut RecipeList> {
        self.lists.get_mut(index)
    }
    pub fn lists(&self) -> &[RecipeList] {
        &self.lists
    }
    pub fn listsMut(&mut self) -> &mut [RecipeList] {
        &mut self.lists
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_non_dynamic_recipes_are_grouped() {
        let client = RecipeBookClient::new();
        let total: usize = client.lists().iter().map(|l| l.recipes().len()).sum();
        assert_eq!(total, 432);
        assert_eq!(
            client.listIndices(RecipeCategory::Search).len(),
            client.lists().len()
        );
    }
}
