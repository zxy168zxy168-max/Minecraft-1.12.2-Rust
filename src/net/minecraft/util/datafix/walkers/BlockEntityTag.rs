use crate::net::minecraft::nbt::NBTBase::TAG_COMPOUND;
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::FixTypes::FixTypes;
use crate::net::minecraft::util::datafix::IDataFixer::IDataFixer;
use crate::net::minecraft::util::datafix::IDataWalker::IDataWalker;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// Exact MCP 1.12.2 `walkers.BlockEntityTag` item→block-entity identity tables.
pub struct BlockEntityTag;
impl BlockEntityTag {
    fn getBlockEntityID(blockID: i32, item: &str) -> Option<&'static str> {
        let item = ResourceLocation::parse(item).to_string();
        if blockID < 515 {
            match item.as_str() {
                "minecraft:furnace" => Some("Furnace"),
                "minecraft:lit_furnace" => Some("Furnace"),
                "minecraft:chest" => Some("Chest"),
                "minecraft:trapped_chest" => Some("Chest"),
                "minecraft:ender_chest" => Some("EnderChest"),
                "minecraft:jukebox" => Some("RecordPlayer"),
                "minecraft:dispenser" => Some("Trap"),
                "minecraft:dropper" => Some("Dropper"),
                "minecraft:sign" => Some("Sign"),
                "minecraft:mob_spawner" => Some("MobSpawner"),
                "minecraft:noteblock" => Some("Music"),
                "minecraft:brewing_stand" => Some("Cauldron"),
                "minecraft:enhanting_table" => Some("EnchantTable"),
                "minecraft:command_block" => Some("CommandBlock"),
                "minecraft:beacon" => Some("Beacon"),
                "minecraft:skull" => Some("Skull"),
                "minecraft:daylight_detector" => Some("DLDetector"),
                "minecraft:hopper" => Some("Hopper"),
                "minecraft:banner" => Some("Banner"),
                "minecraft:flower_pot" => Some("FlowerPot"),
                "minecraft:repeating_command_block" => Some("CommandBlock"),
                "minecraft:chain_command_block" => Some("CommandBlock"),
                "minecraft:standing_sign" => Some("Sign"),
                "minecraft:wall_sign" => Some("Sign"),
                "minecraft:piston_head" => Some("Piston"),
                "minecraft:daylight_detector_inverted" => Some("DLDetector"),
                "minecraft:unpowered_comparator" => Some("Comparator"),
                "minecraft:powered_comparator" => Some("Comparator"),
                "minecraft:wall_banner" => Some("Banner"),
                "minecraft:standing_banner" => Some("Banner"),
                "minecraft:structure_block" => Some("Structure"),
                "minecraft:end_portal" => Some("Airportal"),
                "minecraft:end_gateway" => Some("EndGateway"),
                "minecraft:shield" => Some("Shield"),
                _ => None,
            }
        } else {
            match item.as_str() {
                "minecraft:furnace" => Some("minecraft:furnace"),
                "minecraft:lit_furnace" => Some("minecraft:furnace"),
                "minecraft:chest" => Some("minecraft:chest"),
                "minecraft:trapped_chest" => Some("minecraft:chest"),
                "minecraft:ender_chest" => Some("minecraft:enderchest"),
                "minecraft:jukebox" => Some("minecraft:jukebox"),
                "minecraft:dispenser" => Some("minecraft:dispenser"),
                "minecraft:dropper" => Some("minecraft:dropper"),
                "minecraft:sign" => Some("minecraft:sign"),
                "minecraft:mob_spawner" => Some("minecraft:mob_spawner"),
                "minecraft:noteblock" => Some("minecraft:noteblock"),
                "minecraft:brewing_stand" => Some("minecraft:brewing_stand"),
                "minecraft:enhanting_table" => Some("minecraft:enchanting_table"),
                "minecraft:command_block" => Some("minecraft:command_block"),
                "minecraft:beacon" => Some("minecraft:beacon"),
                "minecraft:skull" => Some("minecraft:skull"),
                "minecraft:daylight_detector" => Some("minecraft:daylight_detector"),
                "minecraft:hopper" => Some("minecraft:hopper"),
                "minecraft:banner" => Some("minecraft:banner"),
                "minecraft:flower_pot" => Some("minecraft:flower_pot"),
                "minecraft:repeating_command_block" => Some("minecraft:command_block"),
                "minecraft:chain_command_block" => Some("minecraft:command_block"),
                "minecraft:shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:white_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:orange_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:magenta_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:light_blue_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:yellow_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:lime_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:pink_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:gray_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:silver_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:cyan_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:purple_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:blue_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:brown_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:green_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:red_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:black_shulker_box" => Some("minecraft:shulker_box"),
                "minecraft:bed" => Some("minecraft:bed"),
                "minecraft:standing_sign" => Some("minecraft:sign"),
                "minecraft:wall_sign" => Some("minecraft:sign"),
                "minecraft:piston_head" => Some("minecraft:piston"),
                "minecraft:daylight_detector_inverted" => Some("minecraft:daylight_detector"),
                "minecraft:unpowered_comparator" => Some("minecraft:comparator"),
                "minecraft:powered_comparator" => Some("minecraft:comparator"),
                "minecraft:wall_banner" => Some("minecraft:banner"),
                "minecraft:standing_banner" => Some("minecraft:banner"),
                "minecraft:structure_block" => Some("minecraft:structure_block"),
                "minecraft:end_portal" => Some("minecraft:end_portal"),
                "minecraft:end_gateway" => Some("minecraft:end_gateway"),
                "minecraft:shield" => Some("minecraft:shield"),
                _ => None,
            }
        }
    }
}
impl IDataWalker for BlockEntityTag {
    fn process(
        &self,
        fixer: &dyn IDataFixer,
        mut compound: NBTTagCompound,
        versionIn: i32,
    ) -> NBTTagCompound {
        if !compound.hasKeyWithType("tag", TAG_COMPOUND) {
            return compound;
        }
        let mut tag = compound.getCompoundTag("tag");
        if tag.hasKeyWithType("BlockEntityTag", TAG_COMPOUND) {
            let mut blockEntity = tag.getCompoundTag("BlockEntityTag");
            let item = compound.getString("id");
            let mut removeId = false;
            if let Some(id) = Self::getBlockEntityID(versionIn, &item) {
                removeId = !blockEntity.hasKey("id");
                blockEntity.setString("id", id);
            } else {
                log::warn!("Unable to resolve BlockEntity for ItemInstance: {}", item);
            }
            blockEntity = fixer.processVersioned(FixTypes::BlockEntity, blockEntity, versionIn);
            if removeId {
                blockEntity.removeTag("id");
            }
            tag.setCompoundTag("BlockEntityTag", blockEntity);
            compound.setCompoundTag("tag", tag);
        }
        compound
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_tables_keep_pre_and_post_515_ids() {
        assert_eq!(
            BlockEntityTag::getBlockEntityID(500, "furnace"),
            Some("Furnace")
        );
        assert_eq!(
            BlockEntityTag::getBlockEntityID(515, "furnace"),
            Some("minecraft:furnace")
        );
        assert_eq!(
            BlockEntityTag::getBlockEntityID(515, "bed"),
            Some("minecraft:bed")
        );
    }
}
