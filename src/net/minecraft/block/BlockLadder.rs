use crate::net::minecraft::block::state::BlockFaceShape::BlockFaceShape;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::{Axis, EnumFacing};
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

pub const fn isBlockLadder(state: IBlockState) -> bool {
    state.getBlockId() == 65
}

pub const fn facing(state: IBlockState) -> EnumFacing {
    match state.getMetadata() % 6 {
        2 => EnumFacing::North,
        3 => EnumFacing::South,
        4 => EnumFacing::West,
        5 => EnumFacing::East,
        _ => EnumFacing::North,
    }
}

pub const fn metadataForFacing(facing: EnumFacing) -> i32 {
    match facing {
        EnumFacing::North => 2,
        EnumFacing::South => 3,
        EnumFacing::West => 4,
        EnumFacing::East => 5,
        _ => 2,
    }
}

pub fn getBoundingBox(state: IBlockState) -> AxisAlignedBB {
    match facing(state) {
        EnumFacing::North => AxisAlignedBB::new(0.0, 0.0, 0.8125, 1.0, 1.0, 1.0),
        EnumFacing::South => AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 0.1875),
        EnumFacing::West => AxisAlignedBB::new(0.8125, 0.0, 0.0, 1.0, 1.0, 1.0),
        _ => AxisAlignedBB::new(0.0, 0.0, 0.0, 0.1875, 1.0, 1.0),
    }
}

fn canAttachTo<A: IBlockAccess>(world: &A, supportPos: BlockPos, face: EnumFacing) -> bool {
    let state = world.getBlockState(supportPos);
    !state.getBlock().func_193382_c()
        && state.getBlockFaceShape(world, supportPos, face) == BlockFaceShape::SOLID
        && !state.getBlock().canProvidePower()
}

/// Exact source ordering in `BlockLadder#canPlaceBlockOnSide`.
pub fn canPlaceBlockOnSide<A: IBlockAccess>(world: &A, pos: BlockPos, side: EnumFacing) -> bool {
    canAttachTo(world, pos.west(1), side)
        || canAttachTo(world, pos.east(1), side)
        || canAttachTo(world, pos.north(1), side)
        || canAttachTo(world, pos.south(1), side)
}

pub fn placementFacing<A: IBlockAccess>(
    world: &A,
    pos: BlockPos,
    requested: EnumFacing,
) -> EnumFacing {
    if requested.axis() != Axis::Y
        && canAttachTo(world, pos.offset(requested.opposite(), 1), requested)
    {
        return requested;
    }
    for facing in [
        EnumFacing::North,
        EnumFacing::East,
        EnumFacing::South,
        EnumFacing::West,
    ] {
        if canAttachTo(world, pos.offset(facing.opposite(), 1), facing) {
            return facing;
        }
    }
    EnumFacing::North
}

pub fn onBlockPlacedState<A: IBlockAccess>(
    world: &A,
    pos: BlockPos,
    requested: EnumFacing,
) -> IBlockState {
    IBlockState::fromGlobalStateId(
        (65 << 4) | metadataForFacing(placementFacing(world, pos, requested)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_uses_requested_supported_horizontal_face() {
        use std::collections::HashMap;
        struct Access(HashMap<BlockPos, IBlockState>);
        impl IBlockAccess for Access {
            fn getBlockState(&self, pos: BlockPos) -> IBlockState {
                self.0.get(&pos).copied().unwrap_or_default()
            }
        }
        let pos = BlockPos::new(0, 64, 0);
        let mut blocks = HashMap::new();
        blocks.insert(pos.west(1), IBlockState::fromGlobalStateId(1 << 4));
        let state = onBlockPlacedState(&Access(blocks), pos, EnumFacing::East);
        assert_eq!(state.getMetadata(), 5);
    }

    #[test]
    fn metadata_two_is_north_face_box() {
        let bounds = getBoundingBox(IBlockState::fromGlobalStateId((65 << 4) | 2));
        assert_eq!((bounds.min_z, bounds.max_z), (0.8125, 1.0));
    }
}
