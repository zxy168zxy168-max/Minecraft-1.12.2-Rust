use uuid::Uuid;
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

pub struct StringToUUID;
impl IFixableData for StringToUUID {
    fn getFixVersion(&self) -> i32 { 108 }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        if compound.hasKeyWithType("UUID", 8) {
            let uuid = Uuid::parse_str(&compound.getString("UUID"))
                .expect("StringToUUID received invalid UUID text");
            compound.setUniqueId("UUID", uuid);
        }
        compound
    }
}
