use crate::net::minecraft::nbt::NBTBase::{NBTBase, TAG_COMPOUND};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

pub struct SpawnerEntityTypes;
impl IFixableData for SpawnerEntityTypes {
    fn getFixVersion(&self) -> i32 { 107 }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        if compound.getString("id") != "MobSpawner" { return compound; }
        if compound.hasKeyWithType("EntityId", 8) {
            let id = compound.getString("EntityId");
            let mut spawn_data = compound.getCompoundTag("SpawnData");
            spawn_data.setString("id", if id.is_empty() { "Pig" } else { id.as_str() });
            compound.setCompoundTag("SpawnData", spawn_data);
            compound.removeTag("EntityId");
        }
        if compound.hasKeyWithType("SpawnPotentials", 9) {
            let mut potentials = compound.getTagList("SpawnPotentials", TAG_COMPOUND);
            for index in 0..potentials.tagCount() {
                let mut potential = potentials.getCompoundTagAt(index);
                if potential.hasKeyWithType("Type", 8) {
                    let mut properties = potential.getCompoundTag("Properties");
                    properties.setString("id", potential.getString("Type"));
                    potential.setCompoundTag("Entity", properties);
                    potential.removeTag("Type");
                    potential.removeTag("Properties");
                    potentials.set(index, NBTBase::Compound(potential));
                }
            }
            compound.setTagList("SpawnPotentials", potentials);
        }
        compound
    }
}
