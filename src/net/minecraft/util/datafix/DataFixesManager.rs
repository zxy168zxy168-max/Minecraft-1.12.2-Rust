use std::sync::Arc;

use crate::net::minecraft::entity::Entity::Entity;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::nbt::NBTBase::{NBTBase, TAG_COMPOUND, TAG_LIST};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::fixes::AddBedTileEntity::AddBedTileEntity;
use crate::net::minecraft::util::datafix::fixes::ArmorStandSilent::ArmorStandSilent;
use crate::net::minecraft::util::datafix::fixes::BannerItemColor::BannerItemColor;
use crate::net::minecraft::util::datafix::fixes::BedItemColor::BedItemColor;
use crate::net::minecraft::util::datafix::fixes::BookPagesStrictJSON::BookPagesStrictJSON;
use crate::net::minecraft::util::datafix::fixes::CookedFishIDTypo::CookedFishIDTypo;
use crate::net::minecraft::util::datafix::fixes::ElderGuardianSplit::ElderGuardianSplit;
use crate::net::minecraft::util::datafix::fixes::EntityArmorAndHeld::EntityArmorAndHeld;
use crate::net::minecraft::util::datafix::fixes::EntityHealth::EntityHealth;
use crate::net::minecraft::util::datafix::fixes::EntityId::EntityId;
use crate::net::minecraft::util::datafix::fixes::ForceVBOOn::ForceVBOOn;
use crate::net::minecraft::util::datafix::fixes::HorseSaddle::HorseSaddle;
use crate::net::minecraft::util::datafix::fixes::HorseSplit::HorseSplit;
use crate::net::minecraft::util::datafix::fixes::ItemIntIDToString::ItemIntIDToString;
use crate::net::minecraft::util::datafix::fixes::MinecartEntityTypes::MinecartEntityTypes;
use crate::net::minecraft::util::datafix::fixes::OptionsLowerCaseLanguage::OptionsLowerCaseLanguage;
use crate::net::minecraft::util::datafix::fixes::PaintingDirection::PaintingDirection;
use crate::net::minecraft::util::datafix::fixes::PotionItems::PotionItems;
use crate::net::minecraft::util::datafix::fixes::PotionWater::PotionWater;
use crate::net::minecraft::util::datafix::fixes::RedundantChanceTags::RedundantChanceTags;
use crate::net::minecraft::util::datafix::fixes::RidingToPassengers::RidingToPassengers;
use crate::net::minecraft::util::datafix::fixes::ShulkerBoxEntityColor::ShulkerBoxEntityColor;
use crate::net::minecraft::util::datafix::fixes::ShulkerBoxItemColor::ShulkerBoxItemColor;
use crate::net::minecraft::util::datafix::fixes::ShulkerBoxTileColor::ShulkerBoxTileColor;
use crate::net::minecraft::util::datafix::fixes::SignStrictJSON::SignStrictJSON;
use crate::net::minecraft::util::datafix::fixes::SkeletonSplit::SkeletonSplit;
use crate::net::minecraft::util::datafix::fixes::SpawnEggNames::SpawnEggNames;
use crate::net::minecraft::util::datafix::fixes::SpawnerEntityTypes::SpawnerEntityTypes;
use crate::net::minecraft::util::datafix::fixes::StringToUUID::StringToUUID;
use crate::net::minecraft::util::datafix::fixes::TileEntityId::TileEntityId;
use crate::net::minecraft::util::datafix::fixes::TotemItemRename::TotemItemRename;
use crate::net::minecraft::util::datafix::fixes::ZombieProfToType::ZombieProfToType;
use crate::net::minecraft::util::datafix::fixes::ZombieSplit::ZombieSplit;
use crate::net::minecraft::util::datafix::walkers::ItemStackData::ItemStackData;
use crate::net::minecraft::util::datafix::walkers::ItemStackDataLists::ItemStackDataLists;
use crate::net::minecraft::util::datafix::walkers::VanillaDataWalkers::{
    MinecartCommandBlockDataWalker, MinecartSpawnerDataWalker, MobSpawnerDataWalker,
    PlayerInventoryDataWalker, PlayerRootVehicleDataWalker, StructureTemplateDataWalker,
    VillagerTradeDataWalker,
};
use crate::net::minecraft::util::datafix::DataFixer::DataFixer;
use crate::net::minecraft::util::datafix::FixTypes::FixTypes;
use crate::net::minecraft::util::datafix::IDataFixer::IDataFixer;
use crate::net::minecraft::world::chunk::storage::AnvilChunkLoader::AnvilChunkLoader;
use crate::net::minecraft::world::storage::WorldInfo::WorldInfo;

