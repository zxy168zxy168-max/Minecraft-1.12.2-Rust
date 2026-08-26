use std::collections::BTreeMap;

use crate::net::minecraft::entity::ai::attributes::AttributeModifier::AttributeModifier;
use crate::net::minecraft::entity::ai::attributes::ModifiableAttributeInstance::ModifiableAttributeInstance;

/// Name-indexed client equivalent of MCP `AbstractAttributeMap`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AbstractAttributeMap {
    attributes: BTreeMap<String, ModifiableAttributeInstance>,
}

impl AbstractAttributeMap {
    pub fn registerAttribute(
        &mut self,
        name: impl Into<String>,
        defaultValue: f64,
    ) -> &mut ModifiableAttributeInstance {
        self.attributes
            .entry(name.into())
            .or_insert_with(|| ModifiableAttributeInstance::new(defaultValue))
    }

    pub fn getAttributeInstanceByName(&self, name: &str) -> Option<&ModifiableAttributeInstance> {
        self.attributes.get(name)
    }

    pub fn getAttributeInstanceByNameMut(
        &mut self,
        name: &str,
    ) -> Option<&mut ModifiableAttributeInstance> {
        self.attributes.get_mut(name)
    }

    pub fn setSnapshot(&mut self, name: &str, baseValue: f64, modifiers: &[AttributeModifier]) {
        let instance = self.registerAttribute(name.to_owned(), 0.0);
        instance.setBaseValue(baseValue);
        instance.removeAllModifiers();
        for modifier in modifiers {
            instance.applyModifier(*modifier);
        }
    }

    pub fn getAttributeValue(&self, name: &str, fallback: f64) -> f64 {
        self.attributes
            .get(name)
            .map(ModifiableAttributeInstance::getAttributeValue)
            .unwrap_or(fallback)
    }
}
