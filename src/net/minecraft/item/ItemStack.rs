use std::sync::Arc;
use crate::net::minecraft::util::datafix::DataFixer::DataFixer;
use crate::net::minecraft::util::datafix::FixTypes::FixTypes;
use crate::net::minecraft::util::datafix::walkers::BlockEntityTag::BlockEntityTag;
use crate::net::minecraft::util::datafix::walkers::EntityTag::EntityTag;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::Block::Block;
use crate::net::minecraft::item::Item::Item;
use crate::net::minecraft::item::ItemRegistryData::{definition as itemDefinition, itemIdByNameOrId};
use crate::net::minecraft::nbt::NBTBase::{TAG_COMPOUND, TAG_STRING};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::item::EnumAction::EnumAction;
use crate::net::minecraft::network::PacketBuffer::{read_i16_be, read_nbt_compound, read_u8, write_i16_be, CodecError};

/// Network-bearing MCP `ItemStack` subset with the mining behavior currently
/// required by PlayerControllerMP. Packet decoding, tag equality, enchantment
/// lookup, destroy speed and harvestability follow the 1.12.2 paths.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ItemStack {
    pub itemId: i16,
    pub count: u8,
    pub itemDamage: i16,
    pub tagCompound: Option<NBTTagCompound>,
}

impl ItemStack {
    /// MCP 1.12.2 `ItemStack#registerFixes`.
    pub fn registerFixes(fixer: &mut DataFixer) {
        fixer.registerWalker(FixTypes::ItemInstance, Arc::new(BlockEntityTag));
        fixer.registerWalker(FixTypes::ItemInstance, Arc::new(EntityTag));
    }

    pub const EMPTY: Self = Self { itemId: -1, count: 0, itemDamage: 0, tagCompound: None };

    /// MCP `ItemStack(NBTTagCompound)`: registry-name ID, signed Count byte,
    /// non-negative Damage and an optional compound `tag`. Unknown IDs become
    /// EMPTY instead of fabricating an item.
    pub fn fromNBT(nbt: &NBTTagCompound) -> Self {
        let Some(itemId) = itemIdByNameOrId(&nbt.getString("id")) else { return Self::EMPTY; };
        if itemId == 0 { return Self::EMPTY; }
        let count = nbt.getByte("Count");
        if count <= 0 { return Self::EMPTY; }
        let itemDamage = nbt.getShort("Damage").max(0);
        let tagCompound = nbt.hasKeyWithType("tag", TAG_COMPOUND).then(|| nbt.getCompoundTag("tag"));
        Self { itemId, count: count as u8, itemDamage, tagCompound }
    }

    /// MCP `ItemStack#writeToNBT`.
    pub fn writeToNBT(&self, nbt: &mut NBTTagCompound) {
        let registry = if self.isEmpty() { "minecraft:air" } else { itemDefinition(self.itemId).registryName };
        nbt.setString("id", registry);
        nbt.setByte("Count", self.count as i8);
        nbt.setShort("Damage", self.itemDamage);
        if let Some(tag) = &self.tagCompound { nbt.setCompoundTag("tag", tag.clone()); }
    }

    pub fn readFromBuffer(input: &mut &[u8]) -> Result<Self, CodecError> {
        let itemId = read_i16_be(input)?;
        if itemId < 0 {
            return Ok(Self::EMPTY);
        }
        let count = read_u8(input)?;
        let itemDamage = read_i16_be(input)?;
        let tagCompound = read_nbt_compound(input)?;
        Ok(Self { itemId, count, itemDamage, tagCompound })
    }

    pub const fn isEmpty(&self) -> bool { self.itemId < 0 || self.count == 0 }

    pub fn writeToBuffer(&self, output: &mut Vec<u8>) -> Result<(), CodecError> {
        if self.isEmpty() {
            write_i16_be(-1, output);
            return Ok(());
        }
        write_i16_be(self.itemId, output);
        output.push(self.count);
        write_i16_be(self.itemDamage, output);
        match &self.tagCompound {
            None => output.push(0),
            Some(tag) => crate::net::minecraft::nbt::CompressedStreamTools::writeRoot(tag, output)
                .map_err(CodecError::Io)?,
        }
        Ok(())
    }