/// MCP 1.12.2 `DataFixesManager` composition root.
///
/// Batch 124 wires the source fix engine to the server persistence path and
/// ports the full vanilla 1.12.2 fix registration list plus the data-only
/// nested ItemStack/entity/tile walkers required by those source classes.
/// Runtime construction remains separate: this layer rewrites NBT only and
/// never fabricates an Entity or TileEntity instance.
pub struct DataFixesManager;
impl DataFixesManager {
    pub fn createFixer() -> DataFixer {
        let mut fixer = DataFixer::new(1343);
        // MCP DataFixesManager#createFixer registration order. Java delegates
        // most walkers to concrete classes; Rust keeps the same ordering while
        // data-only walkers for not-yet-ported server subclasses are registered
        // through the source registry table below.
        WorldInfo::registerFixes(&mut fixer);
        fixer.registerWalker(FixTypes::Player, Arc::new(PlayerRootVehicleDataWalker));
        fixer.registerWalker(FixTypes::Player, Arc::new(PlayerInventoryDataWalker));
        AnvilChunkLoader::registerFixes(&mut fixer);
        ItemStack::registerFixes(&mut fixer);
        fixer.registerWalker(FixTypes::Structure, Arc::new(StructureTemplateDataWalker));
        Entity::registerFixes(&mut fixer);
        Self::registerEntityAndTileWalkers(&mut fixer);
        Self::registerFixes(&mut fixer);
        fixer
    }

    fn registerEntityAndTileWalkers(fixer: &mut DataFixer) {
        // Exact set of concrete EntityLiving subclasses whose 1.12.2
        // registerFixes* methods are invoked by DataFixesManager#createFixer.
        // All delegate to EntityLiving#registerFixesMob and therefore recurse
        // ItemStack fixes through ArmorItems + HandItems.
        const LIVING: &[&str] = &[
            "bat",
            "blaze",
            "cave_spider",
            "chicken",
            "cow",
            "creeper",
            "donkey",
            "elder_guardian",
            "ender_dragon",
            "enderman",
            "endermite",
            "evocation_illager",
            "ghast",
            "giant",
            "guardian",
            "horse",
            "husk",
            "magma_cube",
            "mule",
            "mooshroom",
            "ocelot",
            "pig",
            "zombie_pigman",
            "rabbit",
            "sheep",
            "shulker",
            "silverfish",
            "skeleton",
            "skeleton_horse",
            "slime",
            "snowman",
            "spider",
            "squid",
            "stray",
            "vex",
            "villager",
            "villager_golem",
            "vindication_illager",
            "witch",
            "wither",
            "wither_skeleton",
            "wolf",
            "zombie",
            "zombie_horse",
            "zombie_villager",
        ];
        for id in LIVING {
            fixer.registerWalker(
                FixTypes::Entity,
                Arc::new(ItemStackDataLists::new(id, &["ArmorItems", "HandItems"])),
            );
        }

        // Source class-specific nested ItemStack walkers.
        fixer.registerWalker(
            FixTypes::Entity,
            Arc::new(ItemStackDataLists::new(
                "armor_stand",
                &["ArmorItems", "HandItems"],
            )),
        );
        fixer.registerWalker(
            FixTypes::Entity,
            Arc::new(ItemStackData::new("item", &["Item"])),
        );
        fixer.registerWalker(
            FixTypes::Entity,
            Arc::new(ItemStackData::new("item_frame", &["Item"])),
        );
        fixer.registerWalker(
            FixTypes::Entity,
            Arc::new(ItemStackData::new("fireworks_rocket", &["FireworksItem"])),
        );
        fixer.registerWalker(
            FixTypes::Entity,
            Arc::new(ItemStackData::new("potion", &["Potion"])),
        );
        fixer.registerWalker(
            FixTypes::Entity,
            Arc::new(ItemStackDataLists::new("villager", &["Inventory"])),
        );
        fixer.registerWalker(FixTypes::Entity, Arc::new(VillagerTradeDataWalker));

        for id in ["horse", "donkey", "mule", "skeleton_horse", "zombie_horse"] {
            fixer.registerWalker(
                FixTypes::Entity,
                Arc::new(ItemStackData::new(id, &["SaddleItem"])),
            );
        }
        fixer.registerWalker(
            FixTypes::Entity,
            Arc::new(ItemStackData::new("horse", &["ArmorItem"])),
        );
        for id in ["donkey", "mule"] {
            fixer.registerWalker(
                FixTypes::Entity,
                Arc::new(ItemStackDataLists::new(id, &["Items"])),
            );
        }
        for id in ["chest_minecart", "hopper_minecart"] {
            fixer.registerWalker(
                FixTypes::Entity,
                Arc::new(ItemStackDataLists::new(id, &["Items"])),
            );
        }
        fixer.registerWalker(FixTypes::Entity, Arc::new(MinecartSpawnerDataWalker));
        fixer.registerWalker(FixTypes::Entity, Arc::new(MinecartCommandBlockDataWalker));

        // Exact TileEntity inventory walkers called by createFixer.
        for id in [
            "furnace",
            "chest",
            "dispenser",
            "dropper",
            "brewing_stand",
            "hopper",
            "shulker_box",
        ] {
            fixer.registerWalker(
                FixTypes::BlockEntity,
                Arc::new(ItemStackDataLists::new(id, &["Items"])),
            );
        }
        fixer.registerWalker(
            FixTypes::BlockEntity,
            Arc::new(ItemStackData::new("jukebox", &["RecordItem"])),
        );
        fixer.registerWalker(FixTypes::BlockEntity, Arc::new(MobSpawnerDataWalker));
    }

