use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;
pub struct ShulkerBoxEntityColor;
impl IFixableData for ShulkerBoxEntityColor{fn getFixVersion(&self)->i32{808} fn fixTagCompound(&self,mut c:NBTTagCompound)->NBTTagCompound{if c.getString("id")=="minecraft:shulker"&&!c.hasKeyWithType("Color",99){c.setByte("Color",10);}c}}
