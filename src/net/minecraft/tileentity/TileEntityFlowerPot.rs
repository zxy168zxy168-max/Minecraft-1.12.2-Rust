use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// MCP 1.12.2 `TileEntityFlowerPot` network state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileEntityFlowerPot {
    pub pos: BlockPos,
    itemId: i32,
    itemData: i32,
}

impl TileEntityFlowerPot {
    pub fn new(pos: BlockPos) -> Self {
        Self {
            pos,
            itemId: -1,
            itemData: 0,
        }
    }

    /// MCP `BlockFlowerPot#createNewTileEntity(World, meta)`. The legacy
    /// metadata is only a bootstrap value; later type-5 tile-entity updates
    /// replace it with the authoritative item/data pair.
    pub fn fromLegacyMetadata(pos: BlockPos, meta: i32) -> Self {
        let (itemId, itemData) = match meta & 15 {
            1 => (38, 0),  // poppy
            2 => (37, 0),  // dandelion
            3 => (6, 0),   // oak sapling
            4 => (6, 1),   // spruce sapling
            5 => (6, 2),   // birch sapling
            6 => (6, 3),   // jungle sapling
            7 => (40, 0),  // red mushroom
            8 => (39, 0),  // brown mushroom
            9 => (81, 0),  // cactus
            10 => (32, 0), // dead bush
            11 => (31, 2), // fern
            12 => (6, 4),  // acacia sapling
            13 => (6, 5),  // dark-oak sapling
            _ => (-1, 0),
        };
        Self {
            pos,
            itemId,
            itemData,
        }
    }

    pub fn fromNbt(tag: &NBTTagCompound) -> Option<Self> {
        let id = tag.getString("id");
        if !id.is_empty() && id != "minecraft:flower_pot" && id != "FlowerPot" {
            return None;
        }
        let itemId = if tag.hasKeyWithType("Item", crate::net::minecraft::nbt::NBTBase::TAG_STRING)
        {
            item_name_to_id(&tag.getString("Item"))
        } else {
            tag.getInteger("Item")
        };
        Some(Self {
            pos: BlockPos::new(
                tag.getInteger("x"),
                tag.getInteger("y"),
                tag.getInteger("z"),
            ),
            itemId,
            itemData: tag.getInteger("Data"),
        })
    }

    pub const fn itemId(&self) -> i32 {
        self.itemId
    }
    pub const fn itemData(&self) -> i32 {
        self.itemData
    }
}

fn item_name_to_id(name: &str) -> i32 {
    match name.strip_prefix("minecraft:").unwrap_or(name) {
        "sapling" => 6,
        "tallgrass" => 31,
        "deadbush" => 32,
        "yellow_flower" => 37,
        "red_flower" => 38,
        "brown_mushroom" => 39,
        "red_mushroom" => 40,
        "cactus" => 81,
        "air" | "" => -1,
        _ => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::item_name_to_id;
    #[test]
    fn registry_names_resolve_to_1122_item_ids() {
        assert_eq!(item_name_to_id("minecraft:red_flower"), 38);
        assert_eq!(item_name_to_id("cactus"), 81);
    }
}
