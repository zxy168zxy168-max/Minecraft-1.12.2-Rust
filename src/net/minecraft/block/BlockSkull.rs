use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

/// Exact block-state and outline contract of MCP 1.12.2 `BlockSkull`.
/// The visible head is rendered by `TileEntitySkullRenderer`; the block itself
/// uses the same facing-dependent box for selection and collision, matching
/// Block#getCollisionBoundingBox delegating to the state bounding box.
pub struct BlockSkull;

impl BlockSkull {
    pub const BLOCK_ID: i32 = 144;
    pub const DEFAULT_AABB: AxisAlignedBB = AxisAlignedBB {
        min_x: 0.25,
        min_y: 0.0,
        min_z: 0.25,
        max_x: 0.75,
        max_y: 0.5,
        max_z: 0.75,
    };
    pub const NORTH_AABB: AxisAlignedBB = AxisAlignedBB {
        min_x: 0.25,
        min_y: 0.25,
        min_z: 0.5,
        max_x: 0.75,
        max_y: 0.75,
        max_z: 1.0,
    };
    pub const SOUTH_AABB: AxisAlignedBB = AxisAlignedBB {
        min_x: 0.25,
        min_y: 0.25,
        min_z: 0.0,
        max_x: 0.75,
        max_y: 0.75,
        max_z: 0.5,
    };
    pub const WEST_AABB: AxisAlignedBB = AxisAlignedBB {
        min_x: 0.5,
        min_y: 0.25,
        min_z: 0.25,
        max_x: 1.0,
        max_y: 0.75,
        max_z: 0.75,
    };
    pub const EAST_AABB: AxisAlignedBB = AxisAlignedBB {
        min_x: 0.0,
        min_y: 0.25,
        min_z: 0.25,
        max_x: 0.5,
        max_y: 0.75,
        max_z: 0.75,
    };

    pub const fn isBlockSkull(state: IBlockState) -> bool {
        state.getBlockId() == Self::BLOCK_ID
    }

    /// `BlockSkull#getStateFromMeta` uses `EnumFacing.getFront(meta & 7)`.
    pub const fn getFacing(state: IBlockState) -> EnumFacing {
        match (state.getMetadata() & 7) % 6 {
            0 => EnumFacing::Down,
            1 => EnumFacing::Up,
            2 => EnumFacing::North,
            3 => EnumFacing::South,
            4 => EnumFacing::West,
            _ => EnumFacing::East,
        }
    }

    pub const fn stateForFacing(facing: EnumFacing) -> IBlockState {
        IBlockState::fromGlobalStateId((Self::BLOCK_ID << 4) | facing.index())
    }

    pub const fn getBoundingBox(state: IBlockState) -> AxisAlignedBB {
        match Self::getFacing(state) {
            EnumFacing::North => Self::NORTH_AABB,
            EnumFacing::South => Self::SOUTH_AABB,
            EnumFacing::West => Self::WEST_AABB,
            EnumFacing::East => Self::EAST_AABB,
            _ => Self::DEFAULT_AABB,
        }
    }

    pub fn getCollisionBoxes(state: IBlockState) -> Vec<AxisAlignedBB> {
        vec![Self::getBoundingBox(state)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_and_wall_bounds_match_mcp_constants() {
        assert_eq!(
            BlockSkull::getBoundingBox(BlockSkull::stateForFacing(EnumFacing::Up)),
            BlockSkull::DEFAULT_AABB
        );
        assert_eq!(
            BlockSkull::getBoundingBox(BlockSkull::stateForFacing(EnumFacing::North)),
            BlockSkull::NORTH_AABB
        );
        assert_eq!(
            BlockSkull::getBoundingBox(BlockSkull::stateForFacing(EnumFacing::East)),
            BlockSkull::EAST_AABB
        );
        assert_eq!(
            BlockSkull::getCollisionBoxes(BlockSkull::stateForFacing(EnumFacing::Up)),
            vec![BlockSkull::DEFAULT_AABB],
        );
    }
}
