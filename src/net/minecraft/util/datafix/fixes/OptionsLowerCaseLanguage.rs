use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;
pub struct OptionsLowerCaseLanguage;
impl IFixableData for OptionsLowerCaseLanguage{fn getFixVersion(&self)->i32{816} fn fixTagCompound(&self,mut c:NBTTagCompound)->NBTTagCompound{if c.hasKeyWithType("lang",8){c.setString("lang",c.getString("lang").to_lowercase());}c}}
