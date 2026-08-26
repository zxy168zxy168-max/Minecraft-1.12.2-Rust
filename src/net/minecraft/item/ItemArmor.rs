use crate::net::minecraft::inventory::EntityEquipmentSlot::EntityEquipmentSlot;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::nbt::NBTBase::TAG_COMPOUND;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `ItemArmor.ArmorMaterial` values relevant to client rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArmorMaterial {
    Leather,
    Chain,
    Iron,
    Gold,
    Diamond,
}

impl ArmorMaterial {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Leather => "leather",
            Self::Chain => "chainmail",
            Self::Iron => "iron",
            Self::Gold => "gold",
            Self::Diamond => "diamond",
        }
    }

    pub const fn defaultColor(self) -> i32 {
        if matches!(self, Self::Leather) {
            0xA06540
        } else {
            0xFFFFFF
        }
    }

    /// MCP `ArmorMaterial#getDamageReductionAmount`.
    pub const fn damageReduction(self, slot: EntityEquipmentSlot) -> i32 {
        let values: [i32; 4] = match self {
            Self::Leather => [1, 2, 3, 1],
            Self::Chain => [1, 4, 5, 2],
            Self::Iron => [2, 5, 6, 2],
            Self::Gold => [1, 3, 5, 2],
            Self::Diamond => [3, 6, 8, 3],
        };
        match slot {
            EntityEquipmentSlot::Feet => values[0],
            EntityEquipmentSlot::Legs => values[1],
            EntityEquipmentSlot::Chest => values[2],
            EntityEquipmentSlot::Head => values[3],
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArmorDefinition {
    pub material: ArmorMaterial,
    pub slot: EntityEquipmentSlot,
}

/// Rendering-facing subset of MCP `ItemArmor`.
pub struct ItemArmor;

impl ItemArmor {
    pub const fn definition(itemId: i16) -> Option<ArmorDefinition> {
        let (material, offset) = match itemId {
            298..=301 => (ArmorMaterial::Leather, itemId - 298),
            302..=305 => (ArmorMaterial::Chain, itemId - 302),
            306..=309 => (ArmorMaterial::Iron, itemId - 306),
            310..=313 => (ArmorMaterial::Diamond, itemId - 310),
            314..=317 => (ArmorMaterial::Gold, itemId - 314),
            _ => return None,
        };
        let slot = match offset {
            0 => EntityEquipmentSlot::Head,
            1 => EntityEquipmentSlot::Chest,
            2 => EntityEquipmentSlot::Legs,
            3 => EntityEquipmentSlot::Feet,
            _ => return None,
        };
        Some(ArmorDefinition { material, slot })
    }

    pub const fn isArmor(stack: &ItemStack) -> bool {
        Self::definition(stack.itemId).is_some()
    }

    pub const fn isElytra(stack: &ItemStack) -> bool {
        !stack.isEmpty() && stack.itemId == 443
    }

    pub fn getColor(stack: &ItemStack) -> i32 {
        let Some(definition) = Self::definition(stack.itemId) else {
            return 0xFFFFFF;
        };
        if definition.material != ArmorMaterial::Leather {
            return 0xFFFFFF;
        }
        let Some(tag) = stack.tagCompound.as_ref() else {
            return definition.material.defaultColor();
        };
        if !tag.hasKeyWithType("display", TAG_COMPOUND) {
            return definition.material.defaultColor();
        }
        let display = tag.getCompoundTag("display");
        if display.hasKey("color") {
            display.getInteger("color") & 0xFFFFFF
        } else {
            definition.material.defaultColor()
        }
    }

    pub fn texture(stack: &ItemStack, overlay: bool) -> Option<ResourceLocation> {
        let definition = Self::definition(stack.itemId)?;
        let layer = if definition.slot == EntityEquipmentSlot::Legs {
            2
        } else {
            1
        };
        let suffix = if overlay && definition.material == ArmorMaterial::Leather {
            "_overlay"
        } else {
            ""
        };
        Some(ResourceLocation::new(
            "minecraft",
            format!(
                "textures/models/armor/{}_layer_{}{}.png",
                definition.material.name(),
                layer,
                suffix,
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;

    #[test]
    fn vanilla_ids_map_to_material_and_slot() {
        assert_eq!(
            ItemArmor::definition(298).unwrap().slot,
            EntityEquipmentSlot::Head
        );
        assert_eq!(
            ItemArmor::definition(300).unwrap().slot,
            EntityEquipmentSlot::Legs
        );
        assert_eq!(
            ItemArmor::definition(313).unwrap().material,
            ArmorMaterial::Diamond
        );
        assert_eq!(
            ItemArmor::definition(317).unwrap().slot,
            EntityEquipmentSlot::Feet
        );
        assert!(ItemArmor::definition(443).is_none());
    }

    #[test]
    fn leather_color_uses_display_color_and_vanilla_default() {
        let plain = ItemStack {
            itemId: 299,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        };
        assert_eq!(ItemArmor::getColor(&plain), 0xA06540);

        let mut display = NBTTagCompound::new();
        display.setInteger("color", 0x123456);
        let mut root = NBTTagCompound::new();
        root.setTag(
            "display",
            crate::net::minecraft::nbt::NBTBase::NBTBase::Compound(display),
        );
        let dyed = ItemStack {
            tagCompound: Some(root),
            ..plain
        };
        assert_eq!(ItemArmor::getColor(&dyed), 0x123456);
    }

    #[test]
    fn damage_reduction_matches_armor_material_tables() {
        assert_eq!(
            ArmorMaterial::Leather.damageReduction(EntityEquipmentSlot::Feet),
            1
        );
        assert_eq!(
            ArmorMaterial::Leather.damageReduction(EntityEquipmentSlot::Legs),
            2
        );
        assert_eq!(
            ArmorMaterial::Leather.damageReduction(EntityEquipmentSlot::Chest),
            3
        );
        assert_eq!(
            ArmorMaterial::Leather.damageReduction(EntityEquipmentSlot::Head),
            1
        );
        assert_eq!(
            ArmorMaterial::Iron.damageReduction(EntityEquipmentSlot::Chest),
            6
        );
        assert_eq!(
            ArmorMaterial::Diamond.damageReduction(EntityEquipmentSlot::Head),
            3
        );
        assert_eq!(
            ArmorMaterial::Chain.damageReduction(EntityEquipmentSlot::Legs),
            4
        );
        assert_eq!(
            ArmorMaterial::Gold.damageReduction(EntityEquipmentSlot::Chest),
            5
        );
    }

    #[test]
    fn leggings_use_layer_two_and_leather_has_overlay() {
        let leggings = ItemStack {
            itemId: 300,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        };
        assert_eq!(
            ItemArmor::texture(&leggings, false).unwrap().getPath(),
            "textures/models/armor/leather_layer_2.png",
        );
        assert_eq!(
            ItemArmor::texture(&leggings, true).unwrap().getPath(),
            "textures/models/armor/leather_layer_2_overlay.png",
        );
    }
}
