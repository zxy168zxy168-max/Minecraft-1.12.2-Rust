use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;
pub struct SkeletonSplit;
impl IFixableData for SkeletonSplit {
    fn getFixVersion(&self) -> i32 {
        701
    }
    fn fixTagCompound(&self, mut c: NBTTagCompound) -> NBTTagCompound {
        if c.getString("id") == "Skeleton" {
            match c.getInteger("SkeletonType") {
                1 => c.setString("id", "WitherSkeleton"),
                2 => c.setString("id", "Stray"),
                _ => {}
            }
            c.removeTag("SkeletonType");
        }
        c
    }
}
