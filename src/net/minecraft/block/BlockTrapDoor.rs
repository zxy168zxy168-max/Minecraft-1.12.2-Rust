use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

pub const fn isBlockTrapDoor(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 96 | 167)
}

pub const fn facing(state: IBlockState) -> EnumFacing {
    match state.getMetadata() & 3 {
        0 => EnumFacing::North,
        1 => EnumFacing::South,
        2 => EnumFacing::West,
        _ => EnumFacing::East,
    }
}

pub const fn isOpen(state: IBlockState) -> bool {
    state.getMetadata() & 4 != 0
}
pub const fn isTopHalf(state: IBlockState) -> bool {
    state.getMetadata() & 8 != 0
}

/// Exact state result of `BlockTrapDoor#onBlockActivated`. Iron trapdoors
/// reject manual activation; wooden trapdoors cycle OPEN (metadata bit 2).
pub fn onBlockActivatedState(state: IBlockState) -> Option<IBlockState> {
    if !isBlockTrapDoor(state) || state.getBlockId() == 167 {
        return None;
    }
    Some(IBlockState::fromGlobalStateId(
        (state.getBlockId() << 4) | (state.getMetadata() ^ 4),
    ))
}

pub const fn metadataForFacing(facing: EnumFacing) -> i32 {
    match facing {
        EnumFacing::North => 0,
        EnumFacing::South => 1,
        EnumFacing::West => 2,
        _ => 3,
    }
}

/// Exact metadata result of `BlockTrapDoor#onBlockPlaced`. `powered` is kept
/// explicit so the caller cannot silently invent a redstone result.
pub fn onBlockPlacedState(
    blockId: i32,
    clickedFace: EnumFacing,
    hitY: f32,
    placerHorizontalFacing: EnumFacing,
    powered: bool,
) -> IBlockState {
    let (facing, topHalf) = if !matches!(clickedFace, EnumFacing::Up | EnumFacing::Down) {
        (clickedFace, hitY > 0.5)
    } else {
        (
            placerHorizontalFacing.opposite(),
            clickedFace == EnumFacing::Down,
        )
    };
    let metadata =
        metadataForFacing(facing) | if powered { 4 } else { 0 } | if topHalf { 8 } else { 0 };
    IBlockState::fromGlobalStateId((blockId << 4) | metadata)
}

pub fn getBoundingBox(state: IBlockState) -> AxisAlignedBB {
    if isOpen(state) {
        match facing(state) {
            EnumFacing::North => AxisAlignedBB::new(0.0, 0.0, 0.8125, 1.0, 1.0, 1.0),
            EnumFacing::South => AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 0.1875),
            EnumFacing::West => AxisAlignedBB::new(0.8125, 0.0, 0.0, 1.0, 1.0, 1.0),
            _ => AxisAlignedBB::new(0.0, 0.0, 0.0, 0.1875, 1.0, 1.0),
        }
    } else if isTopHalf(state) {
        AxisAlignedBB::new(0.0, 0.8125, 0.0, 1.0, 1.0, 1.0)
    } else {
        AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 0.1875, 1.0)
    }
}

/// MCP `BlockTrapDoor#canPlaceBlockOnSide` is intentionally unconditional.
pub const fn canPlaceBlockOnSide() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_placement_uses_hit_height_and_face() {
        let state = onBlockPlacedState(96, EnumFacing::West, 0.75, EnumFacing::South, false);
        assert_eq!(state.getMetadata(), 10); // WEST + TOP
    }

    #[test]
    fn vertical_placement_uses_player_opposite_facing() {
        let state = onBlockPlacedState(96, EnumFacing::Up, 0.0, EnumFacing::South, false);
        assert_eq!(state.getMetadata(), 0); // NORTH + BOTTOM
    }

    #[test]
    fn open_north_trapdoor_uses_north_plane() {
        let bounds = getBoundingBox(IBlockState::fromGlobalStateId((96 << 4) | 4));
        assert_eq!((bounds.min_z, bounds.max_z), (0.8125, 1.0));
    }
}
