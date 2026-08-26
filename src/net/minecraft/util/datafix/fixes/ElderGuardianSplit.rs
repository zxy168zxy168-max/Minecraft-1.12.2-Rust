use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;
pub struct ElderGuardianSplit;
impl IFixableData for ElderGuardianSplit {
    fn getFixVersion(&self) -> i32 {
        700
    }
    fn fixTagCompound(&self, mut c: NBTTagCompound) -> NBTTagCompound {
        if c.getString("id") == "Guardian" {
            if c.getBoolean("Elder") {
                c.setString("id", "ElderGuardian");
            }
            c.removeTag("Elder");
        }
        c
    }
}
