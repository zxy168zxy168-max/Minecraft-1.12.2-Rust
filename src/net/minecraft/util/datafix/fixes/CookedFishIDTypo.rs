use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;
pub struct CookedFishIDTypo;
impl IFixableData for CookedFishIDTypo {
    fn getFixVersion(&self) -> i32 {
        502
    }
    fn fixTagCompound(&self, mut c: NBTTagCompound) -> NBTTagCompound {
        let id = c.getString("id");
        if id == "cooked_fished" || id == "minecraft:cooked_fished" {
            c.setString("id", "minecraft:cooked_fish");
        }
        c
    }
}
