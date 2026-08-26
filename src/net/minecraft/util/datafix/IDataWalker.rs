use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IDataFixer::IDataFixer;

pub trait IDataWalker: Send + Sync {
    fn process(
        &self,
        fixer: &dyn IDataFixer,
        compound: NBTTagCompound,
        versionIn: i32,
    ) -> NBTTagCompound;
}