    fn registerFixes(fixer: &mut DataFixer) {
        // Preserve MCP registration order. DataFixer additionally sorts by
        // fix version exactly as the source engine does.
        fixer.registerFix(FixTypes::Entity, Arc::new(EntityArmorAndHeld));
        fixer.registerFix(FixTypes::BlockEntity, Arc::new(SignStrictJSON));
        fixer.registerFix(FixTypes::ItemInstance, Arc::new(ItemIntIDToString));
        fixer.registerFix(FixTypes::ItemInstance, Arc::new(PotionItems));
        fixer.registerFix(FixTypes::ItemInstance, Arc::new(SpawnEggNames));
        fixer.registerFix(FixTypes::Entity, Arc::new(MinecartEntityTypes));
        fixer.registerFix(FixTypes::BlockEntity, Arc::new(SpawnerEntityTypes));
        fixer.registerFix(FixTypes::Entity, Arc::new(StringToUUID));
        fixer.registerFix(FixTypes::Entity, Arc::new(EntityHealth));
        fixer.registerFix(FixTypes::Entity, Arc::new(HorseSaddle));
        fixer.registerFix(FixTypes::Entity, Arc::new(PaintingDirection));
        fixer.registerFix(FixTypes::Entity, Arc::new(RedundantChanceTags));
        fixer.registerFix(FixTypes::Entity, Arc::new(RidingToPassengers));
        fixer.registerFix(FixTypes::Entity, Arc::new(ArmorStandSilent));
        fixer.registerFix(FixTypes::ItemInstance, Arc::new(BookPagesStrictJSON));
        fixer.registerFix(FixTypes::ItemInstance, Arc::new(CookedFishIDTypo));
        fixer.registerFix(FixTypes::Entity, Arc::new(ZombieProfToType));
        fixer.registerFix(FixTypes::Options, Arc::new(ForceVBOOn));
        fixer.registerFix(FixTypes::Entity, Arc::new(ElderGuardianSplit));
        fixer.registerFix(FixTypes::Entity, Arc::new(SkeletonSplit));
        fixer.registerFix(FixTypes::Entity, Arc::new(ZombieSplit));
        fixer.registerFix(FixTypes::Entity, Arc::new(HorseSplit));
        fixer.registerFix(FixTypes::BlockEntity, Arc::new(TileEntityId));
        fixer.registerFix(FixTypes::Entity, Arc::new(EntityId));
        fixer.registerFix(FixTypes::ItemInstance, Arc::new(BannerItemColor));
        fixer.registerFix(FixTypes::ItemInstance, Arc::new(PotionWater));
        fixer.registerFix(FixTypes::Entity, Arc::new(ShulkerBoxEntityColor));
        fixer.registerFix(FixTypes::ItemInstance, Arc::new(ShulkerBoxItemColor));
        fixer.registerFix(FixTypes::BlockEntity, Arc::new(ShulkerBoxTileColor));
        fixer.registerFix(FixTypes::Options, Arc::new(OptionsLowerCaseLanguage));
        fixer.registerFix(FixTypes::ItemInstance, Arc::new(TotemItemRename));
        fixer.registerFix(FixTypes::Chunk, Arc::new(AddBedTileEntity));
        fixer.registerFix(FixTypes::ItemInstance, Arc::new(BedItemColor));
    }

