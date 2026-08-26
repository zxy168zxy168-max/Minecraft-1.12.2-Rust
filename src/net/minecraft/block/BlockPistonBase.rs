use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

/// Collision-state subset of MCP 1.12.2 `BlockPistonBase`.
pub struct BlockPistonBase;

impl BlockPistonBase {
    pub const fn isPistonBase(state: IBlockState) -> bool {
        matches!(state.getBlockId(), 29 | 33)
    }

    pub fn getBoundingBox(state: IBlockState) -> AxisAlignedBB {
        let meta = state.getMetadata();
        if meta & 8 == 0 {
            return AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        }
        match EnumFacing::getFront(meta & 7) {
            EnumFacing::Down => AxisAlignedBB::new(0.0, 0.25, 0.0, 1.0, 1.0, 1.0),
            EnumFacing::Up => AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 0.75, 1.0),
            EnumFacing::North => AxisAlignedBB::new(0.0, 0.0, 0.25, 1.0, 1.0, 1.0),
            EnumFacing::South => AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 0.75),
            EnumFacing::West => AxisAlignedBB::new(0.25, 0.0, 0.0, 1.0, 1.0, 1.0),
            EnumFacing::East => AxisAlignedBB::new(0.0, 0.0, 0.0, 0.75, 1.0, 1.0),
        }
    }
}
