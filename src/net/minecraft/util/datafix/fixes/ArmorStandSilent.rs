use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;
pub struct ArmorStandSilent;
impl IFixableData for ArmorStandSilent {
    fn getFixVersion(&self) -> i32 {
        147
    }
    fn fixTagCompound(&self, mut c: NBTTagCompound) -> NBTTagCompound {
        if c.getString("id") == "ArmorStand" && c.getBoolean("Silent") && !c.getBoolean("Marker") {
            c.removeTag("Silent");
        }
        c
    }
}
