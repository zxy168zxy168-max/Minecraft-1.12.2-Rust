use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

/// MCP 1.12.2 `TileEntityEndPortal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileEntityEndPortal {
    pub pos: BlockPos,
}

impl TileEntityEndPortal {
    pub const fn new(pos: BlockPos) -> Self {
        Self { pos }
    }

    pub fn fromNbt(tag: &NBTTagCompound) -> Option<Self> {
        let id = tag.getString("id");
        if !id.is_empty() && id != "minecraft:end_portal" && id != "Airportal" {
            return None;
        }
        Some(Self::new(BlockPos::new(
            tag.getInteger("x"),
            tag.getInteger("y"),
            tag.getInteger("z"),
        )))
    }

    /// Vanilla's base end-portal tile renders only its top face. End gateways
    /// subclass this tile and override the decision; they remain a separate
    /// renderer path.
    pub const fn shouldRenderFace(facing: EnumFacing) -> bool {
        matches!(facing, EnumFacing::Up)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_end_portal_only_renders_up() {
        assert!(TileEntityEndPortal::shouldRenderFace(EnumFacing::Up));
        assert!(!TileEntityEndPortal::shouldRenderFace(EnumFacing::Down));
        assert!(!TileEntityEndPortal::shouldRenderFace(EnumFacing::North));
    }
}