    pub const fn getCount(&self) -> i32 { self.count as i32 }
    pub fn setCount(&mut self, count: i32) {
        if count <= 0 { *self = Self::EMPTY; } else { self.count = count.min(u8::MAX as i32) as u8; }
    }
    pub fn shrink(&mut self, amount: i32) { self.setCount(self.getCount() - amount.max(0)); }
    pub fn grow(&mut self, amount: i32) { self.setCount(self.getCount() + amount.max(0)); }
    pub fn splitStack(&mut self, amount: i32) -> Self {
        if self.isEmpty() || amount <= 0 { return Self::EMPTY; }
        let removed = amount.min(self.getCount());
        let result = Self { count: removed as u8, ..self.clone() };
        self.shrink(removed);
        result
    }
    pub fn copy(&self) -> Self { self.clone() }
    pub fn getMaxStackSize(&self) -> i32 { if self.isEmpty() { 64 } else { Item::getItemStackLimit(self.itemId) } }
    pub fn getMaxDamage(&self) -> i32 { if self.isEmpty() { 0 } else { Item::getMaxDamage(self.itemId) } }
    pub fn getItemUseAction(&self) -> EnumAction { if self.isEmpty() { EnumAction::None } else { Item::getItemUseAction(self.itemId) } }
    pub fn getMaxItemUseDuration(&self) -> i32 { if self.isEmpty() { 0 } else { Item::getMaxItemUseDuration(self.itemId) } }
    pub fn getHasSubtypes(&self) -> bool { !self.isEmpty() && Item::getHasSubtypes(self.itemId) }
    pub fn isFood(&self) -> bool { !self.isEmpty() && Item::isFood(self.itemId) }
    pub fn isAlwaysEdible(&self) -> bool { !self.isEmpty() && Item::isAlwaysEdible(self.itemId) }
    pub fn isItemDamaged(&self) -> bool { self.isItemStackDamageable() && self.itemDamage > 0 }
    pub fn showDurabilityBar(&self) -> bool { self.isItemDamaged() }
    pub fn getDurabilityForDisplay(&self) -> f64 {
        let maximum = self.getMaxDamage();
        if maximum <= 0 { 0.0 } else { self.itemDamage.max(0) as f64 / maximum as f64 }
    }
    /// MCP `ItemStack.hasDisplayName`: `display.Name` is a string tag.
    pub fn hasDisplayName(&self) -> bool {
        let Some(tag) = &self.tagCompound else { return false; };
        if !tag.hasKeyWithType("display", crate::net::minecraft::nbt::NBTBase::TAG_COMPOUND) { return false; }
        tag.getCompoundTag("display").hasKeyWithType("Name", crate::net::minecraft::nbt::NBTBase::TAG_STRING)
    }

    pub fn isItemEnchanted(&self) -> bool {
        self.tagCompound.as_ref().is_some_and(|tag| !tag.getTagList("ench", TAG_COMPOUND).hasNoTags())
    }
    pub fn hasEffect(&self) -> bool {
        if self.isItemEnchanted() { return true; }
        if matches!(self.itemId, 384 | 387 | 399 | 403 | 426) { return true; }
        if self.itemId == 322 && self.itemDamage > 0 { return true; }
        if matches!(self.itemId, 373 | 438 | 441) {
            let Some(tag) = &self.tagCompound else { return false; };
            if !tag.getTagList("CustomPotionEffects", TAG_COMPOUND).hasNoTags() {
                return true;
            }
            let potion = tag.getString("Potion");
            return !potion.is_empty()
                && !matches!(potion.as_str(),
                    "minecraft:water" | "minecraft:mundane" | "minecraft:thick" | "minecraft:awkward");
        }
        false
    }
    /// MCP `ItemStack.isItemEqual`: compares only registered item and damage.
    pub fn isItemEqual(&self, other: &Self) -> bool {
        !self.isEmpty()
            && !other.isEmpty()
            && self.itemId == other.itemId
            && self.itemDamage == other.itemDamage
    }

    /// MCP `ItemStack.areItemsEqual`.
    pub fn areItemsEqual(left: &Self, right: &Self) -> bool {
        std::ptr::eq(left, right) || left.isItemEqual(right)
    }

    pub fn canStackWith(&self, other: &Self) -> bool {
        self.isItemEqual(other) && Self::areItemStackTagsEqual(self, other)
    }

    /// MCP `ItemStack.getStrVsBlock` -> `Item.getDestroySpeed`.
    pub fn getStrVsBlock(&self, state: IBlockState) -> f32 {
        if self.isEmpty() { 1.0 } else { Item::getDestroySpeed(self.itemId, state.getBlockId()) }
    }

    /// MCP `ItemStack.canHarvestBlock`.
    pub fn canHarvestBlock(&self, state: IBlockState) -> bool {
        !self.isEmpty() && Item::canHarvestBlock(self.itemId, state.getBlockId())
    }

