use crate::net::minecraft::item::ItemStack::ItemStack;

/// MCP `Ingredient`: an empty alternative list matches an empty slot; ordinary
/// alternatives compare item ID and metadata, with 32767 as the wildcard.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ingredient {
    matchingStacks: Vec<ItemStack>,
}
impl Ingredient {
    pub fn empty() -> Self {
        Self::default()
    }
    pub fn fromStacks(stacks: Vec<ItemStack>) -> Self {
        Self {
            matchingStacks: stacks,
        }
    }
    pub fn fromEncodedAlternatives(encoded: &str) -> Self {
        if encoded.is_empty() {
            return Self::empty();
        }
        let stacks = encoded
            .split('|')
            .filter_map(|value| {
                let mut parts = value.split(':');
                let itemId = parts.next()?.parse::<i16>().ok()?;
                let itemDamage = parts.next()?.parse::<i16>().ok()?;
                let count = parts.next()?.parse::<u8>().ok()?;
                if parts.next().is_some() {
                    return None;
                }
                Some(ItemStack {
                    itemId,
                    count,
                    itemDamage,
                    tagCompound: None,
                })
            })
            .collect();
        Self::fromStacks(stacks)
    }
    pub fn apply(&self, stack: &ItemStack) -> bool {
        if self.matchingStacks.is_empty() {
            return stack.isEmpty();
        }
        if stack.isEmpty() {
            return false;
        }
        self.matchingStacks.iter().any(|candidate| {
            candidate.itemId == stack.itemId
                && (candidate.itemDamage == 32767 || candidate.itemDamage == stack.itemDamage)
        })
    }
    pub fn getMatchingStacks(&self) -> &[ItemStack] {
        &self.matchingStacks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wildcard_metadata_and_empty_slot_follow_mcp() {
        let coal = Ingredient::fromEncodedAlternatives("263:0:1|263:1:1");
        assert!(coal.apply(&ItemStack {
            itemId: 263,
            count: 2,
            itemDamage: 1,
            tagCompound: None
        }));
        let wildcard = Ingredient::fromEncodedAlternatives("5:32767:1");
        assert!(wildcard.apply(&ItemStack {
            itemId: 5,
            count: 1,
            itemDamage: 4,
            tagCompound: None
        }));
        assert!(Ingredient::empty().apply(&ItemStack::EMPTY));
    }
}
