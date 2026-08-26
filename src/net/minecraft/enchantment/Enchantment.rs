use crate::net::minecraft::client::resources::Locale::Locale;

/// Minimal registry-facing projection of MCP 1.12.2 `Enchantment` used by
/// `GuiEnchantment` clue rendering. IDs, translation keys and maximum levels
/// are the values registered by `Enchantment#registerEnchantments`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Enchantment {
    id: i32,
    translationKey: &'static str,
    maxLevel: i32,
}

impl Enchantment {
    pub const fn getEnchantmentID(self) -> i32 {
        self.id
    }
    pub const fn getName(self) -> &'static str {
        self.translationKey
    }
    pub const fn getMaxLevel(self) -> i32 {
        self.maxLevel
    }

    pub fn getTranslatedName(self, level: i32, locale: &Locale) -> String {
        let mut name = locale.translate_key(self.translationKey).to_owned();
        if self.maxLevel != 1 {
            name.push(' ');
            let levelKey = format!("enchantment.level.{level}");
            name.push_str(locale.translate_key(&levelKey));
        }
        name
    }

    pub const fn getEnchantmentByID(id: i32) -> Option<Self> {
        let (translationKey, maxLevel) = match id {
            0 => ("enchantment.protect.all", 4),
            1 => ("enchantment.protect.fire", 4),
            2 => ("enchantment.protect.fall", 4),
            3 => ("enchantment.protect.explosion", 4),
            4 => ("enchantment.protect.projectile", 4),
            5 => ("enchantment.oxygen", 3),
            6 => ("enchantment.waterWorker", 1),
            7 => ("enchantment.thorns", 3),
            8 => ("enchantment.waterWalker", 3),
            9 => ("enchantment.frostWalker", 2),
            10 => ("enchantment.binding_curse", 1),
            16 => ("enchantment.damage.all", 5),
            17 => ("enchantment.damage.undead", 5),
            18 => ("enchantment.damage.arthropods", 5),
            19 => ("enchantment.knockback", 2),
            20 => ("enchantment.fire", 2),
            21 => ("enchantment.lootBonus", 3),
            22 => ("enchantment.sweeping", 3),
            32 => ("enchantment.digging", 5),
            33 => ("enchantment.untouching", 1),
            34 => ("enchantment.durability", 3),
            35 => ("enchantment.lootBonusDigger", 3),
            48 => ("enchantment.arrowDamage", 5),
            49 => ("enchantment.arrowKnockback", 2),
            50 => ("enchantment.arrowFire", 1),
            51 => ("enchantment.arrowInfinite", 1),
            61 => ("enchantment.lootBonusFishing", 3),
            62 => ("enchantment.fishingSpeed", 3),
            70 => ("enchantment.mending", 1),
            71 => ("enchantment.vanishing_curse", 1),
            _ => return None,
        };
        Some(Self {
            id,
            translationKey,
            maxLevel,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_ids_match_1122_enchantment_registration() {
        assert_eq!(
            Enchantment::getEnchantmentByID(22).unwrap().getName(),
            "enchantment.sweeping"
        );
        assert_eq!(
            Enchantment::getEnchantmentByID(48).unwrap().getMaxLevel(),
            5
        );
        assert!(Enchantment::getEnchantmentByID(15).is_none());
    }
}
