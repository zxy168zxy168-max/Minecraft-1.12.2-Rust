use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// Source registry row from MCP 1.12.2 `EntityList#init`.
///
/// `javaClass` is retained as authority/provenance only. Runtime construction
/// remains deliberately separate until that concrete Rust entity class exists;
/// registry metadata must never masquerade as an instantiated entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityRegistryEntry {
    pub numericId: i32,
    pub registryPath: &'static str,
    pub javaClass: &'static str,
    pub legacyName: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityEggInfo {
    pub registryPath: &'static str,
    pub primaryColor: i32,
    pub secondaryColor: i32,
}

pub struct EntityList;

const ENTITY_REGISTRY: [EntityRegistryEntry; 83] = [
    EntityRegistryEntry {
        numericId: 1,
        registryPath: "item",
        javaClass: "EntityItem",
        legacyName: "Item",
    },
    EntityRegistryEntry {
        numericId: 2,
        registryPath: "xp_orb",
        javaClass: "EntityXPOrb",
        legacyName: "XPOrb",
    },
    EntityRegistryEntry {
        numericId: 3,
        registryPath: "area_effect_cloud",
        javaClass: "EntityAreaEffectCloud",
        legacyName: "AreaEffectCloud",
    },
    EntityRegistryEntry {
        numericId: 4,
        registryPath: "elder_guardian",
        javaClass: "EntityElderGuardian",
        legacyName: "ElderGuardian",
    },
    EntityRegistryEntry {
        numericId: 5,
        registryPath: "wither_skeleton",
        javaClass: "EntityWitherSkeleton",
        legacyName: "WitherSkeleton",
    },
    EntityRegistryEntry {
        numericId: 6,
        registryPath: "stray",
        javaClass: "EntityStray",
        legacyName: "Stray",
    },
    EntityRegistryEntry {
        numericId: 7,
        registryPath: "egg",
        javaClass: "EntityEgg",
        legacyName: "ThrownEgg",
    },
    EntityRegistryEntry {
        numericId: 8,
        registryPath: "leash_knot",
        javaClass: "EntityLeashKnot",
        legacyName: "LeashKnot",
    },
    EntityRegistryEntry {
        numericId: 9,
        registryPath: "painting",
        javaClass: "EntityPainting",
        legacyName: "Painting",
    },
    EntityRegistryEntry {
        numericId: 10,
        registryPath: "arrow",
        javaClass: "EntityTippedArrow",
        legacyName: "Arrow",
    },
    EntityRegistryEntry {
        numericId: 11,
        registryPath: "snowball",
        javaClass: "EntitySnowball",
        legacyName: "Snowball",
    },
    EntityRegistryEntry {
        numericId: 12,
        registryPath: "fireball",
        javaClass: "EntityLargeFireball",
        legacyName: "Fireball",
    },
    EntityRegistryEntry {
        numericId: 13,
        registryPath: "small_fireball",
        javaClass: "EntitySmallFireball",
        legacyName: "SmallFireball",
    },
    EntityRegistryEntry {
        numericId: 14,
        registryPath: "ender_pearl",
        javaClass: "EntityEnderPearl",
        legacyName: "ThrownEnderpearl",
    },
    EntityRegistryEntry {
        numericId: 15,
        registryPath: "eye_of_ender_signal",
        javaClass: "EntityEnderEye",
        legacyName: "EyeOfEnderSignal",
    },
    EntityRegistryEntry {
        numericId: 16,
        registryPath: "potion",
        javaClass: "EntityPotion",
        legacyName: "ThrownPotion",
    },
    EntityRegistryEntry {
        numericId: 17,
        registryPath: "xp_bottle",
        javaClass: "EntityExpBottle",
        legacyName: "ThrownExpBottle",
    },
    EntityRegistryEntry {
        numericId: 18,
        registryPath: "item_frame",
        javaClass: "EntityItemFrame",
        legacyName: "ItemFrame",
    },
    EntityRegistryEntry {
        numericId: 19,
        registryPath: "wither_skull",
        javaClass: "EntityWitherSkull",
        legacyName: "WitherSkull",
    },
    EntityRegistryEntry {
        numericId: 20,
        registryPath: "tnt",
        javaClass: "EntityTNTPrimed",
        legacyName: "PrimedTnt",
    },
    EntityRegistryEntry {
        numericId: 21,
        registryPath: "falling_block",
        javaClass: "EntityFallingBlock",
        legacyName: "FallingSand",
    },
    EntityRegistryEntry {
        numericId: 22,
        registryPath: "fireworks_rocket",
        javaClass: "EntityFireworkRocket",
        legacyName: "FireworksRocketEntity",
    },
    EntityRegistryEntry {
        numericId: 23,
        registryPath: "husk",
        javaClass: "EntityHusk",
        legacyName: "Husk",
    },
    EntityRegistryEntry {
        numericId: 24,
        registryPath: "spectral_arrow",
        javaClass: "EntitySpectralArrow",
        legacyName: "SpectralArrow",
    },
    EntityRegistryEntry {
        numericId: 25,
        registryPath: "shulker_bullet",
        javaClass: "EntityShulkerBullet",
        legacyName: "ShulkerBullet",
    },
    EntityRegistryEntry {
        numericId: 26,
        registryPath: "dragon_fireball",
        javaClass: "EntityDragonFireball",
        legacyName: "DragonFireball",
    },
    EntityRegistryEntry {
        numericId: 27,
        registryPath: "zombie_villager",
        javaClass: "EntityZombieVillager",
        legacyName: "ZombieVillager",
    },
    EntityRegistryEntry {
        numericId: 28,
        registryPath: "skeleton_horse",
        javaClass: "EntitySkeletonHorse",
        legacyName: "SkeletonHorse",
    },
    EntityRegistryEntry {
        numericId: 29,
        registryPath: "zombie_horse",
        javaClass: "EntityZombieHorse",
        legacyName: "ZombieHorse",
    },
    EntityRegistryEntry {
        numericId: 30,
        registryPath: "armor_stand",
        javaClass: "EntityArmorStand",
        legacyName: "ArmorStand",
    },
    EntityRegistryEntry {
        numericId: 31,
        registryPath: "donkey",
        javaClass: "EntityDonkey",
        legacyName: "Donkey",
    },
    EntityRegistryEntry {
        numericId: 32,
        registryPath: "mule",
        javaClass: "EntityMule",
        legacyName: "Mule",
    },
    EntityRegistryEntry {
        numericId: 33,
        registryPath: "evocation_fangs",
        javaClass: "EntityEvokerFangs",
        legacyName: "EvocationFangs",
    },
    EntityRegistryEntry {
        numericId: 34,
        registryPath: "evocation_illager",
        javaClass: "EntityEvoker",
        legacyName: "EvocationIllager",
    },
    EntityRegistryEntry {
        numericId: 35,
        registryPath: "vex",
        javaClass: "EntityVex",
        legacyName: "Vex",
    },
    EntityRegistryEntry {
        numericId: 36,
        registryPath: "vindication_illager",
        javaClass: "EntityVindicator",
        legacyName: "VindicationIllager",
    },
    EntityRegistryEntry {
        numericId: 37,
        registryPath: "illusion_illager",
        javaClass: "EntityIllusionIllager",
        legacyName: "IllusionIllager",
    },
    EntityRegistryEntry {
        numericId: 40,
        registryPath: "commandblock_minecart",
        javaClass: "EntityMinecartCommandBlock",
        legacyName: "MinecartCommandBlock",
    },
    EntityRegistryEntry {
        numericId: 41,
        registryPath: "boat",
        javaClass: "EntityBoat",
        legacyName: "Boat",
    },
    EntityRegistryEntry {
        numericId: 42,
        registryPath: "minecart",
        javaClass: "EntityMinecartEmpty",
        legacyName: "MinecartRideable",
    },
    EntityRegistryEntry {
        numericId: 43,
        registryPath: "chest_minecart",
        javaClass: "EntityMinecartChest",
        legacyName: "MinecartChest",
    },
    EntityRegistryEntry {
        numericId: 44,
        registryPath: "furnace_minecart",
        javaClass: "EntityMinecartFurnace",
        legacyName: "MinecartFurnace",
    },
    EntityRegistryEntry {
        numericId: 45,
        registryPath: "tnt_minecart",
        javaClass: "EntityMinecartTNT",
        legacyName: "MinecartTNT",
    },
    EntityRegistryEntry {
        numericId: 46,
        registryPath: "hopper_minecart",
        javaClass: "EntityMinecartHopper",
        legacyName: "MinecartHopper",
    },
    EntityRegistryEntry {
        numericId: 47,
        registryPath: "spawner_minecart",
        javaClass: "EntityMinecartMobSpawner",
        legacyName: "MinecartSpawner",
    },
    EntityRegistryEntry {
        numericId: 50,
        registryPath: "creeper",
        javaClass: "EntityCreeper",
        legacyName: "Creeper",
    },
    EntityRegistryEntry {
        numericId: 51,
        registryPath: "skeleton",
        javaClass: "EntitySkeleton",
        legacyName: "Skeleton",
    },
    EntityRegistryEntry {
        numericId: 52,
        registryPath: "spider",
        javaClass: "EntitySpider",
        legacyName: "Spider",
    },
    EntityRegistryEntry {
        numericId: 53,
        registryPath: "giant",
        javaClass: "EntityGiantZombie",
        legacyName: "Giant",
    },
    EntityRegistryEntry {
        numericId: 54,
        registryPath: "zombie",
        javaClass: "EntityZombie",
        legacyName: "Zombie",
    },
    EntityRegistryEntry {
        numericId: 55,
        registryPath: "slime",
        javaClass: "EntitySlime",
        legacyName: "Slime",
    },
    EntityRegistryEntry {
        numericId: 56,
        registryPath: "ghast",
        javaClass: "EntityGhast",
        legacyName: "Ghast",
    },
    EntityRegistryEntry {
        numericId: 57,
        registryPath: "zombie_pigman",
        javaClass: "EntityPigZombie",
        legacyName: "PigZombie",
    },
    EntityRegistryEntry {
        numericId: 58,
        registryPath: "enderman",
        javaClass: "EntityEnderman",
        legacyName: "Enderman",
    },
    EntityRegistryEntry {
        numericId: 59,
        registryPath: "cave_spider",
        javaClass: "EntityCaveSpider",
        legacyName: "CaveSpider",
    },
    EntityRegistryEntry {
        numericId: 60,
        registryPath: "silverfish",
        javaClass: "EntitySilverfish",
        legacyName: "Silverfish",
    },
    EntityRegistryEntry {
        numericId: 61,
        registryPath: "blaze",
        javaClass: "EntityBlaze",
        legacyName: "Blaze",
    },
    EntityRegistryEntry {
        numericId: 62,
        registryPath: "magma_cube",
        javaClass: "EntityMagmaCube",
        legacyName: "LavaSlime",
    },
    EntityRegistryEntry {
        numericId: 63,
        registryPath: "ender_dragon",
        javaClass: "EntityDragon",
        legacyName: "EnderDragon",
    },
    EntityRegistryEntry {
        numericId: 64,
        registryPath: "wither",
        javaClass: "EntityWither",
        legacyName: "WitherBoss",
    },
    EntityRegistryEntry {
        numericId: 65,
        registryPath: "bat",
        javaClass: "EntityBat",
        legacyName: "Bat",
    },
    EntityRegistryEntry {
        numericId: 66,
        registryPath: "witch",
        javaClass: "EntityWitch",
        legacyName: "Witch",
    },
    EntityRegistryEntry {
        numericId: 67,
        registryPath: "endermite",
        javaClass: "EntityEndermite",
        legacyName: "Endermite",
    },
    EntityRegistryEntry {
        numericId: 68,
        registryPath: "guardian",
        javaClass: "EntityGuardian",
        legacyName: "Guardian",
    },
    EntityRegistryEntry {
        numericId: 69,
        registryPath: "shulker",
        javaClass: "EntityShulker",
        legacyName: "Shulker",
    },
    EntityRegistryEntry {
        numericId: 90,
        registryPath: "pig",
        javaClass: "EntityPig",
        legacyName: "Pig",
    },
    EntityRegistryEntry {
        numericId: 91,
        registryPath: "sheep",
        javaClass: "EntitySheep",
        legacyName: "Sheep",
    },
    EntityRegistryEntry {
        numericId: 92,
        registryPath: "cow",
        javaClass: "EntityCow",
        legacyName: "Cow",
    },
    EntityRegistryEntry {
        numericId: 93,
        registryPath: "chicken",
        javaClass: "EntityChicken",
        legacyName: "Chicken",
    },
    EntityRegistryEntry {
        numericId: 94,
        registryPath: "squid",
        javaClass: "EntitySquid",
        legacyName: "Squid",
    },
    EntityRegistryEntry {
        numericId: 95,
        registryPath: "wolf",
        javaClass: "EntityWolf",
        legacyName: "Wolf",
    },
    EntityRegistryEntry {
        numericId: 96,
        registryPath: "mooshroom",
        javaClass: "EntityMooshroom",
        legacyName: "MushroomCow",
    },
    EntityRegistryEntry {
        numericId: 97,
        registryPath: "snowman",
        javaClass: "EntitySnowman",
        legacyName: "SnowMan",
    },
    EntityRegistryEntry {
        numericId: 98,
        registryPath: "ocelot",
        javaClass: "EntityOcelot",
        legacyName: "Ozelot",
    },
    EntityRegistryEntry {
        numericId: 99,
        registryPath: "villager_golem",
        javaClass: "EntityIronGolem",
        legacyName: "VillagerGolem",
    },
    EntityRegistryEntry {
        numericId: 100,
        registryPath: "horse",
        javaClass: "EntityHorse",
        legacyName: "Horse",
    },
    EntityRegistryEntry {
        numericId: 101,
        registryPath: "rabbit",
        javaClass: "EntityRabbit",
        legacyName: "Rabbit",
    },
    EntityRegistryEntry {
        numericId: 102,
        registryPath: "polar_bear",
        javaClass: "EntityPolarBear",
        legacyName: "PolarBear",
    },
    EntityRegistryEntry {
        numericId: 103,
        registryPath: "llama",
        javaClass: "EntityLlama",
        legacyName: "Llama",
    },
    EntityRegistryEntry {
        numericId: 104,
        registryPath: "llama_spit",
        javaClass: "EntityLlamaSpit",
        legacyName: "LlamaSpit",
    },
    EntityRegistryEntry {
        numericId: 105,
        registryPath: "parrot",
        javaClass: "EntityParrot",
        legacyName: "Parrot",
    },
    EntityRegistryEntry {
        numericId: 120,
        registryPath: "villager",
        javaClass: "EntityVillager",
        legacyName: "Villager",
    },
    EntityRegistryEntry {
        numericId: 200,
        registryPath: "ender_crystal",
        javaClass: "EntityEnderCrystal",
        legacyName: "EnderCrystal",
    },
];

