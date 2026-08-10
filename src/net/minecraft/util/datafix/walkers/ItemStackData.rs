use crate::net::minecraft::nbt::NBTBase::TAG_COMPOUND;
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::FixTypes::FixTypes;
use crate::net::minecraft::util::datafix::IDataFixer::IDataFixer;
use crate::net::minecraft::util::datafix::IDataWalker::IDataWalker;
use crate::net::minecraft::util::datafix::walkers::Filtered::Filtered;

/// MCP 1.12.2 `ItemStackData`, using a registry key instead of Java Class reflection.
pub struct ItemStackData { filtered: Filtered, matchingTags: Vec<&'static str> }
impl ItemStackData { pub fn new(registryName: &str, matchingTags: &[&'static str]) -> Self { Self { filtered: Filtered::new(registryName), matchingTags: matchingTags.to_vec() } } }
impl IDataWalker for ItemStackData {
    fn process(&self, fixer: &dyn IDataFixer, compound: NBTTagCompound, versionIn: i32) -> NBTTagCompound {
        self.filtered.processIf(fixer, compound, versionIn, |fixer, mut compound, versionIn| {
            for key in &self.matchingTags {
                if compound.hasKeyWithType(key, TAG_COMPOUND) {
                    let fixed = fixer.processVersioned(FixTypes::ItemInstance, compound.getCompoundTag(key), versionIn);
                    compound.setCompoundTag(*key, fixed);
                }
            }
            compound
        })
    }
}
