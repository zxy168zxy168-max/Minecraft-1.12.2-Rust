use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;
pub struct PotionWater;
impl IFixableData for PotionWater {
    fn getFixVersion(&self) -> i32 {
        806
    }
    fn fixTagCompound(&self, mut c: NBTTagCompound) -> NBTTagCompound {
        let id = c.getString("id");
        if matches!(
            id.as_str(),
            "minecraft:potion"
                | "minecraft:splash_potion"
                | "minecraft:lingering_potion"
                | "minecraft:tipped_arrow"
        ) {
            let had = c.hasKeyWithType("tag", 10);
            let mut tag = c.getCompoundTag("tag");
            if !tag.hasKeyWithType("Potion", 8) {
                tag.setString("Potion", "minecraft:water");
            }
            if !had {
                c.setCompoundTag("tag", tag);
            } else {
                c.setCompoundTag("tag", tag);
            }
        }
        c
    }
}
