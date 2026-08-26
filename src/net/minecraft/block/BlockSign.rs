use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

pub const fn isBlockSign(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 63 | 68)
}
pub const fn isStandingSign(state: IBlockState) -> bool {
    state.getBlockId() == 63
}

pub const fn wallFacing(state: IBlockState) -> EnumFacing {
    match state.getMetadata() % 6 {
        2 => EnumFacing::North,
        3 => EnumFacing::South,
        4 => EnumFacing::West,
        5 => EnumFacing::East,
        _ => EnumFacing::North,
    }
}

pub const fn wallMetadata(facing: EnumFacing) -> i32 {
    match facing {
        EnumFacing::North => 2,
        EnumFacing::South => 3,
        EnumFacing::West => 4,
        EnumFacing::East => 5,
        _ => 2,
    }
}

pub fn standingPlacementState(rotation: i32) -> IBlockState {
    IBlockState::fromGlobalStateId((63 << 4) | (rotation & 15))
}

pub fn wallPlacementState(facing: EnumFacing) -> IBlockState {
    IBlockState::fromGlobalStateId((68 << 4) | wallMetadata(facing))
}

/// `BlockContainer#hasInvalidNeighbor` plus `BlockSign#canPlaceBlockAt`.
pub fn canPlaceBlockAt(world: &WorldClient, pos: BlockPos) -> bool {
    let cactusNeighbour = [
        EnumFacing::North,
        EnumFacing::South,
        EnumFacing::West,
        EnumFacing::East,
    ]
    .into_iter()
    .any(|facing| world.getBlockState(pos.offset(facing, 1)).getBlockId() == 81);
    !cactusNeighbour && world.isBlockReplaceable(pos)
}

pub fn getBoundingBox(state: IBlockState) -> AxisAlignedBB {
    if isStandingSign(state) {
        AxisAlignedBB::new(0.25, 0.0, 0.25, 0.75, 1.0, 0.75)
    } else {
        match wallFacing(state) {
            EnumFacing::North => AxisAlignedBB::new(0.0, 0.28125, 0.875, 1.0, 0.78125, 1.0),
            EnumFacing::South => AxisAlignedBB::new(0.0, 0.28125, 0.0, 1.0, 0.78125, 0.125),
            EnumFacing::West => AxisAlignedBB::new(0.875, 0.28125, 0.0, 1.0, 0.78125, 1.0),
            _ => AxisAlignedBB::new(0.0, 0.28125, 0.0, 0.125, 0.78125, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_states_encode_rotation_and_wall_facing() {
        assert_eq!(standingPlacementState(17).getMetadata(), 1);
        assert_eq!(wallPlacementState(EnumFacing::East).getMetadata(), 5);
    }

    #[test]
    fn wall_sign_bounds_follow_received_facing() {
        let north = getBoundingBox(IBlockState::fromGlobalStateId((68 << 4) | 2));
        assert_eq!((north.min_z, north.max_z), (0.875, 1.0));
    }
}
