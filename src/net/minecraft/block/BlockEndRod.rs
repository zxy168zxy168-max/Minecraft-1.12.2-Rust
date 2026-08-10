use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::EnumFacing::{Axis, EnumFacing};
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;

pub const fn isBlockEndRod(state: IBlockState) -> bool { state.getBlockId() == 198 }

/// `BlockEndRod#getStateFromMeta`: `EnumFacing.getFront(meta)` uses index
/// order DOWN, UP, NORTH, SOUTH, WEST, EAST and wraps modulo six.
pub const fn facing(state: IBlockState) -> EnumFacing {
    match (state.getMetadata() & 7) % 6 {
        0 => EnumFacing::Down,
        1 => EnumFacing::Up,
        2 => EnumFacing::North,
        3 => EnumFacing::South,
        4 => EnumFacing::West,
        _ => EnumFacing::East,
    }
}

/// MCP `BlockEndRod#onBlockPlaced`: normally faces the clicked face; if the
/// supporting end rod already faces that same direction, the new rod reverses.
pub fn onBlockPlacedState<A: crate::net::minecraft::world::IBlockAccess::IBlockAccess>(world:&A,pos:crate::net::minecraft::util::math::BlockPos::BlockPos,clickedFace:EnumFacing)->IBlockState{
    let support=world.getBlockState(pos.offset(clickedFace.opposite(),1));
    let resolved=if isBlockEndRod(support)&&facing(support)==clickedFace{clickedFace.opposite()}else{clickedFace};
    IBlockState::fromGlobalStateId((198<<4)|(resolved.index()&7))
}

/// Exact local-space result of `BlockEndRod#getBoundingBox`.
pub fn getBoundingBox(state: IBlockState) -> AxisAlignedBB {
    match facing(state).axis() {
        Axis::X => AxisAlignedBB::new(0.0, 0.375, 0.375, 1.0, 0.625, 0.625),
        Axis::Z => AxisAlignedBB::new(0.375, 0.375, 0.0, 0.625, 0.625, 1.0),
        Axis::Y => AxisAlignedBB::new(0.375, 0.0, 0.375, 0.625, 1.0, 0.625),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_axis_uses_the_source_aabb() {
        let vertical = getBoundingBox(IBlockState::fromGlobalStateId((198 << 4) | 1));
        let northSouth = getBoundingBox(IBlockState::fromGlobalStateId((198 << 4) | 2));
        let eastWest = getBoundingBox(IBlockState::fromGlobalStateId((198 << 4) | 5));
        assert_eq!((vertical.min_y, vertical.max_y), (0.0, 1.0));
        assert_eq!((northSouth.min_z, northSouth.max_z), (0.0, 1.0));
        assert_eq!((eastWest.min_x, eastWest.max_x), (0.0, 1.0));
    }
}
