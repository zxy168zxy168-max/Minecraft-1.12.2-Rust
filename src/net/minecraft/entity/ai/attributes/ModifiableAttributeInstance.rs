use crate::net::minecraft::entity::ai::attributes::AttributeModifier::AttributeModifier;

/// Client-side value owner for MCP `ModifiableAttributeInstance`.
///
/// Modifier evaluation preserves the three Java operations and their ordering:
/// operation 0 adds to the base, operation 1 adds a fraction of that adjusted
/// base, and operation 2 multiplies the accumulated value.
#[derive(Debug, Clone, PartialEq)]
pub struct ModifiableAttributeInstance {
    baseValue: f64,
    modifiers: Vec<AttributeModifier>,
}

impl ModifiableAttributeInstance {
    pub const fn new(baseValue: f64) -> Self {
        Self {
            baseValue,
            modifiers: Vec::new(),
        }
    }

    pub const fn getBaseValue(&self) -> f64 {
        self.baseValue
    }
    pub fn setBaseValue(&mut self, value: f64) {
        self.baseValue = value;
    }
    pub fn removeAllModifiers(&mut self) {
        self.modifiers.clear();
    }
    pub fn applyModifier(&mut self, modifier: AttributeModifier) {
        self.modifiers
            .retain(|existing| existing.getID() != modifier.getID());
        self.modifiers.push(modifier);
    }
    pub fn removeModifier(&mut self, id: uuid::Uuid) {
        self.modifiers.retain(|existing| existing.getID() != id);
    }
    pub fn getModifier(&self, id: uuid::Uuid) -> Option<&AttributeModifier> {
        self.modifiers
            .iter()
            .find(|modifier| modifier.getID() == id)
    }
    pub fn getModifiers(&self) -> &[AttributeModifier] {
        &self.modifiers
    }

    pub fn getAttributeValue(&self) -> f64 {
        let mut adjustedBase = self.baseValue;
        for modifier in self
            .modifiers
            .iter()
            .filter(|modifier| modifier.getOperation() == 0)
        {
            adjustedBase += modifier.getAmount();
        }

        let mut value = adjustedBase;
        for modifier in self
            .modifiers
            .iter()
            .filter(|modifier| modifier.getOperation() == 1)
        {
            value += adjustedBase * modifier.getAmount();
        }
        for modifier in self
            .modifiers
            .iter()
            .filter(|modifier| modifier.getOperation() == 2)
        {
            value *= 1.0 + modifier.getAmount();
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn modifier_operations_follow_mcp_order() {
        let mut instance = ModifiableAttributeInstance::new(10.0);
        instance.applyModifier(AttributeModifier::new(Uuid::from_u128(1), 2.0, 0));
        instance.applyModifier(AttributeModifier::new(Uuid::from_u128(2), 0.5, 1));
        instance.applyModifier(AttributeModifier::new(Uuid::from_u128(3), 0.25, 2));
        assert_eq!(instance.getAttributeValue(), 22.5);
    }
}
