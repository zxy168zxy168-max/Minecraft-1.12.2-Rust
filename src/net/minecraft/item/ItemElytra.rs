use crate::net::minecraft::item::ItemStack::ItemStack;

/// MCP 1.12.2 `ItemElytra` predicates used by both the renderer and
/// `EntityPlayerSP#onLivingUpdate` fall-flying request.
pub struct ItemElytra;

impl ItemElytra {
    pub const ITEM_ID: i16 = 443;
    pub const MAX_DAMAGE: i32 = 432;

    /// Mojang's 1.12.2 method name is misleading: it returns `true` while the
    /// elytra still has at least two durability points remaining.
    pub fn isBroken(stack: &ItemStack) -> bool {
        !stack.isEmpty()
            && stack.itemId == Self::ITEM_ID
            && i32::from(stack.itemDamage) < stack.getMaxDamage() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_predicate_rejects_last_durability_point() {
        let mut stack = ItemStack {
            itemId: 443,
            count: 1,
            itemDamage: 430,
            tagCompound: None,
        };
        assert!(ItemElytra::isBroken(&stack));
        stack.itemDamage = 431;
        assert!(!ItemElytra::isBroken(&stack));
    }
}
