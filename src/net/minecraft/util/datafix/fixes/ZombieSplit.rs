use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;
pub struct ZombieSplit;
impl IFixableData for ZombieSplit {
    fn getFixVersion(&self) -> i32 {
        702
    }
    fn fixTagCompound(&self, mut c: NBTTagCompound) -> NBTTagCompound {
        if c.getString("id") == "Zombie" {
            match c.getInteger("ZombieType") {
                1..=5 => {
                    let p = c.getInteger("ZombieType") - 1;
                    c.setString("id", "ZombieVillager");
                    c.setInteger("Profession", p);
                }
                6 => c.setString("id", "Husk"),
                _ => {}
            }
            c.removeTag("ZombieType");
        }
        c
    }
}
