use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

/// MCP 1.12.2 `TileEntityEndPortal`, with `TileEntityEndGateway` folded in
/// as an `is_gateway` flag: the gateway subclasses the portal tile and only
/// overrides the render-face decision (plus server-side teleport logic that a
/// thin client never receives).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileEntityEndPortal {
    pub pos: BlockPos,
    pub is_gateway: bool,
}

impl TileEntityEndPortal {
    pub const fn new(pos: BlockPos) -> Self {
        Self {
            pos,
            is_gateway: false,
        }
    }

    pub const fn new_gateway(pos: BlockPos) -> Self {
        Self {
            pos,
            is_gateway: true,
        }
    }

    pub fn fromNbt(tag: &NBTTagCompound) -> Option<Self> {
        let id = tag.getString("id");
        let gateway = id == "minecraft:end_gateway";
        if !id.is_empty() && id != "minecraft:end_portal" && id != "Airportal" && !gateway {
            return None;
        }
        Some(Self {
            pos: BlockPos::new(
                tag.getInteger("x"),
                tag.getInteger("y"),
                tag.getInteger("z"),
            ),
            is_gateway: gateway,
        })
    }

    /// Vanilla `TileEntityEndPortal#shouldRenderFace` renders only the top
    /// face. `TileEntityEndGateway` overrides it with the block state's
    /// `shouldSideBeRendered`, and because the gateway block is neither a
    /// full cube nor opaque every side is reported.
    pub const fn shouldRenderFace(&self, facing: EnumFacing) -> bool {
        self.is_gateway || matches!(facing, EnumFacing::Up)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_end_portal_only_renders_up() {
        let portal = TileEntityEndPortal::new(BlockPos::new(0, 0, 0));
        assert!(portal.shouldRenderFace(EnumFacing::Up));
        assert!(!portal.shouldRenderFace(EnumFacing::Down));
        assert!(!portal.shouldRenderFace(EnumFacing::North));
    }

    #[test]
    fn gateway_renders_every_face() {
        let gateway = TileEntityEndPortal::new_gateway(BlockPos::new(0, 0, 0));
        for facing in EnumFacing::VALUES {
            assert!(gateway.shouldRenderFace(facing));
        }
    }
}