    /// MCP `ItemStack#canPlaceOn`. Java caches the last block lookup; the
    /// cache is an implementation detail and does not affect observable
    /// semantics, so Rust performs the tiny NBT-list scan directly.
    pub fn canPlaceOn(&self, block: Block) -> bool {
        let Some(tag) = &self.tagCompound else { return false; };
        if !tag.hasKeyWithType("CanPlaceOn", crate::net::minecraft::nbt::NBTBase::TAG_LIST) {
            return false;
        }
        let list = tag.getTagList("CanPlaceOn", TAG_STRING);
        for index in 0..list.tagCount() {
            if Block::getBlockFromName(&list.getStringTagAt(index)).is_some_and(|candidate| candidate == block) {
                return true;
            }
        }
        false
    }

    /// MCP `ItemStack#canEditBlocks` -> `Item#canItemEditBlocks`. None of the
    /// vanilla 1.12.2 Item subclasses override that base method, so the
    /// registered vanilla result is always false.
    pub const fn canEditBlocks(&self) -> bool { false }

    /// MCP `ItemStack.isItemStackDamageable`: the registered item must have
    /// positive max damage and an `Unbreakable` byte tag disables durability.
    pub fn isItemStackDamageable(&self) -> bool {
        !self.isEmpty()
            && Item::isDamageable(self.itemId)
            && !self.tagCompound.as_ref().is_some_and(|tag| tag.getBoolean("Unbreakable"))
    }

    pub fn areItemStackTagsEqual(left: &ItemStack, right: &ItemStack) -> bool {
        left.tagCompound == right.tagCompound
    }

    /// Exact NBT path used by `EnchantmentHelper.getEnchantmentLevel` for
    /// normal enchanted items in 1.12.2 (`ench` list of compound tags).
    pub fn getEnchantmentLevel(&self, enchantmentId: i16) -> i32 {
        let Some(tag) = &self.tagCompound else { return 0; };
        let enchantments = tag.getTagList("ench", TAG_COMPOUND);
        for index in 0..enchantments.tagCount() {
            let entry = enchantments.getCompoundTagAt(index);
            if entry.getShort("id") == enchantmentId {
                return entry.getShort("lvl").max(0) as i32;
            }
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nbt_round_trip_uses_registry_name_count_damage_and_tag() {
        let mut custom=NBTTagCompound::new(); custom.setString("display-test","yes");
        let original=ItemStack{itemId:57,count:23,itemDamage:2,tagCompound:Some(custom)};
        let mut nbt=NBTTagCompound::new(); original.writeToNBT(&mut nbt);
        assert_eq!(nbt.getString("id"),"minecraft:diamond_block");
        assert_eq!(nbt.getByte("Count"),23); assert_eq!(nbt.getShort("Damage"),2);
        assert_eq!(ItemStack::fromNBT(&nbt),original);
    }

    #[test]
    fn can_place_on_reads_vanilla_adventure_tag() {
        let mut tag = NBTTagCompound::new();
        let mut list = crate::net::minecraft::nbt::NBTTagList::NBTTagList::new();
        list.appendTag(crate::net::minecraft::nbt::NBTBase::NBTBase::String("minecraft:grass".to_owned()));
        tag.setTagList("CanPlaceOn", list);
        let stack = ItemStack { itemId: 290, count: 1, itemDamage: 0, tagCompound: Some(tag) };
        assert!(stack.canPlaceOn(Block::getBlockById(2)));
        assert!(!stack.canPlaceOn(Block::getBlockById(1)));
        assert!(!stack.canEditBlocks());
    }

    #[test]
    fn can_place_on_accepts_legacy_numeric_block_names_like_mcp() {
        let mut tag = NBTTagCompound::new();
        let mut list = crate::net::minecraft::nbt::NBTTagList::NBTTagList::new();
        list.appendTag(crate::net::minecraft::nbt::NBTBase::NBTBase::String("2".to_owned()));
        tag.setTagList("CanPlaceOn", list);
        let stack = ItemStack { itemId: 294, count: 1, itemDamage: 0, tagCompound: Some(tag) };
        assert!(stack.canPlaceOn(Block::getBlockById(2)));
        assert!(!stack.canPlaceOn(Block::getBlockById(1)));
    }

    #[test]
    fn damageability_uses_the_full_registry_and_unbreakable_tag() {
        let bow = ItemStack { itemId: 261, count: 1, itemDamage: 0, tagCompound: None };
        assert!(bow.isItemStackDamageable());

        let mut unbreakable = NBTTagCompound::new();
        unbreakable.setBoolean("Unbreakable", true);
        let protected = ItemStack { tagCompound: Some(unbreakable), ..bow };
        assert!(!protected.isItemStackDamageable());

        let stick = ItemStack { itemId: 280, count: 1, itemDamage: 0, tagCompound: None };
        assert!(!stick.isItemStackDamageable());
    }
}
