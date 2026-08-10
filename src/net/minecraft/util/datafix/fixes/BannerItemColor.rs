use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

pub struct BannerItemColor;
impl IFixableData for BannerItemColor {
    fn getFixVersion(&self) -> i32 { 804 }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        if compound.getString("id") != "minecraft:banner" || !compound.hasKeyWithType("tag", 10) { return compound; }
        let mut tag = compound.getCompoundTag("tag");
        if !tag.hasKeyWithType("BlockEntityTag", 10) { return compound; }
        let mut block_entity = tag.getCompoundTag("BlockEntityTag");
        if !block_entity.hasKeyWithType("Base", 99) { return compound; }
        compound.setShort("Damage", block_entity.getShort("Base") & 15);
        if tag.hasKeyWithType("display", 10) {
            let display = tag.getCompoundTag("display");
            if display.hasKeyWithType("Lore", 9) {
                let lore = display.getTagList("Lore", 8);
                if lore.tagCount() == 1 && lore.getStringTagAt(0) == "(+NBT)" { return compound; }
            }
        }
        block_entity.removeTag("Base");
        if block_entity.hasNoTags() { tag.removeTag("BlockEntityTag"); }
        else { tag.setCompoundTag("BlockEntityTag", block_entity); }
        if tag.hasNoTags() { compound.removeTag("tag"); }
        else { compound.setCompoundTag("tag", tag); }
        compound
    }
}
