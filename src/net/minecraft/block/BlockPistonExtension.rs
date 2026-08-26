use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

/// Collision-state subset of MCP 1.12.2 `BlockPistonExtension`.
pub struct BlockPistonExtension;

impl BlockPistonExtension {
    pub const fn isPistonHead(state: IBlockState) -> bool {
        state.getBlockId() == 34
    }

    /// MCP `BlockPistonExtension#getBoundingBox`: the selection/model bound is
    /// the piston head plate only. The arm is added separately for entity
    /// collision by `addCollisionBoxToList`.
    pub fn getBoundingBox(state: IBlockState) -> AxisAlignedBB {
        match EnumFacing::getFront(state.getMetadata() & 7) {
            EnumFacing::Down => AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 0.25, 1.0),
            EnumFacing::Up => AxisAlignedBB::new(0.0, 0.75, 0.0, 1.0, 1.0, 1.0),
            EnumFacing::North => AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 0.25),
            EnumFacing::South => AxisAlignedBB::new(0.0, 0.0, 0.75, 1.0, 1.0, 1.0),
            EnumFacing::West => AxisAlignedBB::new(0.0, 0.0, 0.0, 0.25, 1.0, 1.0),
            EnumFacing::East => AxisAlignedBB::new(0.75, 0.0, 0.0, 1.0, 1.0, 1.0),
        }
    }

    pub fn collisionBoxes(state: IBlockState, short: bool) -> Vec<AxisAlignedBB> {
        Self::collisionBoxesForFacing(EnumFacing::getFront(state.getMetadata() & 7), short)
    }

    pub fn collisionBoxesForFacing(facing: EnumFacing, short: bool) -> Vec<AxisAlignedBB> {
        let head = match facing {
            EnumFacing::Down => AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 0.25, 1.0),
            EnumFacing::Up => AxisAlignedBB::new(0.0, 0.75, 0.0, 1.0, 1.0, 1.0),
            EnumFacing::North => AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 0.25),
            EnumFacing::South => AxisAlignedBB::new(0.0, 0.0, 0.75, 1.0, 1.0, 1.0),
            EnumFacing::West => AxisAlignedBB::new(0.0, 0.0, 0.0, 0.25, 1.0, 1.0),
            EnumFacing::East => AxisAlignedBB::new(0.75, 0.0, 0.0, 1.0, 1.0, 1.0),
        };
        let arm = match (facing, short) {
            (EnumFacing::Down, false) => AxisAlignedBB::new(0.375, 0.25, 0.375, 0.625, 1.25, 0.625),
            (EnumFacing::Down, true) => AxisAlignedBB::new(0.375, 0.25, 0.375, 0.625, 1.0, 0.625),
            (EnumFacing::Up, false) => AxisAlignedBB::new(0.375, -0.25, 0.375, 0.625, 0.75, 0.625),
            (EnumFacing::Up, true) => AxisAlignedBB::new(0.375, 0.0, 0.375, 0.625, 0.75, 0.625),
            (EnumFacing::North, false) => {
                AxisAlignedBB::new(0.375, 0.375, 0.25, 0.625, 0.625, 1.25)
            }
            (EnumFacing::North, true) => AxisAlignedBB::new(0.375, 0.375, 0.25, 0.625, 0.625, 1.0),
            (EnumFacing::South, false) => {
                AxisAlignedBB::new(0.375, 0.375, -0.25, 0.625, 0.625, 0.75)
            }
            (EnumFacing::South, true) => AxisAlignedBB::new(0.375, 0.375, 0.0, 0.625, 0.625, 0.75),
            (EnumFacing::West, false) => AxisAlignedBB::new(0.25, 0.375, 0.375, 1.25, 0.625, 0.625),
            (EnumFacing::West, true) => AxisAlignedBB::new(0.25, 0.375, 0.375, 1.0, 0.625, 0.625),
            (EnumFacing::East, false) => {
                AxisAlignedBB::new(-0.25, 0.375, 0.375, 0.75, 0.625, 0.625)
            }
            (EnumFacing::East, true) => AxisAlignedBB::new(0.0, 0.375, 0.375, 0.75, 0.625, 0.625),
        };
        vec![head, arm]
    }
}
