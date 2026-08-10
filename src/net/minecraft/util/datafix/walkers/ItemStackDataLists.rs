use crate::net::minecraft::nbt::NBTBase::{NBTBase, TAG_COMPOUND, TAG_LIST};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::FixTypes::FixTypes;
use crate::net::minecraft::util::datafix::IDataFixer::IDataFixer;
use crate::net::minecraft::util::datafix::IDataWalker::IDataWalker;
use crate::net::minecraft::util::datafix::walkers::Filtered::Filtered;

/// MCP 1.12.2 `ItemStackDataLists`, using a registry key instead of Java Class reflection.
pub struct ItemStackDataLists { filtered: Filtered, matchingTags: Vec<&'static str> }
impl ItemStackDataLists { pub fn new(registryName: &str, matchingTags: &[&'static str]) -> Self { Self { filtered: Filtered::new(registryName), matchingTags: matchingTags.to_vec() } } }
impl IDataWalker for ItemStackDataLists {
    fn process(&self, fixer: &dyn IDataFixer, compound: NBTTagCompound, versionIn: i32) -> NBTTagCompound {
        self.filtered.processIf(fixer, compound, versionIn, |fixer, mut compound, versionIn| {
            for key in &self.matchingTags {
                if compound.hasKeyWithType(key, TAG_LIST) {
                    let mut list=compound.getTagList(key, TAG_COMPOUND);
                    for index in 0..list.tagCount() {
                        let fixed=fixer.processVersioned(FixTypes::ItemInstance,list.getCompoundTagAt(index),versionIn);
                        list.set(index,NBTBase::Compound(fixed));
                    }
                    compound.setTagList(*key,list);
                }
            }
            compound
        })
    }
}