const ENTITY_EGGS: [EntityEggInfo; 43] = [
    EntityEggInfo {
        registryPath: "bat",
        primaryColor: 4996656,
        secondaryColor: 986895,
    },
    EntityEggInfo {
        registryPath: "blaze",
        primaryColor: 16167425,
        secondaryColor: 16775294,
    },
    EntityEggInfo {
        registryPath: "cave_spider",
        primaryColor: 803406,
        secondaryColor: 11013646,
    },
    EntityEggInfo {
        registryPath: "chicken",
        primaryColor: 10592673,
        secondaryColor: 16711680,
    },
    EntityEggInfo {
        registryPath: "cow",
        primaryColor: 4470310,
        secondaryColor: 10592673,
    },
    EntityEggInfo {
        registryPath: "creeper",
        primaryColor: 894731,
        secondaryColor: 0,
    },
    EntityEggInfo {
        registryPath: "donkey",
        primaryColor: 5457209,
        secondaryColor: 8811878,
    },
    EntityEggInfo {
        registryPath: "elder_guardian",
        primaryColor: 13552826,
        secondaryColor: 7632531,
    },
    EntityEggInfo {
        registryPath: "enderman",
        primaryColor: 1447446,
        secondaryColor: 0,
    },
    EntityEggInfo {
        registryPath: "endermite",
        primaryColor: 1447446,
        secondaryColor: 7237230,
    },
    EntityEggInfo {
        registryPath: "evocation_illager",
        primaryColor: 9804699,
        secondaryColor: 1973274,
    },
    EntityEggInfo {
        registryPath: "ghast",
        primaryColor: 16382457,
        secondaryColor: 12369084,
    },
    EntityEggInfo {
        registryPath: "guardian",
        primaryColor: 5931634,
        secondaryColor: 15826224,
    },
    EntityEggInfo {
        registryPath: "horse",
        primaryColor: 12623485,
        secondaryColor: 15656192,
    },
    EntityEggInfo {
        registryPath: "husk",
        primaryColor: 7958625,
        secondaryColor: 15125652,
    },
    EntityEggInfo {
        registryPath: "llama",
        primaryColor: 12623485,
        secondaryColor: 10051392,
    },
    EntityEggInfo {
        registryPath: "magma_cube",
        primaryColor: 3407872,
        secondaryColor: 16579584,
    },
    EntityEggInfo {
        registryPath: "mooshroom",
        primaryColor: 10489616,
        secondaryColor: 12040119,
    },
    EntityEggInfo {
        registryPath: "mule",
        primaryColor: 1769984,
        secondaryColor: 5321501,
    },
    EntityEggInfo {
        registryPath: "ocelot",
        primaryColor: 15720061,
        secondaryColor: 5653556,
    },
    EntityEggInfo {
        registryPath: "parrot",
        primaryColor: 894731,
        secondaryColor: 16711680,
    },
    EntityEggInfo {
        registryPath: "pig",
        primaryColor: 15771042,
        secondaryColor: 14377823,
    },
    EntityEggInfo {
        registryPath: "polar_bear",
        primaryColor: 15921906,
        secondaryColor: 9803152,
    },
    EntityEggInfo {
        registryPath: "rabbit",
        primaryColor: 10051392,
        secondaryColor: 7555121,
    },
    EntityEggInfo {
        registryPath: "sheep",
        primaryColor: 15198183,
        secondaryColor: 16758197,
    },
    EntityEggInfo {
        registryPath: "shulker",
        primaryColor: 9725844,
        secondaryColor: 5060690,
    },
    EntityEggInfo {
        registryPath: "silverfish",
        primaryColor: 7237230,
        secondaryColor: 3158064,
    },
    EntityEggInfo {
        registryPath: "skeleton",
        primaryColor: 12698049,
        secondaryColor: 4802889,
    },
    EntityEggInfo {
        registryPath: "skeleton_horse",
        primaryColor: 6842447,
        secondaryColor: 15066584,
    },
    EntityEggInfo {
        registryPath: "slime",
        primaryColor: 5349438,
        secondaryColor: 8306542,
    },
    EntityEggInfo {
        registryPath: "spider",
        primaryColor: 3419431,
        secondaryColor: 11013646,
    },
    EntityEggInfo {
        registryPath: "squid",
        primaryColor: 2243405,
        secondaryColor: 7375001,
    },
    EntityEggInfo {
        registryPath: "stray",
        primaryColor: 6387319,
        secondaryColor: 14543594,
    },
    EntityEggInfo {
        registryPath: "vex",
        primaryColor: 8032420,
        secondaryColor: 15265265,
    },
    EntityEggInfo {
        registryPath: "villager",
        primaryColor: 5651507,
        secondaryColor: 12422002,
    },
    EntityEggInfo {
        registryPath: "vindication_illager",
        primaryColor: 9804699,
        secondaryColor: 2580065,
    },
    EntityEggInfo {
        registryPath: "witch",
        primaryColor: 3407872,
        secondaryColor: 5349438,
    },
    EntityEggInfo {
        registryPath: "wither_skeleton",
        primaryColor: 1315860,
        secondaryColor: 4672845,
    },
    EntityEggInfo {
        registryPath: "wolf",
        primaryColor: 14144467,
        secondaryColor: 13545366,
    },
    EntityEggInfo {
        registryPath: "zombie",
        primaryColor: 44975,
        secondaryColor: 7969893,
    },
    EntityEggInfo {
        registryPath: "zombie_horse",
        primaryColor: 3232308,
        secondaryColor: 9945732,
    },
    EntityEggInfo {
        registryPath: "zombie_pigman",
        primaryColor: 15373203,
        secondaryColor: 5009705,
    },
    EntityEggInfo {
        registryPath: "zombie_villager",
        primaryColor: 5651507,
        secondaryColor: 7969893,
    },
];

