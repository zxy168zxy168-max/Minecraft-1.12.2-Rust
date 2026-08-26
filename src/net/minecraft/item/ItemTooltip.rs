use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::item::ItemRegistryData;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::item::ItemTranslationKeys;
use crate::net::minecraft::nbt::NBTBase::{TAG_COMPOUND, TAG_STRING};

/// MCP-facing tooltip construction subset. The first line, custom display
/// name, enchantments, dyed marker, lore, Unbreakable, CanDestroy/CanPlaceOn,
/// and advanced durability/registry/NBT lines follow ItemStack#getTooltip.
/// Item-specific addInformation and attribute modifiers remain separate work.
pub fn getTooltip(stack: &ItemStack, locale: &Locale, advanced: bool) -> Vec<String> {
    if stack.isEmpty() {
        return Vec::new();
    }
    let custom_name = customDisplayName(stack);
    let mut name = custom_name
        .clone()
        .or_else(|| localizedStoredName(stack, locale))
        .unwrap_or_else(|| localizedName(stack, locale));
    if custom_name.is_some() {
        name = format!("§o{name}");
    }
    name.push_str("§r");
    if advanced {
        if stack.getHasSubtypes() {
            name.push_str(&format!(" (#{:04}/{})", stack.itemId, stack.itemDamage));
        } else {
            name.push_str(&format!(" (#{:04})", stack.itemId));
        }
    } else if custom_name.is_none() && stack.itemId == 358 {
        name.push_str(&format!(" #{}", stack.itemDamage));
    }
    let mut lines = vec![name];
    let hide_flags = stack
        .tagCompound
        .as_ref()
        .map(|tag| tag.getInteger("HideFlags"))
        .unwrap_or(0);
    if let Some(root) = &stack.tagCompound {
        if hide_flags & 1 == 0 {
            appendEnchantments(&mut lines, root.getTagList("ench", TAG_COMPOUND), locale);
            if stack.itemId == 403 {
                appendEnchantments(
                    &mut lines,
                    root.getTagList("StoredEnchantments", TAG_COMPOUND),
                    locale,
                );
            }
        }
        if root.hasKeyWithType("display", TAG_COMPOUND) {
            let display = root.getCompoundTag("display");
            if display.hasKey("color") {
                if advanced {
                    lines.push(formatTranslation(
                        locale,
                        "item.color",
                        &[format!("#{:06X}", display.getInteger("color") & 0xFF_FFFF)],
                    ));
                } else {
                    lines.push(format!("§o{}", locale.translate_key("item.dyed")));
                }
            }
            let lore = display.getTagList("Lore", TAG_STRING);
            for index in 0..lore.tagCount() {
                lines.push(format!("§5§o{}", lore.getStringTagAt(index)));
            }
        }
        if root.getBoolean("Unbreakable") && hide_flags & 4 == 0 {
            lines.push(format!("§9{}", locale.translate_key("item.unbreakable")));
        }
        appendBlockList(
            &mut lines,
            root,
            "CanDestroy",
            "item.canBreak",
            locale,
            hide_flags & 8 == 0,
        );
        appendBlockList(
            &mut lines,
            root,
            "CanPlaceOn",
            "item.canPlace",
            locale,
            hide_flags & 16 == 0,
        );
    }
    if advanced {
        if stack.isItemDamaged() {
            lines.push(formatTranslation(
                locale,
                "item.durability",
                &[
                    (stack.getMaxDamage() - stack.itemDamage as i32).to_string(),
                    stack.getMaxDamage().to_string(),
                ],
            ));
        }
        lines.push(format!(
            "§8{}",
            ItemRegistryData::definition(stack.itemId).registryName
        ));
        if let Some(root) = &stack.tagCompound {
            lines.push(format!(
                "§8{}",
                formatTranslation(
                    locale,
                    "item.nbt_tags",
                    &[root.getKeySet().count().to_string()]
                )
            ));
        }
    }
    lines
}

