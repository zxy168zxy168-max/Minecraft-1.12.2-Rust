use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;
pub struct BedItemColor;
impl IFixableData for BedItemColor {
    fn getFixVersion(&self) -> i32 {
        1125
    }
    fn fixTagCompound(&self, mut c: NBTTagCompound) -> NBTTagCompound {
        if c.getString("id") == "minecraft:bed" && c.getShort("Damage") == 0 {
            c.setShort("Damage", 14);
        }
        c
    }
}
