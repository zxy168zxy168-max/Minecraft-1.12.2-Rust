use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;
pub struct ShulkerBoxTileColor;
impl IFixableData for ShulkerBoxTileColor {
    fn getFixVersion(&self) -> i32 {
        813
    }
    fn fixTagCompound(&self, mut c: NBTTagCompound) -> NBTTagCompound {
        if c.getString("id") == "minecraft:shulker" {
            c.removeTag("Color");
        }
        c
    }
}