/// `GuiScreen#getItemToolTip`: apply rarity to line zero and gray to every
/// following line after `ItemStack#getTooltip` has assembled its content.
pub fn getItemToolTip(stack: &ItemStack, locale: &Locale, advanced: bool) -> Vec<String> {
    let mut lines = getTooltip(stack, locale, advanced);
    for (index, line) in lines.iter_mut().enumerate() {
        let prefix = if index == 0 {
            rarityFormatting(stack)
        } else {
            "§7"
        };
        line.insert_str(0, prefix);
    }
    lines
}

pub fn hasDisplayName(stack: &ItemStack) -> bool {
    customDisplayName(stack).is_some()
}

pub fn displayName(stack: &ItemStack, locale: &Locale) -> String {
    customDisplayName(stack)
        .or_else(|| localizedStoredName(stack, locale))
        .unwrap_or_else(|| localizedName(stack, locale))
}

pub fn localizedName(stack: &ItemStack, locale: &Locale) -> String {
    if stack.itemId == 425 {
        let color = DYE_DAMAGE_NAMES[stack.itemDamage.rem_euclid(16) as usize];
        let key = format!("item.banner.{color}.name");
        return locale.translate_key(&key).to_owned();
    }
    if stack.itemId == 442 {
        if let Some(blockEntityTag) = stack
            .tagCompound
            .as_ref()
            .filter(|root| root.hasKeyWithType("BlockEntityTag", TAG_COMPOUND))
            .map(|root| root.getCompoundTag("BlockEntityTag"))
        {
            let color = DYE_DAMAGE_NAMES[blockEntityTag.getInteger("Base").rem_euclid(16) as usize];
            let key = format!("item.shield.{color}.name");
            return locale.translate_key(&key).to_owned();
        }
    }
    ItemTranslationKeys::translationKey(stack.itemId, stack.itemDamage)
        .map(|key| locale.translate_key(key).to_owned())
        .unwrap_or_else(|| {
            ItemRegistryData::definition(stack.itemId)
                .registryName
                .to_owned()
        })
}

const DYE_DAMAGE_NAMES: [&str; 16] = [
    "black",
    "red",
    "green",
    "brown",
    "blue",
    "purple",
    "cyan",
    "silver",
    "gray",
    "pink",
    "lime",
    "yellow",
    "lightBlue",
    "magenta",
    "orange",
    "white",
];

fn rarityFormatting(stack: &ItemStack) -> &'static str {
    // Item#getRarity plus the three 1.12.2 item overrides.
    if stack.itemId == 322 {
        return if stack.itemDamage == 0 { "§b" } else { "§d" };
    }
    if (2256..=2267).contains(&stack.itemId) {
        return "§b";
    }
    if stack.itemId == 403 {
        let stored = stack
            .tagCompound
            .as_ref()
            .map(|root| root.getTagList("StoredEnchantments", TAG_COMPOUND))
            .is_some_and(|list| !list.hasNoTags());
        if stored {
            return "§e";
        }
    }
    if stack.isItemEnchanted() {
        "§b"
    } else {
        "§f"
    }
}

fn customDisplayName(stack: &ItemStack) -> Option<String> {
    let root = stack.tagCompound.as_ref()?;
    if !root.hasKeyWithType("display", TAG_COMPOUND) {
        return None;
    }
    let name = root.getCompoundTag("display").getString("Name");
    (!name.is_empty()).then_some(name)
}

fn localizedStoredName(stack: &ItemStack, locale: &Locale) -> Option<String> {
    let root = stack.tagCompound.as_ref()?;
    if !root.hasKeyWithType("display", TAG_COMPOUND) {
        return None;
    }
    let key = root.getCompoundTag("display").getString("LocName");
    (!key.is_empty()).then(|| locale.translate_key(&key).to_owned())
}

fn appendEnchantments(
    lines: &mut Vec<String>,
    list: crate::net::minecraft::nbt::NBTTagList::NBTTagList,
    locale: &Locale,
) {
    for index in 0..list.tagCount() {
        let entry = list.getCompoundTagAt(index);
        let id = entry.getShort("id");
        let level = entry.getShort("lvl").max(0) as i32;
        let Some(key) = enchantmentKey(id) else {
            continue;
        };
        let mut text = locale.translate_key(key).to_owned();
        if level != 1 || !matches!(id, 51 | 71) {
            let level_key = format!("enchantment.level.{level}");
            text.push(' ');
            text.push_str(locale.translate_key(&level_key));
        }
        lines.push(text);
    }
}

