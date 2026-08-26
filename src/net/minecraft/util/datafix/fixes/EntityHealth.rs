use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

pub struct EntityHealth;
impl IFixableData for EntityHealth {
    fn getFixVersion(&self) -> i32 {
        109
    }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        let id = compound.getString("id");
        let affected = matches!(
            id.as_str(),
            "ArmorStand"
                | "Bat"
                | "Blaze"
                | "CaveSpider"
                | "Chicken"
                | "Cow"
                | "Creeper"
                | "EnderDragon"
                | "Enderman"
                | "Endermite"
                | "EntityHorse"
                | "Ghast"
                | "Giant"
                | "Guardian"
                | "LavaSlime"
                | "MushroomCow"
                | "Ozelot"
                | "Pig"
                | "PigZombie"
                | "Rabbit"
                | "Sheep"
                | "Shulker"
                | "Silverfish"
                | "Skeleton"
                | "Slime"
                | "SnowMan"
                | "Spider"
                | "Squid"
                | "Villager"
                | "VillagerGolem"
                | "Witch"
                | "WitherBoss"
                | "Wolf"
                | "Zombie"
        );
        if affected {
            let health = if compound.hasKeyWithType("HealF", 99) {
                let value = compound.getFloat("HealF");
                compound.removeTag("HealF");
                value
            } else if compound.hasKeyWithType("Health", 99) {
                compound.getFloat("Health")
            } else {
                return compound;
            };
            compound.setFloat("Health", health);
        }
        compound
    }
}
