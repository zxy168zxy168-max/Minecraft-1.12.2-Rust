use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

pub struct RedundantChanceTags;
impl IFixableData for RedundantChanceTags {
    fn getFixVersion(&self) -> i32 { 113 }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        if compound.hasKeyWithType("HandDropChances", 9) {
            let list=compound.getTagList("HandDropChances",5);
            if list.tagCount()==2 && list.getFloatAt(0)==0.0 && list.getFloatAt(1)==0.0 { compound.removeTag("HandDropChances"); }
        }
        if compound.hasKeyWithType("ArmorDropChances", 9) {
            let list=compound.getTagList("ArmorDropChances",5);
            if list.tagCount()==4 && (0..4).all(|i| list.getFloatAt(i)==0.0) { compound.removeTag("ArmorDropChances"); }
        }
        compound
    }
}
