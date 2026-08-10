use std::collections::BTreeMap;

use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;

/// Faithful data model for MCP 1.12.2 `GameRules`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    AnyValue,
    BooleanValue,
    NumericalValue,
    Function,
}

#[derive(Debug, Clone, PartialEq)]
struct Value {
    valueString: String,
    valueBoolean: bool,
    valueInteger: i32,
    valueDouble: f64,
    valueType: ValueType,
}

impl Value {
    fn new(value: &str, valueType: ValueType) -> Self {
        let mut result = Self {
            valueString: String::new(),
            valueBoolean: false,
            valueInteger: 0,
            valueDouble: 0.0,
            valueType,
        };
        result.setValue(value);
        result
    }

    /// MCP `GameRules.Value#setValue`, including Java's early return for the
    /// literal strings `true` and `false` and retention of the previous
    /// numeric values when parsing a later non-numeric string fails.
    fn setValue(&mut self, value: &str) {
        self.valueString = value.to_owned();
        if value == "false" {
            self.valueBoolean = false;
            return;
        }
        if value == "true" {
            self.valueBoolean = true;
            return;
        }

        self.valueBoolean = value.eq_ignore_ascii_case("true");
        self.valueInteger = if self.valueBoolean { 1 } else { 0 };
        if let Ok(parsed) = value.parse::<i32>() {
            self.valueInteger = parsed;
        }
        if let Ok(parsed) = value.parse::<f64>() {
            self.valueDouble = parsed;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameRules {
    theGameRules: BTreeMap<String, Value>,
}

impl Default for GameRules {
    fn default() -> Self {
        let mut rules = Self { theGameRules: BTreeMap::new() };
        // Constructor order and defaults from MCP 1.12.2 `GameRules`.
        rules.addGameRule("doFireTick", "true", ValueType::BooleanValue);
        rules.addGameRule("mobGriefing", "true", ValueType::BooleanValue);
        rules.addGameRule("keepInventory", "false", ValueType::BooleanValue);
        rules.addGameRule("doMobSpawning", "true", ValueType::BooleanValue);
        rules.addGameRule("doMobLoot", "true", ValueType::BooleanValue);
        rules.addGameRule("doTileDrops", "true", ValueType::BooleanValue);
        rules.addGameRule("doEntityDrops", "true", ValueType::BooleanValue);
        rules.addGameRule("commandBlockOutput", "true", ValueType::BooleanValue);
        rules.addGameRule("naturalRegeneration", "true", ValueType::BooleanValue);
        rules.addGameRule("doDaylightCycle", "true", ValueType::BooleanValue);
        rules.addGameRule("logAdminCommands", "true", ValueType::BooleanValue);
        rules.addGameRule("showDeathMessages", "true", ValueType::BooleanValue);
        rules.addGameRule("randomTickSpeed", "3", ValueType::NumericalValue);
        rules.addGameRule("sendCommandFeedback", "true", ValueType::BooleanValue);
        rules.addGameRule("reducedDebugInfo", "false", ValueType::BooleanValue);
        rules.addGameRule("spectatorsGenerateChunks", "true", ValueType::BooleanValue);
        rules.addGameRule("spawnRadius", "10", ValueType::NumericalValue);
        rules.addGameRule("disableElytraMovementCheck", "false", ValueType::BooleanValue);
        rules.addGameRule("maxEntityCramming", "24", ValueType::NumericalValue);
        rules.addGameRule("doWeatherCycle", "true", ValueType::BooleanValue);
        rules.addGameRule("doLimitedCrafting", "false", ValueType::BooleanValue);
        rules.addGameRule("maxCommandChainLength", "65536", ValueType::NumericalValue);
        rules.addGameRule("announceAdvancements", "true", ValueType::BooleanValue);
        rules.addGameRule("gameLoopFunction", "-", ValueType::Function);
        rules
    }
}

impl GameRules {
    pub fn new() -> Self { Self::default() }

    pub fn addGameRule(&mut self, key: &str, value: &str, valueType: ValueType) {
        self.theGameRules.insert(key.to_owned(), Value::new(value, valueType));
    }

    pub fn setOrCreateGameRule(&mut self, key: &str, ruleValue: &str) {
        if let Some(value) = self.theGameRules.get_mut(key) {
            value.setValue(ruleValue);
        } else {
            self.addGameRule(key, ruleValue, ValueType::AnyValue);
        }
    }

    pub fn getString(&self, name: &str) -> &str {
        self.theGameRules.get(name).map(|value| value.valueString.as_str()).unwrap_or("")
    }

    pub fn getBoolean(&self, name: &str) -> bool {
        self.theGameRules.get(name).is_some_and(|value| value.valueBoolean)
    }

    pub fn getInt(&self, name: &str) -> i32 {
        self.theGameRules.get(name).map_or(0, |value| value.valueInteger)
    }

    pub fn writeToNBT(&self) -> NBTTagCompound {
        let mut tag = NBTTagCompound::new();
        for (key, value) in &self.theGameRules {
            tag.setString(key.clone(), value.valueString.clone());
        }
        tag
    }

    pub fn readFromNBT(&mut self, nbt: &NBTTagCompound) {
        let entries = nbt.getKeySet().cloned().collect::<Vec<_>>();
        for key in entries {
            let value = nbt.getString(&key);
            self.setOrCreateGameRule(&key, &value);
        }
    }

    pub fn getRules(&self) -> Vec<&str> {
        self.theGameRules.keys().map(String::as_str).collect()
    }

    pub fn hasRule(&self, name: &str) -> bool { self.theGameRules.contains_key(name) }

    pub fn areSameType(&self, key: &str, otherValue: ValueType) -> bool {
        self.theGameRules.get(key).is_some_and(|value| {
            value.valueType == otherValue || otherValue == ValueType::AnyValue
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_defaults_include_natural_regeneration() {
        let rules = GameRules::new();
        assert!(rules.getBoolean("naturalRegeneration"));
        assert!(rules.getBoolean("doDaylightCycle"));
        assert_eq!(rules.getInt("randomTickSpeed"), 3);
        assert_eq!(rules.getString("gameLoopFunction"), "-");
    }

    #[test]
    fn nbt_round_trip_preserves_rules() {
        let mut rules = GameRules::new();
        rules.setOrCreateGameRule("naturalRegeneration", "false");
        rules.setOrCreateGameRule("customRule", "17");
        let tag = rules.writeToNBT();
        let mut restored = GameRules::new();
        restored.readFromNBT(&tag);
        assert!(!restored.getBoolean("naturalRegeneration"));
        assert_eq!(restored.getInt("customRule"), 17);
        assert!(restored.hasRule("customRule"));
        assert!(restored.areSameType("customRule", ValueType::AnyValue));
    }
}
