use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;

pub trait IFixableData: Send + Sync {
    fn getFixVersion(&self) -> i32;
    fn fixTagCompound(&self, compound: NBTTagCompound) -> NBTTagCompound;
}