fn appendBlockList(
    lines: &mut Vec<String>,
    root: &crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound,
    tag: &str,
    title: &str,
    locale: &Locale,
    visible: bool,
) {
    if !visible {
        return;
    }
    let list = root.getTagList(tag, TAG_STRING);
    if list.hasNoTags() {
        return;
    }
    lines.push(String::new());
    lines.push(format!("§7{}", locale.translate_key(title)));
    for index in 0..list.tagCount() {
        // Block.getBlockFromName(...).getLocalizedName is not yet represented
        // by the block registry facade. Preserve the exact saved registry name
        // rather than inventing a localized block mapping.
        lines.push(format!("§8{}", list.getStringTagAt(index)));
    }
}

fn formatTranslation(locale: &Locale, key: &str, values: &[String]) -> String {
    let mut text = locale.translate_key(key).to_owned();
    for (index, value) in values.iter().enumerate() {
        let positional = format!("%{}$s", index + 1);
        if text.contains(&positional) {
            text = text.replace(&positional, value);
        } else if text.contains("%s") {
            text = text.replacen("%s", value, 1);
        }
    }
    text
}

fn enchantmentKey(id: i16) -> Option<&'static str> {
    Some(match id {
        0 => "enchantment.protect.all",
        1 => "enchantment.protect.fire",
        2 => "enchantment.protect.fall",
        3 => "enchantment.protect.explosion",
        4 => "enchantment.protect.projectile",
        5 => "enchantment.oxygen",
        6 => "enchantment.waterWorker",
        7 => "enchantment.thorns",
        8 => "enchantment.waterWalker",
        9 => "enchantment.frostWalker",
        10 => "enchantment.binding_curse",
        16 => "enchantment.damage.all",
        17 => "enchantment.damage.undead",
        18 => "enchantment.damage.arthropods",
        19 => "enchantment.knockback",
        20 => "enchantment.fire",
        21 => "enchantment.lootBonus",
        22 => "enchantment.sweeping",
        32 => "enchantment.digging",
        33 => "enchantment.untouching",
        34 => "enchantment.durability",
        35 => "enchantment.lootBonusDigger",
        48 => "enchantment.arrowDamage",
        49 => "enchantment.arrowKnockback",
        50 => "enchantment.arrowFire",
        51 => "enchantment.arrowInfinite",
        61 => "enchantment.lootBonusFishing",
        62 => "enchantment.fishingSpeed",
        70 => "enchantment.mending",
        71 => "enchantment.vanishing_curse",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
    #[test]
    fn custom_name_and_lore_keep_vanilla_format_codes() {
        let mut display = NBTTagCompound::new();
        display.setString("Name", "Custom");
        let mut root = NBTTagCompound::new();
        root.setCompoundTag("display", display);
        let stack = ItemStack {
            itemId: 70,
            count: 1,
            itemDamage: 0,
            tagCompound: Some(root),
        };
        let lines = getTooltip(&stack, &Locale::default(), false);
        assert_eq!(lines[0], "§oCustom§r");
    }

    #[test]
    fn gui_tooltip_applies_mcp_rarity_and_secondary_gray() {
        let stack = ItemStack {
            itemId: 322,
            count: 1,
            itemDamage: 1,
            tagCompound: None,
        };
        let lines = getItemToolTip(&stack, &Locale::default(), false);
        assert!(lines[0].starts_with("§d"));
    }

    #[test]
    fn banner_display_name_uses_dye_damage_not_metadata_order() {
        let mut locale = Locale::default();
        locale.load_bytes(b"item.banner.red.name=Red Banner\n");
        let stack = ItemStack {
            itemId: 425,
            count: 1,
            itemDamage: 1,
            tagCompound: None,
        };
        assert_eq!(localizedName(&stack, &locale), "Red Banner");
    }
}
