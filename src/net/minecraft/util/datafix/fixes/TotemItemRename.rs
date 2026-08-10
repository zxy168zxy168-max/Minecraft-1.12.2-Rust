use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;
pub struct TotemItemRename;
impl IFixableData for TotemItemRename{fn getFixVersion(&self)->i32{820} fn fixTagCompound(&self,mut c:NBTTagCompound)->NBTTagCompound{if c.getString("id")=="minecraft:totem"{c.setString("id","minecraft:totem_of_undying");}c}}
