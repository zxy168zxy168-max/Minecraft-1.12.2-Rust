use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::FixTypes::FixTypes;

pub trait IDataFixer: Send + Sync {
    fn processVersioned(&self, fixType: FixTypes, compound: NBTTagCompound, versionIn: i32) -> NBTTagCompound;
}
