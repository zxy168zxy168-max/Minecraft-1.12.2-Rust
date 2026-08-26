use crate::net::minecraft::nbt::NBTBase::{NBTBase, TAG_COMPOUND};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

/// MCP 1.12.2 `AddBedTileEntity` (DataVersion 1125).
pub struct AddBedTileEntity;
impl IFixableData for AddBedTileEntity {
    fn getFixVersion(&self) -> i32 {
        1125
    }
    fn fixTagCompound(&self, mut root: NBTTagCompound) -> NBTTagCompound {
        let mut level = root.getCompoundTag("Level");
        let chunk_x = level.getInteger("xPos");
        let chunk_z = level.getInteger("zPos");
        let mut tile_entities = level.getTagList("TileEntities", TAG_COMPOUND);
        let sections = level.getTagList("Sections", TAG_COMPOUND);
        for section_index in 0..sections.tagCount() {
            let section = sections.getCompoundTagAt(section_index);
            let section_y = section.getByte("Y") as i32;
            let blocks = section.getByteArray("Blocks");
            for (index, block) in blocks.iter().enumerate() {
                // Source compares `416 == (Blocks[index] & 255) << 4`.
                if ((*block as i32) << 4) == 416 {
                    let local_x = (index & 15) as i32;
                    let local_y = ((index >> 8) & 15) as i32;
                    let local_z = ((index >> 4) & 15) as i32;
                    let mut bed = NBTTagCompound::new();
                    bed.setString("id", "bed");
                    bed.setInteger("x", local_x + (chunk_x << 4));
                    bed.setInteger("y", local_y + (section_y << 4));
                    bed.setInteger("z", local_z + (chunk_z << 4));
                    tile_entities.appendTag(NBTBase::Compound(bed));
                }
            }
        }
        // Java mutates the list by reference. Rust NBT getters return owned
        // clones, so explicitly put the modified list/Level back into root.
        level.setTagList("TileEntities", tile_entities);
        root.setCompoundTag("Level", level);
        root
    }
}
