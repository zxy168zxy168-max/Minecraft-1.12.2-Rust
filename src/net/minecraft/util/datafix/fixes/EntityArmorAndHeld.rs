use crate::net::minecraft::nbt::NBTBase::{NBTBase, TAG_COMPOUND};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::nbt::NBTTagList::NBTTagList;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

/// MCP 1.12.2 `EntityArmorAndHeld` (DataVersion 100).
pub struct EntityArmorAndHeld;
impl IFixableData for EntityArmorAndHeld {
    fn getFixVersion(&self) -> i32 {
        100
    }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        let equipment = compound.getTagList("Equipment", TAG_COMPOUND);
        if !equipment.hasNoTags() && !compound.hasKeyWithType("HandItems", 10) {
            let mut hand = NBTTagList::new();
            if let Some(first) = equipment.tags().first().cloned() {
                hand.appendTag(first);
            }
            hand.appendTag(NBTBase::Compound(NBTTagCompound::new()));
            compound.setTagList("HandItems", hand);
        }
        // Deliberately preserves the decompiled 1.12.2 key test `ArmorItem`
        // (singular) while writing `ArmorItems`, exactly as MCP source.
        if equipment.tagCount() > 1 && !compound.hasKeyWithType("ArmorItem", 10) {
            let mut armor = NBTTagList::new();
            for index in 1..=4 {
                armor.appendTag(NBTBase::Compound(equipment.getCompoundTagAt(index)));
            }
            compound.setTagList("ArmorItems", armor);
        }
        compound.removeTag("Equipment");

        if compound.hasKeyWithType("DropChances", 9) {
            let chances = compound.getTagList("DropChances", 5);
            if !compound.hasKeyWithType("HandDropChances", 10) {
                let mut hand = NBTTagList::new();
                hand.appendTag(NBTBase::Float(chances.getFloatAt(0)));
                hand.appendTag(NBTBase::Float(0.0));
                compound.setTagList("HandDropChances", hand);
            }
            if !compound.hasKeyWithType("ArmorDropChances", 10) {
                let mut armor = NBTTagList::new();
                for index in 1..=4 {
                    armor.appendTag(NBTBase::Float(chances.getFloatAt(index)));
                }
                compound.setTagList("ArmorDropChances", armor);
            }
            compound.removeTag("DropChances");
        }
        compound
    }
}
