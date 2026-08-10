use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

/// MCP 1.12.2 `EntityId` (DataVersion 704).
pub struct EntityId;
impl IFixableData for EntityId {
    fn getFixVersion(&self) -> i32 { 704 }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        let replacement = match compound.getString("id").as_str() {
            "AreaEffectCloud" => Some("minecraft:area_effect_cloud"), "ArmorStand" => Some("minecraft:armor_stand"),
            "Arrow" => Some("minecraft:arrow"), "Bat" => Some("minecraft:bat"), "Blaze" => Some("minecraft:blaze"),
            "Boat" => Some("minecraft:boat"), "CaveSpider" => Some("minecraft:cave_spider"), "Chicken" => Some("minecraft:chicken"),
            "Cow" => Some("minecraft:cow"), "Creeper" => Some("minecraft:creeper"), "Donkey" => Some("minecraft:donkey"),
            "DragonFireball" => Some("minecraft:dragon_fireball"), "ElderGuardian" => Some("minecraft:elder_guardian"),
            "EnderCrystal" => Some("minecraft:ender_crystal"), "EnderDragon" => Some("minecraft:ender_dragon"),
            "Enderman" => Some("minecraft:enderman"), "Endermite" => Some("minecraft:endermite"),
            "EyeOfEnderSignal" => Some("minecraft:eye_of_ender_signal"), "FallingSand" => Some("minecraft:falling_block"),
            "Fireball" => Some("minecraft:fireball"), "FireworksRocketEntity" => Some("minecraft:fireworks_rocket"),
            "Ghast" => Some("minecraft:ghast"), "Giant" => Some("minecraft:giant"), "Guardian" => Some("minecraft:guardian"),
            "Horse" => Some("minecraft:horse"), "Husk" => Some("minecraft:husk"), "Item" => Some("minecraft:item"),
            "ItemFrame" => Some("minecraft:item_frame"), "LavaSlime" => Some("minecraft:magma_cube"),
            "LeashKnot" => Some("minecraft:leash_knot"), "MinecartChest" => Some("minecraft:chest_minecart"),
            "MinecartCommandBlock" => Some("minecraft:commandblock_minecart"), "MinecartFurnace" => Some("minecraft:furnace_minecart"),
            "MinecartHopper" => Some("minecraft:hopper_minecart"), "MinecartRideable" => Some("minecraft:minecart"),
            "MinecartSpawner" => Some("minecraft:spawner_minecart"), "MinecartTNT" => Some("minecraft:tnt_minecart"),
            "Mule" => Some("minecraft:mule"), "MushroomCow" => Some("minecraft:mooshroom"), "Ozelot" => Some("minecraft:ocelot"),
            "Painting" => Some("minecraft:painting"), "Pig" => Some("minecraft:pig"), "PigZombie" => Some("minecraft:zombie_pigman"),
            "PolarBear" => Some("minecraft:polar_bear"), "PrimedTnt" => Some("minecraft:tnt"), "Rabbit" => Some("minecraft:rabbit"),
            "Sheep" => Some("minecraft:sheep"), "Shulker" => Some("minecraft:shulker"), "ShulkerBullet" => Some("minecraft:shulker_bullet"),
            "Silverfish" => Some("minecraft:silverfish"), "Skeleton" => Some("minecraft:skeleton"),
            "SkeletonHorse" => Some("minecraft:skeleton_horse"), "Slime" => Some("minecraft:slime"),
            "SmallFireball" => Some("minecraft:small_fireball"), "SnowMan" => Some("minecraft:snowman"),
            "Snowball" => Some("minecraft:snowball"), "SpectralArrow" => Some("minecraft:spectral_arrow"),
            "Spider" => Some("minecraft:spider"), "Squid" => Some("minecraft:squid"), "Stray" => Some("minecraft:stray"),
            "ThrownEgg" => Some("minecraft:egg"), "ThrownEnderpearl" => Some("minecraft:ender_pearl"),
            "ThrownExpBottle" => Some("minecraft:xp_bottle"), "ThrownPotion" => Some("minecraft:potion"),
            "Villager" => Some("minecraft:villager"), "VillagerGolem" => Some("minecraft:villager_golem"),
            "Witch" => Some("minecraft:witch"), "WitherBoss" => Some("minecraft:wither"),
            "WitherSkeleton" => Some("minecraft:wither_skeleton"), "WitherSkull" => Some("minecraft:wither_skull"),
            "Wolf" => Some("minecraft:wolf"), "XPOrb" => Some("minecraft:xp_orb"), "Zombie" => Some("minecraft:zombie"),
            "ZombieHorse" => Some("minecraft:zombie_horse"), "ZombieVillager" => Some("minecraft:zombie_villager"),
            _ => None,
        };
        if let Some(id) = replacement { compound.setString("id", id); }
        compound
    }
}
