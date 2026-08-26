use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnumRailDirection {
    NorthSouth,
    EastWest,
    AscendingEast,
    AscendingWest,
    AscendingNorth,
    AscendingSouth,
    SouthEast,
    SouthWest,
    NorthWest,
    NorthEast,
}

pub const fn isRailBlock(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 27 | 28 | 66 | 157)
}

pub const fn direction(state: IBlockState) -> EnumRailDirection {
    let meta = if matches!(state.getBlockId(), 27 | 28 | 157) {
        state.getMetadata() & 7
    } else {
        state.getMetadata()
    };
    match meta {
        1 => EnumRailDirection::EastWest,
        2 => EnumRailDirection::AscendingEast,
        3 => EnumRailDirection::AscendingWest,
        4 => EnumRailDirection::AscendingNorth,
        5 => EnumRailDirection::AscendingSouth,
        6 => EnumRailDirection::SouthEast,
        7 => EnumRailDirection::SouthWest,
        8 => EnumRailDirection::NorthWest,
        9 => EnumRailDirection::NorthEast,
        _ => EnumRailDirection::NorthSouth,
    }
}

pub const fn isAscending(direction: EnumRailDirection) -> bool {
    matches!(
        direction,
        EnumRailDirection::AscendingEast
            | EnumRailDirection::AscendingWest
            | EnumRailDirection::AscendingNorth
            | EnumRailDirection::AscendingSouth
    )
}

pub fn getBoundingBox(state: IBlockState) -> AxisAlignedBB {
    if isAscending(direction(state)) {
        AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)
    } else {
        AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 0.125, 1.0)
    }
}

pub fn canPlaceBlockAt<A: IBlockAccess>(world: &A, pos: BlockPos) -> bool {
    world.getBlockState(pos.down(1)).isTopSolid()
}

/// `BlockRailBase` inherits `Block#onBlockPlaced`, so the initial client state
/// is simply `getStateFromMeta(meta)`. Server-side `onBlockAdded/updateDir`
/// later resolves neighbour topology.
pub fn onBlockPlacedState(blockId: i32, metadata: i32) -> IBlockState {
    IBlockState::fromGlobalStateId((blockId << 4) | (metadata & 15))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_itemblock_state_preserves_legacy_metadata() {
        assert_eq!(onBlockPlacedState(66, 9).getGlobalStateId(), (66 << 4) | 9);
    }

    #[test]
    fn ascending_rail_uses_half_block_selection_height() {
        let flat = getBoundingBox(IBlockState::fromGlobalStateId(66 << 4));
        let ascending = getBoundingBox(IBlockState::fromGlobalStateId((66 << 4) | 2));
        assert_eq!(flat.max_y, 0.125);
        assert_eq!(ascending.max_y, 0.5);
    }
}
