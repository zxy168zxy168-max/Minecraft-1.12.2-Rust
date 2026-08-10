use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

pub struct HorseSaddle;
impl IFixableData for HorseSaddle {
    fn getFixVersion(&self) -> i32 { 110 }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        if compound.getString("id") == "EntityHorse" && !compound.hasKeyWithType("SaddleItem", 10) && compound.getBoolean("Saddle") {
            let mut saddle = NBTTagCompound::new();
            saddle.setString("id", "minecraft:saddle"); saddle.setByte("Count", 1); saddle.setShort("Damage", 0);
            compound.setCompoundTag("SaddleItem", saddle); compound.removeTag("Saddle");
        }
        compound
    }
}
