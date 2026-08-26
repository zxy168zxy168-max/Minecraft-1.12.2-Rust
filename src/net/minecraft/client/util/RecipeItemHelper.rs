use crate::net::minecraft::item::crafting::IRecipe::IRecipe;
use crate::net::minecraft::item::crafting::Ingredient::Ingredient;
use crate::net::minecraft::item::Item::Item;
use crate::net::minecraft::item::ItemStack::ItemStack;
use std::collections::HashMap;

/// Behavior-equivalent Rust port of MCP `RecipeItemHelper`. Vanilla uses a
/// BitSet augmenting-path picker; this implementation solves the same bounded
/// bipartite assignment directly, preserving item packing and eligibility.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecipeItemHelper {
    counts: HashMap<i32, i32>,
}
impl RecipeItemHelper {
    pub fn accountStack(&mut self, stack: &ItemStack) {
        if stack.isEmpty()
            || stack.isItemDamaged()
            || stack.isItemEnchanted()
            || stack.hasDisplayName()
        {
            return;
        }
        *self.counts.entry(Self::pack(stack)).or_insert(0) += stack.getCount();
    }
    pub fn clear(&mut self) {
        self.counts.clear();
    }
    pub fn has(&self, packed: i32) -> bool {
        self.counts.get(&packed).copied().unwrap_or(0) > 0
    }
    pub fn pack(stack: &ItemStack) -> i32 {
        let metadata = if Item::getHasSubtypes(stack.itemId) {
            stack.itemDamage as i32
        } else {
            0
        };
        ((stack.itemId as i32 & 0xFFFF) << 16) | (metadata & 0xFFFF)
    }
    pub fn unpack(packed: i32) -> ItemStack {
        if packed == 0 {
            return ItemStack::EMPTY;
        }
        ItemStack {
            itemId: ((packed >> 16) & 0xFFFF) as i16,
            count: 1,
            itemDamage: (packed & 0xFFFF) as i16,
            tagCompound: None,
        }
    }
    pub fn canCraft(&self, recipe: IRecipe, assignment: Option<&mut Vec<i32>>) -> bool {
        self.canCraftAmount(recipe, 1, assignment)
    }
    pub fn canCraftAmount(
        &self,
        recipe: IRecipe,
        amount: i32,
        assignment: Option<&mut Vec<i32>>,
    ) -> bool {
        if amount <= 0 {
            if let Some(out) = assignment {
                out.clear();
            }
            return true;
        }
        let ingredients = recipe.getIngredients();
        let nonempty: Vec<(usize, &Ingredient)> = ingredients
            .iter()
            .enumerate()
            .filter(|(_, ingredient)| !ingredient.getMatchingStacks().is_empty())
            .collect();
        let mut candidates: Vec<(usize, Vec<i32>)> = nonempty
            .iter()
            .map(|(slot, ingredient)| {
                let mut packed = ingredient
                    .getMatchingStacks()
                    .iter()
                    .map(Self::pack)
                    .filter(|key| self.counts.get(key).copied().unwrap_or(0) >= amount)
                    .collect::<Vec<_>>();
                packed.sort_unstable();
                packed.dedup();
                (*slot, packed)
            })
            .collect();
        if candidates.iter().any(|(_, values)| values.is_empty()) {
            return false;
        }
        candidates.sort_by_key(|(_, values)| values.len());
        let mut remaining = self.counts.clone();
        let mut chosen = HashMap::<usize, i32>::new();
        if !assignIngredients(0, &candidates, amount, &mut remaining, &mut chosen) {
            return false;
        }
        if let Some(out) = assignment {
            out.clear();
            out.extend((0..ingredients.len()).map(|slot| chosen.get(&slot).copied().unwrap_or(0)));
        }
        true
    }
    pub fn maximumCraftable(
        &self,
        recipe: IRecipe,
        limit: i32,
        assignment: Option<&mut Vec<i32>>,
    ) -> i32 {
        let ingredients = recipe.getIngredients();
        let nonempty = ingredients
            .iter()
            .filter(|ingredient| !ingredient.getMatchingStacks().is_empty())
            .count();
        if nonempty == 0 {
            return 0;
        }
        let upper = limit
            .max(0)
            .min(self.counts.values().copied().sum::<i32>() / nonempty as i32);
        let mut low = 0;
        let mut high = upper + 1;
        while high - low > 1 {
            let middle = low + (high - low) / 2;
            if self.canCraftAmount(recipe, middle, None) {
                low = middle;
            } else {
                high = middle;
            }
        }
        if let Some(out) = assignment {
            let _ = self.canCraftAmount(recipe, low, Some(out));
        }
        low
    }
}
fn assignIngredients(
    index: usize,
    candidates: &[(usize, Vec<i32>)],
    amount: i32,
    remaining: &mut HashMap<i32, i32>,
    chosen: &mut HashMap<usize, i32>,
) -> bool {
    if index == candidates.len() {
        return true;
    }
    let (slot, keys) = &candidates[index];
    for &key in keys {
        let available = remaining.get(&key).copied().unwrap_or(0);
        if available < amount {
            continue;
        }
        remaining.insert(key, available - amount);
        chosen.insert(*slot, key);
        if assignIngredients(index + 1, candidates, amount, remaining, chosen) {
            return true;
        }
        chosen.remove(slot);
        remaining.insert(key, available);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::item::crafting::CraftingManager::CraftingManager;
    #[test]
    fn torch_recipe_requires_coal_and_stick() {
        let recipe = CraftingManager::getRecipeByName("minecraft:torch").unwrap();
        let mut helper = RecipeItemHelper::default();
        helper.accountStack(&ItemStack {
            itemId: 263,
            count: 2,
            itemDamage: 0,
            tagCompound: None,
        });
        assert!(!helper.canCraft(recipe, None));
        helper.accountStack(&ItemStack {
            itemId: 280,
            count: 2,
            itemDamage: 0,
            tagCompound: None,
        });
        assert!(helper.canCraft(recipe, None));
        assert_eq!(helper.maximumCraftable(recipe, 64, None), 2);
    }
}
