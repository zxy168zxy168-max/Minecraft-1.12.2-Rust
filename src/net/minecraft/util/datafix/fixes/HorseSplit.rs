use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;
pub struct HorseSplit;
impl IFixableData for HorseSplit{fn getFixVersion(&self)->i32{703} fn fixTagCompound(&self,mut c:NBTTagCompound)->NBTTagCompound{if c.getString("id")=="EntityHorse"{let id=match c.getInteger("Type"){1=>"Donkey",2=>"Mule",3=>"ZombieHorse",4=>"SkeletonHorse",_=>"Horse"};c.setString("id",id);c.removeTag("Type");}c}}