    /// MCP `processItemStack` helper.
    pub fn processItemStack(
        fixer: &dyn IDataFixer,
        mut compound: NBTTagCompound,
        version: i32,
        key: &str,
    ) -> NBTTagCompound {
        if compound.hasKeyWithType(key, TAG_COMPOUND) {
            let fixed = fixer.processVersioned(
                FixTypes::ItemInstance,
                compound.getCompoundTag(key),
                version,
            );
            compound.setCompoundTag(key, fixed);
        }
        compound
    }

    /// MCP `processInventory` helper.
    pub fn processInventory(
        fixer: &dyn IDataFixer,
        mut compound: NBTTagCompound,
        version: i32,
        key: &str,
    ) -> NBTTagCompound {
        if compound.hasKeyWithType(key, TAG_LIST) {
            let mut list = compound.getTagList(key, TAG_COMPOUND);
            for index in 0..list.tagCount() {
                let fixed = fixer.processVersioned(
                    FixTypes::ItemInstance,
                    list.getCompoundTagAt(index),
                    version,
                );
                list.set(index, NBTBase::Compound(fixed));
            }
            compound.setTagList(key, list);
        }
        compound
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::nbt::NBTBase::NBTBase;
    use crate::net::minecraft::nbt::NBTTagList::NBTTagList;

    #[test]
    fn legacy_entity_id_and_riding_chain_are_fixed_before_chunk_object_load() {
        let fixer = DataFixesManager::createFixer();
        let mut rider = NBTTagCompound::new();
        rider.setString("id", "Pig");
        let mut vehicle = NBTTagCompound::new();
        vehicle.setString("id", "Boat");
        rider.setCompoundTag("Riding", vehicle);
        let fixed = fixer.processVersioned(FixTypes::Entity, rider, 100);
        assert_eq!(fixed.getString("id"), "minecraft:boat");
        let passengers = fixed.getTagList("Passengers", TAG_COMPOUND);
        assert_eq!(passengers.tagCount(), 1);
        assert_eq!(
            passengers.getCompoundTagAt(0).getString("id"),
            "minecraft:pig"
        );
    }

    #[test]
    fn chunk_bed_fix_adds_tile_entity_at_source_block_coordinates() {
        let fixer = DataFixesManager::createFixer();
        let mut root = NBTTagCompound::new();
        let mut level = NBTTagCompound::new();
        level.setInteger("xPos", 2);
        level.setInteger("zPos", -3);
        level.setTagList("TileEntities", NBTTagList::new());
        let mut section = NBTTagCompound::new();
        section.setByte("Y", 4);
        let mut blocks = vec![0u8; 4096];
        let index = 5 | (7 << 8) | (9 << 4);
        blocks[index] = 26; // 26 << 4 == 416, source bed block test.
        section.setByteArray("Blocks", blocks);
        let mut sections = NBTTagList::new();
        sections.appendTag(NBTBase::Compound(section));
        level.setTagList("Sections", sections);
        root.setCompoundTag("Level", level);
        let fixed = fixer.processVersioned(FixTypes::Chunk, root, 1000);
        let beds = fixed
            .getCompoundTag("Level")
            .getTagList("TileEntities", TAG_COMPOUND);
        assert_eq!(beds.tagCount(), 1);
        let bed = beds.getCompoundTagAt(0);
        assert_eq!(
            (
                bed.getInteger("x"),
                bed.getInteger("y"),
                bed.getInteger("z")
            ),
            (37, 71, -39)
        );
    }
}
