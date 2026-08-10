use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

pub struct SpawnEggNames;
impl SpawnEggNames {
    fn legacyEntityId(index: usize) -> Option<&'static str> {
        match index {
            1 => Some("Item"),
            2 => Some("XPOrb"),
            7 => Some("ThrownEgg"),
            8 => Some("LeashKnot"),
            9 => Some("Painting"),
            10 => Some("Arrow"),
            11 => Some("Snowball"),
            12 => Some("Fireball"),
            13 => Some("SmallFireball"),
            14 => Some("ThrownEnderpearl"),
            15 => Some("EyeOfEnderSignal"),
            16 => Some("ThrownPotion"),
            17 => Some("ThrownExpBottle"),
            18 => Some("ItemFrame"),
            19 => Some("WitherSkull"),
            20 => Some("PrimedTnt"),
            21 => Some("FallingSand"),
            22 => Some("FireworksRocketEntity"),
            23 => Some("TippedArrow"),
            24 => Some("SpectralArrow"),
            25 => Some("ShulkerBullet"),
            26 => Some("DragonFireball"),
            30 => Some("ArmorStand"),
            41 => Some("Boat"),
            42 => Some("MinecartRideable"),
            43 => Some("MinecartChest"),
            44 => Some("MinecartFurnace"),
            45 => Some("MinecartTNT"),
            46 => Some("MinecartHopper"),
            47 => Some("MinecartSpawner"),
            40 => Some("MinecartCommandBlock"),
            48 => Some("Mob"),
            49 => Some("Monster"),
            50 => Some("Creeper"),
            51 => Some("Skeleton"),
            52 => Some("Spider"),
            53 => Some("Giant"),
            54 => Some("Zombie"),
            55 => Some("Slime"),
            56 => Some("Ghast"),
            57 => Some("PigZombie"),
            58 => Some("Enderman"),
            59 => Some("CaveSpider"),
            60 => Some("Silverfish"),
            61 => Some("Blaze"),
            62 => Some("LavaSlime"),
            63 => Some("EnderDragon"),
            64 => Some("WitherBoss"),
            65 => Some("Bat"),
            66 => Some("Witch"),
            67 => Some("Endermite"),
            68 => Some("Guardian"),
            69 => Some("Shulker"),
            90 => Some("Pig"),
            91 => Some("Sheep"),
            92 => Some("Cow"),
            93 => Some("Chicken"),
            94 => Some("Squid"),
            95 => Some("Wolf"),
            96 => Some("MushroomCow"),
            97 => Some("SnowMan"),
            98 => Some("Ozelot"),
            99 => Some("VillagerGolem"),
            100 => Some("EntityHorse"),
            101 => Some("Rabbit"),
            120 => Some("Villager"),
            200 => Some("EnderCrystal"),
            _ => None,
        }
    }
}
impl IFixableData for SpawnEggNames {
    fn getFixVersion(&self) -> i32 { 105 }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        if compound.getString("id") == "minecraft:spawn_egg" {
            let mut tag = compound.getCompoundTag("tag");
            let mut entity = tag.getCompoundTag("EntityTag");
            let damage = compound.getShort("Damage");
            if !entity.hasKeyWithType("id", 8) {
                if let Some(id) = Self::legacyEntityId((damage as i32 & 255) as usize) {
                    entity.setString("id", id);
                    tag.setCompoundTag("EntityTag", entity);
                    compound.setCompoundTag("tag", tag);
                }
            }
            if damage != 0 { compound.setShort("Damage", 0); }
        }
        compound
    }
}
