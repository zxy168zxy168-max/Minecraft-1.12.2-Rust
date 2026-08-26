use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

pub struct MinecartEntityTypes;
impl IFixableData for MinecartEntityTypes {
    fn getFixVersion(&self) -> i32 {
        106
    }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        if compound.getString("id") == "Minecart" {
            const TYPES: [&str; 7] = [
                "MinecartRideable",
                "MinecartChest",
                "MinecartFurnace",
                "MinecartTNT",
                "MinecartSpawner",
                "MinecartHopper",
                "MinecartCommandBlock",
            ];
            let i = compound.getInteger("Type");
            let id = if i > 0 && (i as usize) < TYPES.len() {
                TYPES[i as usize]
            } else {
                TYPES[0]
            };
            compound.setString("id", id);
            compound.removeTag("Type");
        }
        compound
    }
}
