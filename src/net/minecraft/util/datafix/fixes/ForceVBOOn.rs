use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;
pub struct ForceVBOOn;
impl IFixableData for ForceVBOOn{fn getFixVersion(&self)->i32{505} fn fixTagCompound(&self,mut c:NBTTagCompound)->NBTTagCompound{c.setString("useVbo","true");c}}
