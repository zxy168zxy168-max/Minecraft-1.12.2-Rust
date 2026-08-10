use crate::net::minecraft::nbt::NBTBase::{TAG_COMPOUND, TAG_STRING};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::FixTypes::FixTypes;
use crate::net::minecraft::util::datafix::IDataFixer::IDataFixer;
use crate::net::minecraft::util::datafix::IDataWalker::IDataWalker;

/// MCP 1.12.2 `walkers.EntityTag`.
pub struct EntityTag;
impl IDataWalker for EntityTag {
    fn process(&self, fixer:&dyn IDataFixer, mut compound:NBTTagCompound, versionIn:i32)->NBTTagCompound {
        if !compound.hasKeyWithType("tag", TAG_COMPOUND) { return compound; }
        let mut tag=compound.getCompoundTag("tag");
        if tag.hasKeyWithType("EntityTag",TAG_COMPOUND) {
            let mut entity=tag.getCompoundTag("EntityTag");
            let item=compound.getString("id");
            let id = if item == "minecraft:armor_stand" {
                if versionIn < 515 { "ArmorStand".to_owned() } else { "minecraft:armor_stand".to_owned() }
            } else if item == "minecraft:spawn_egg" { entity.getString("id") } else { return compound; };
            let removeId=!entity.hasKeyWithType("id",TAG_STRING);
            entity.setString("id",id);
            entity=fixer.processVersioned(FixTypes::Entity,entity,versionIn);
            if removeId { entity.removeTag("id"); }
            tag.setCompoundTag("EntityTag",entity);
            compound.setCompoundTag("tag",tag);
        }
        compound
    }
}
