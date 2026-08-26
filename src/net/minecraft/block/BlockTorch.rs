use crate::net::minecraft::block::state::BlockFaceShape::BlockFaceShape;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockFence;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

pub const fn isBlockTorch(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 50 | 75 | 76)
}

/// Exact `BlockTorch#getStateFromMeta` decode.
pub const fn facing(state: IBlockState) -> EnumFacing {
    match state.getMetadata() {
        1 => EnumFacing::East,
        2 => EnumFacing::West,
        3 => EnumFacing::South,
        4 => EnumFacing::North,
        _ => EnumFacing::Up,
    }
}

pub const fn metadataForFacing(facing: EnumFacing) -> i32 {
    match facing {
        EnumFacing::East => 1,
        EnumFacing::West => 2,
        EnumFacing::South => 3,
        EnumFacing::North => 4,
        _ => 5,
    }
}

/// Exact `BlockTorch#onBlockPlaced` facing selection.
pub fn placementFacing<A: IBlockAccess>(
    world: &A,
    pos: BlockPos,
    requested: EnumFacing,
) -> EnumFacing {
    if canPlaceAt(world, pos, requested) {
        return requested;
    }
    for candidate in [
        EnumFacing::North,
        EnumFacing::East,
        EnumFacing::South,
        EnumFacing::West,
    ] {
        if canPlaceAt(world, pos, candidate) {
            return candidate;
        }
    }
    EnumFacing::Up
}

pub fn onBlockPlacedState<A: IBlockAccess>(
    blockId: i32,
    world: &A,
    pos: BlockPos,
    requested: EnumFacing,
) -> IBlockState {
    IBlockState::fromGlobalStateId(
        (blockId << 4) | metadataForFacing(placementFacing(world, pos, requested)),
    )
}

/// Exact local-space `BlockTorch#getBoundingBox` result.
pub fn getBoundingBox(state: IBlockState) -> AxisAlignedBB {
    match facing(state) {
        EnumFacing::East => AxisAlignedBB::new(
            0.0,
            0.20000000298023224,
            0.3499999940395355,
            0.30000001192092896,
            0.800000011920929,
            0.6499999761581421,
        ),
        EnumFacing::West => AxisAlignedBB::new(
            0.699999988079071,
            0.20000000298023224,
            0.3499999940395355,
            1.0,
            0.800000011920929,
            0.6499999761581421,
        ),
        EnumFacing::South => AxisAlignedBB::new(
            0.3499999940395355,
            0.20000000298023224,
            0.0,
            0.6499999761581421,
            0.800000011920929,
            0.30000001192092896,
        ),
        EnumFacing::North => AxisAlignedBB::new(
            0.3499999940395355,
            0.20000000298023224,
            0.699999988079071,
            0.6499999761581421,
            0.800000011920929,
            1.0,
        ),
        _ => AxisAlignedBB::new(
            0.4000000059604645,
            0.0,
            0.4000000059604645,
            0.6000000238418579,
            0.6000000238418579,
            0.6000000238418579,
        ),
    }
}

fn canPlaceOn<A: IBlockAccess>(world: &A, pos: BlockPos) -> bool {
    let state = world.getBlockState(pos);
    let id = state.getBlockId();
    let forbidden = matches!(id, 91 | 209); // lit pumpkin / end gateway
    if state.isTopSolid() {
        !forbidden
    } else {
        (BlockFence::isBlockFence(state) || matches!(id, 20 | 95 | 139)) && !forbidden
    }
}

pub fn canPlaceAt<A: IBlockAccess>(world: &A, pos: BlockPos, facing: EnumFacing) -> bool {
    let supportPos = pos.offset(facing.opposite(), 1);
    let support = world.getBlockState(supportPos);
    if facing == EnumFacing::Up && canPlaceOn(world, supportPos) {
        true
    } else if !matches!(facing, EnumFacing::Up | EnumFacing::Down) {
        !support.getBlock().func_193382_c()
            && support.getBlockFaceShape(world, supportPos, facing) == BlockFaceShape::SOLID
    } else {
        false
    }
}

pub fn canPlaceBlockAt<A: IBlockAccess>(world: &A, pos: BlockPos) -> bool {
    [
        EnumFacing::Up,
        EnumFacing::North,
        EnumFacing::South,
        EnumFacing::West,
        EnumFacing::East,
    ]
    .into_iter()
    .any(|facing| canPlaceAt(world, pos, facing))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct Access(HashMap<BlockPos, IBlockState>);
    impl IBlockAccess for Access {
        fn getBlockState(&self, pos: BlockPos) -> IBlockState {
            self.0.get(&pos).copied().unwrap_or_default()
        }
    }

    #[test]
    fn wall_and_floor_bounds_match_mcp() {
        let east = getBoundingBox(IBlockState::fromGlobalStateId((50 << 4) | 1));
        let floor = getBoundingBox(IBlockState::fromGlobalStateId((50 << 4) | 5));
        assert_eq!(east.max_x, 0.30000001192092896);
        assert_eq!(floor.min_y, 0.0);
        assert_eq!(floor.max_y, 0.6000000238418579);
    }

    #[test]
    fn placement_falls_back_in_horizontal_plane_order() {
        let pos = BlockPos::new(0, 64, 0);
        let mut blocks = HashMap::new();
        blocks.insert(pos.south(1), IBlockState::fromGlobalStateId(1 << 4));
        let state = onBlockPlacedState(50, &Access(blocks), pos, EnumFacing::Down);
        assert_eq!(state.getMetadata(), 4); // NORTH attaches to the southern support
    }

    #[test]
    fn opaque_floor_supports_torch() {
        let pos = BlockPos::new(0, 64, 0);
        let mut blocks = HashMap::new();
        blocks.insert(pos.down(1), IBlockState::fromGlobalStateId(1 << 4));
        assert!(canPlaceBlockAt(&Access(blocks), pos));
    }
}
