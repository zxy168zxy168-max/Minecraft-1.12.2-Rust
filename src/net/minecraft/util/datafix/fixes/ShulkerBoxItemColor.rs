use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

pub struct ShulkerBoxItemColor;
impl ShulkerBoxItemColor {
    pub const COLORS: [&'static str; 16] = [
        "minecraft:white_shulker_box",
        "minecraft:orange_shulker_box",
        "minecraft:magenta_shulker_box",
        "minecraft:light_blue_shulker_box",
        "minecraft:yellow_shulker_box",
        "minecraft:lime_shulker_box",
        "minecraft:pink_shulker_box",
        "minecraft:gray_shulker_box",
        "minecraft:silver_shulker_box",
        "minecraft:cyan_shulker_box",
        "minecraft:purple_shulker_box",
        "minecraft:blue_shulker_box",
        "minecraft:brown_shulker_box",
        "minecraft:green_shulker_box",
        "minecraft:red_shulker_box",
        "minecraft:black_shulker_box",
    ];
}
impl IFixableData for ShulkerBoxItemColor {
    fn getFixVersion(&self) -> i32 {
        813
    }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        if compound.getString("id") != "minecraft:shulker_box"
            || !compound.hasKeyWithType("tag", 10)
        {
            return compound;
        }
        let mut tag = compound.getCompoundTag("tag");
        if !tag.hasKeyWithType("BlockEntityTag", 10) {
            return compound;
        }
        let mut block_entity = tag.getCompoundTag("BlockEntityTag");
        if block_entity.getTagList("Items", 10).hasNoTags() {
            block_entity.removeTag("Items");
        }
        let color = block_entity.getInteger("Color");
        block_entity.removeTag("Color");
        if block_entity.hasNoTags() {
            tag.removeTag("BlockEntityTag");
        } else {
            tag.setCompoundTag("BlockEntityTag", block_entity);
        }
        if tag.hasNoTags() {
            compound.removeTag("tag");
        } else {
            compound.setCompoundTag("tag", tag);
        }
        compound.setString("id", Self::COLORS[(color % 16) as usize]);
        compound
    }
}
