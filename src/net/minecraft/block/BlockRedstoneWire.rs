use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

pub const BLOCK_ID: i32 = 55;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumAttachPosition {
    None,
    Side,
    Up,
}

impl EnumAttachPosition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Side => "side",
            Self::Up => "up",
        }
    }

    pub const fn ordinal(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Side => 1,
            Self::Up => 2,
        }
    }

    pub const fn fromOrdinal(value: u8) -> Self {
        match value {
            1 => Self::Side,
            2 => Self::Up,
            _ => Self::None,
        }
    }
}

pub fn getAttachPosition<A: IBlockAccess>(
    world: &A,
    pos: BlockPos,
    direction: EnumFacing,
) -> EnumAttachPosition {
    let neighbourPos = pos.offset(direction, 1);
    let neighbour = world.getBlockState(neighbourPos);

    if !canConnectTo(neighbour, Some(direction))
        && (isNormalCube(neighbour)
            || !canConnectUpwardsTo(world.getBlockState(neighbourPos.down(1))))
    {
        let aboveWire = world.getBlockState(pos.up(1));
        if !isNormalCube(aboveWire) {
            let topSupportsWire = neighbour.isTopSolid() || neighbour.getBlockId() == 89;
            if topSupportsWire && canConnectUpwardsTo(world.getBlockState(neighbourPos.up(1))) {
                return if isNormalCube(neighbour) {
                    EnumAttachPosition::Up
                } else {
                    EnumAttachPosition::Side
                };
            }
        }
        EnumAttachPosition::None
    } else {
        EnumAttachPosition::Side
    }
}

pub fn modelKey<A: IBlockAccess>(world: &A, pos: BlockPos) -> u8 {
    let north = getAttachPosition(world, pos, EnumFacing::North).ordinal();
    let east = getAttachPosition(world, pos, EnumFacing::East).ordinal();
    let south = getAttachPosition(world, pos, EnumFacing::South).ordinal();
    let west = getAttachPosition(world, pos, EnumFacing::West).ordinal();
    north + east * 3 + south * 9 + west * 27
}

pub fn modelVariant<A: IBlockAccess>(world: &A, pos: BlockPos) -> String {
    modelVariantFromKey(modelKey(world, pos))
}

pub fn modelVariantFromKey(mut key: u8) -> String {
    let north = EnumAttachPosition::fromOrdinal(key % 3);
    key /= 3;
    let east = EnumAttachPosition::fromOrdinal(key % 3);
    key /= 3;
    let south = EnumAttachPosition::fromOrdinal(key % 3);
    key /= 3;
    let west = EnumAttachPosition::fromOrdinal(key % 3);
    format!(
        "east={},north={},south={},west={}",
        east.as_str(),
        north.as_str(),
        south.as_str(),
        west.as_str(),
    )
}

fn canConnectUpwardsTo(state: IBlockState) -> bool {
    canConnectTo(state, None)
}

fn canConnectTo(state: IBlockState, side: Option<EnumFacing>) -> bool {
    match state.getBlockId() {
        BLOCK_ID => true,
        93 | 94 => {
            let facing = horizontalFacing(state.getMetadata());
            side.is_some_and(|side| facing == side || facing.opposite() == side)
        }
        218 => side.is_some_and(|side| observerFacing(state.getMetadata()) == side),
        _ => side.is_some() && state.getBlock().canProvidePower(),
    }
}

fn horizontalFacing(meta: i32) -> EnumFacing {
    match meta & 3 {
        0 => EnumFacing::South,
        1 => EnumFacing::West,
        2 => EnumFacing::North,
        _ => EnumFacing::East,
    }
}

fn observerFacing(meta: i32) -> EnumFacing {
    match meta & 7 {
        0 => EnumFacing::Down,
        1 => EnumFacing::Up,
        2 => EnumFacing::North,
        3 => EnumFacing::South,
        4 => EnumFacing::West,
        _ => EnumFacing::East,
    }
}

fn isNormalCube(state: IBlockState) -> bool {
    state.getBlock().isFullyOpaque(state)
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
    fn isolated_wire_has_four_none_properties() {
        let pos = BlockPos::ORIGIN;
        assert_eq!(
            modelVariant(&Access(HashMap::new()), pos),
            "east=none,north=none,south=none,west=none"
        );
    }

    #[test]
    fn adjacent_wire_connects_side() {
        let pos = BlockPos::ORIGIN;
        let mut blocks = HashMap::new();
        blocks.insert(pos.north(1), IBlockState::fromGlobalStateId(BLOCK_ID << 4));
        assert_eq!(
            getAttachPosition(&Access(blocks), pos, EnumFacing::North),
            EnumAttachPosition::Side
        );
    }

    #[test]
    fn key_round_trips_all_multipart_combinations() {
        for key in 0_u8..81 {
            let variant = modelVariantFromKey(key);
            assert!(variant.contains("east="));
            assert!(variant.contains("north="));
            assert!(variant.contains("south="));
            assert!(variant.contains("west="));
        }
    }
}
