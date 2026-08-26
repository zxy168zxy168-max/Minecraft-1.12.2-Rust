use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// Client data required by MCP 1.12.2 `TileEntityBedRenderer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileEntityBed {
    pub pos: BlockPos,
    colorMetadata: i32,
}

impl TileEntityBed {
    pub fn new(pos: BlockPos) -> Self {
        Self {
            pos,
            colorMetadata: 14,
        }
    }

    pub fn fromNbt(tag: &NBTTagCompound) -> Option<Self> {
        let id = tag.getString("id");
        if !id.is_empty() && id != "minecraft:bed" && id != "Bed" {
            return None;
        }
        Some(Self {
            pos: BlockPos::new(
                tag.getInteger("x"),
                tag.getInteger("y"),
                tag.getInteger("z"),
            ),
            colorMetadata: tag.getInteger("color").clamp(0, 15),
        })
    }

    pub const fn colorMetadata(&self) -> i32 {
        self.colorMetadata
    }
}