impl EntityList {
    pub const PLAYER: &'static str = "minecraft:player";
    pub const LIGHTNING_BOLT: &'static str = "minecraft:lightning_bolt";

    pub fn init() -> &'static [EntityRegistryEntry] {
        &ENTITY_REGISTRY
    }
    pub fn entityEggs() -> &'static [EntityEggInfo] {
        &ENTITY_EGGS
    }

    pub fn getClassFromID(entityID: i32) -> Option<&'static str> {
        ENTITY_REGISTRY
            .iter()
            .find(|entry| entry.numericId == entityID)
            .map(|entry| entry.javaClass)
    }

    pub fn getKeyByID(entityID: i32) -> Option<ResourceLocation> {
        ENTITY_REGISTRY
            .iter()
            .find(|entry| entry.numericId == entityID)
            .map(|entry| ResourceLocation::parse(entry.registryPath))
    }

    pub fn getEntryByName(name: &ResourceLocation) -> Option<&'static EntityRegistryEntry> {
        ENTITY_REGISTRY
            .iter()
            .find(|entry| ResourceLocation::parse(entry.registryPath) == *name)
    }

    /// MCP `func_192839_a` lookup without construction.
    pub fn func_192839_a(name: &str) -> Option<&'static EntityRegistryEntry> {
        Self::getEntryByName(&ResourceLocation::parse(name))
    }

    /// MCP legacy serializer name (`field_191311_g`) by registry key.
    pub fn func_191302_a(name: &ResourceLocation) -> Option<&'static str> {
        Self::getEntryByName(name).map(|entry| entry.legacyName)
    }

    pub fn getEntityNameList() -> Vec<ResourceLocation> {
        ENTITY_REGISTRY
            .iter()
            .map(|entry| ResourceLocation::parse(entry.registryPath))
            .collect()
    }

    pub fn isStringValidEntityName(entityName: &ResourceLocation) -> bool {
        entityName.to_string() == Self::PLAYER || Self::getEntryByName(entityName).is_some()
    }

    /// Registry half of MCP `createEntityFromNBT`. Full construction/readFromNBT
    /// is intentionally not claimed until every concrete server entity factory
    /// is available. This predicate lets persistence distinguish a valid 1.12.2
    /// vanilla identity from malformed/unknown data without fabricating objects.
    pub fn isRegisteredEntityNBT(nbt: &NBTTagCompound) -> bool {
        Self::func_192839_a(&nbt.getString("id")).is_some()
    }

    pub fn getEggInfo(name: &ResourceLocation) -> Option<&'static EntityEggInfo> {
        ENTITY_EGGS
            .iter()
            .find(|entry| ResourceLocation::parse(entry.registryPath) == *name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_registry_has_all_vanilla_1122_entries() {
        assert_eq!(EntityList::init().len(), 83);
        assert_eq!(
            EntityList::getKeyByID(63).unwrap().to_string(),
            "minecraft:ender_dragon"
        );
        assert_eq!(
            EntityList::getKeyByID(58).unwrap().to_string(),
            "minecraft:enderman"
        );
        assert_eq!(
            EntityList::getKeyByID(94).unwrap().to_string(),
            "minecraft:squid"
        );
        assert_eq!(
            EntityList::func_191302_a(&ResourceLocation::parse("minecart")).unwrap(),
            "MinecartRideable"
        );
    }
    #[test]
    fn source_spawn_egg_table_is_complete() {
        assert_eq!(EntityList::entityEggs().len(), 43);
        let creeper = EntityList::getEggInfo(&ResourceLocation::parse("creeper")).unwrap();
        assert_eq!((creeper.primaryColor, creeper.secondaryColor), (894731, 0));
    }
}
