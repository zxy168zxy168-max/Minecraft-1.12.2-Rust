use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

/// MCP 1.12.2 `TileEntityId` (DataVersion 704).
pub struct TileEntityId;
impl IFixableData for TileEntityId {
    fn getFixVersion(&self) -> i32 { 704 }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        let replacement = match compound.getString("id").as_str() {
            "Airportal" => Some("minecraft:end_portal"),
            "Banner" => Some("minecraft:banner"),
            "Beacon" => Some("minecraft:beacon"),
            "Cauldron" => Some("minecraft:brewing_stand"),
            "Chest" => Some("minecraft:chest"),
            "Comparator" => Some("minecraft:comparator"),
            "Control" => Some("minecraft:command_block"),
            "DLDetector" => Some("minecraft:daylight_detector"),
            "Dropper" => Some("minecraft:dropper"),
            "EnchantTable" => Some("minecraft:enchanting_table"),
            "EndGateway" => Some("minecraft:end_gateway"),
            "EnderChest" => Some("minecraft:ender_chest"),
            "FlowerPot" => Some("minecraft:flower_pot"),
            "Furnace" => Some("minecraft:furnace"),
            "Hopper" => Some("minecraft:hopper"),
            "MobSpawner" => Some("minecraft:mob_spawner"),
            "Music" => Some("minecraft:noteblock"),
            "Piston" => Some("minecraft:piston"),
            "RecordPlayer" => Some("minecraft:jukebox"),
            "Sign" => Some("minecraft:sign"),
            "Skull" => Some("minecraft:skull"),
            "Structure" => Some("minecraft:structure_block"),
            "Trap" => Some("minecraft:dispenser"),
            _ => None,
        };
        if let Some(id) = replacement { compound.setString("id", id); }
        compound
    }
}
