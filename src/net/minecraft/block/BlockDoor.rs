use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnumDoorHalf {
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnumHingePosition {
    Left,
    Right,
}

/// Source-visible actual state reconstructed from the split lower/upper door
/// metadata exactly as `BlockDoor#getActualState` does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DoorActualState {
    pub facing: EnumFacing,
    pub open: bool,
    pub hinge: EnumHingePosition,
    pub powered: bool,
    pub half: EnumDoorHalf,
}

pub const fn isBlockDoor(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 64 | 71 | 193 | 194 | 195 | 196 | 197)
}

pub const fn half(state: IBlockState) -> EnumDoorHalf {
    if state.getMetadata() & 8 != 0 {
        EnumDoorHalf::Upper
    } else {
        EnumDoorHalf::Lower
    }
}

fn lowerFacing(meta: i32) -> EnumFacing {
    // EnumFacing.getHorizontal(meta & 3).rotateYCCW(). Horizontal order is
    // SOUTH, WEST, NORTH, EAST.
    match meta & 3 {
        0 => EnumFacing::East,
        1 => EnumFacing::South,
        2 => EnumFacing::West,
        _ => EnumFacing::North,
    }
}

fn lowerState(state: IBlockState) -> DoorActualState {
    DoorActualState {
        facing: lowerFacing(state.getMetadata()),
        open: state.getMetadata() & 4 != 0,
        hinge: EnumHingePosition::Left,
        powered: false,
        half: EnumDoorHalf::Lower,
    }
}

fn upperState(state: IBlockState) -> DoorActualState {
    DoorActualState {
        facing: EnumFacing::North,
        open: false,
        hinge: if state.getMetadata() & 1 != 0 {
            EnumHingePosition::Right
        } else {
            EnumHingePosition::Left
        },
        powered: state.getMetadata() & 2 != 0,
        half: EnumDoorHalf::Upper,
    }
}

/// Exact world-state part of `BlockDoor#onBlockActivated`. The method returns
/// the lower-half position, its expected snapshot and the toggled state. Iron
/// doors reject manual activation.
pub fn onBlockActivatedState<A: IBlockAccess>(
    world: &A,
    pos: BlockPos,
    state: IBlockState,
) -> Option<(BlockPos, IBlockState, IBlockState)> {
    if !isBlockDoor(state) || state.getBlockId() == 71 {
        return None;
    }
    let lowerPos = if half(state) == EnumDoorHalf::Lower {
        pos
    } else {
        pos.down(1)
    };
    let lowerState = world.getBlockState(lowerPos);
    if lowerState.getBlockId() != state.getBlockId() || half(lowerState) != EnumDoorHalf::Lower {
        return None;
    }
    let toggled = IBlockState::fromGlobalStateId(
        (lowerState.getBlockId() << 4) | (lowerState.getMetadata() ^ 4),
    );
    Some((lowerPos, lowerState, toggled))
}

pub fn getActualState<A: IBlockAccess>(
    state: IBlockState,
    world: &A,
    pos: BlockPos,
) -> DoorActualState {
    let mut actual = match half(state) {
        EnumDoorHalf::Lower => lowerState(state),
        EnumDoorHalf::Upper => upperState(state),
    };
    let id = state.getBlockId();
    match actual.half {
        EnumDoorHalf::Lower => {
            let neighbour = world.getBlockState(pos.up(1));
            if neighbour.getBlockId() == id && half(neighbour) == EnumDoorHalf::Upper {
                let upper = upperState(neighbour);
                actual.hinge = upper.hinge;
                actual.powered = upper.powered;
            }
        }
        EnumDoorHalf::Upper => {
            let neighbour = world.getBlockState(pos.down(1));
            if neighbour.getBlockId() == id && half(neighbour) == EnumDoorHalf::Lower {
                let lower = lowerState(neighbour);
                actual.facing = lower.facing;
                actual.open = lower.open;
            }
        }
    }
    actual
}

/// Exact local-space result of `BlockDoor#getBoundingBox` after actual-state
/// reconstruction. The same box is also the default collision box.
pub fn getBoundingBox<A: IBlockAccess>(
    state: IBlockState,
    world: &A,
    pos: BlockPos,
) -> AxisAlignedBB {
    let actual = getActualState(state, world, pos);
    let closed = !actual.open;
    let right = actual.hinge == EnumHingePosition::Right;
    match actual.facing {
        EnumFacing::East => {
            if closed {
                AxisAlignedBB::new(0.0, 0.0, 0.0, 0.1875, 1.0, 1.0)
            } else if right {
                AxisAlignedBB::new(0.0, 0.0, 0.8125, 1.0, 1.0, 1.0)
            } else {
                AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 0.1875)
            }
        }
        EnumFacing::South => {
            if closed {
                AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 0.1875)
            } else if right {
                AxisAlignedBB::new(0.0, 0.0, 0.0, 0.1875, 1.0, 1.0)
            } else {
                AxisAlignedBB::new(0.8125, 0.0, 0.0, 1.0, 1.0, 1.0)
            }
        }
        EnumFacing::West => {
            if closed {
                AxisAlignedBB::new(0.8125, 0.0, 0.0, 1.0, 1.0, 1.0)
            } else if right {
                AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 0.1875)
            } else {
                AxisAlignedBB::new(0.0, 0.0, 0.8125, 1.0, 1.0, 1.0)
            }
        }
        EnumFacing::North => {
            if closed {
                AxisAlignedBB::new(0.0, 0.0, 0.8125, 1.0, 1.0, 1.0)
            } else if right {
                AxisAlignedBB::new(0.8125, 0.0, 0.0, 1.0, 1.0, 1.0)
            } else {
                AxisAlignedBB::new(0.0, 0.0, 0.0, 0.1875, 1.0, 1.0)
            }
        }
        _ => AxisAlignedBB::new(0.0, 0.0, 0.8125, 1.0, 1.0, 1.0),
    }
}

/// Variant consumed by the 1.12.2 door blockstate JSON. `powered` is
/// deliberately omitted because vanilla's state mapper ignores it.
pub fn modelVariant<A: IBlockAccess>(state: IBlockState, world: &A, pos: BlockPos) -> String {
    let actual = getActualState(state, world, pos);
    let facing = match actual.facing {
        EnumFacing::East => "east",
        EnumFacing::South => "south",
        EnumFacing::West => "west",
        _ => "north",
    };
    let half = match actual.half {
        EnumDoorHalf::Lower => "lower",
        EnumDoorHalf::Upper => "upper",
    };
    let hinge = match actual.hinge {
        EnumHingePosition::Left => "left",
        EnumHingePosition::Right => "right",
    };
    format!(
        "facing={facing},half={half},hinge={hinge},open={}",
        actual.open
    )
}

/// Decode the compact key into the exact state-mapper variant. This is used
/// during atlas construction to bake all source-valid door combinations before
/// any world is joined.
pub fn modelVariantFromKey(key: u8) -> String {
    let facing = match key & 3 {
        0 => "south",
        1 => "west",
        2 => "north",
        _ => "east",
    };
    let half = if key & 4 != 0 { "upper" } else { "lower" };
    let hinge = if key & 8 != 0 { "right" } else { "left" };
    let open = key & 16 != 0;
    format!("facing={facing},half={half},hinge={hinge},open={open}")
}

/// Compact cache key for all model-relevant actual properties.
pub fn modelKey<A: IBlockAccess>(state: IBlockState, world: &A, pos: BlockPos) -> u8 {
    let actual = getActualState(state, world, pos);
    let facing = match actual.facing {
        EnumFacing::South => 0,
        EnumFacing::West => 1,
        EnumFacing::North => 2,
        EnumFacing::East => 3,
        _ => 2,
    };
    facing
        | ((actual.half == EnumDoorHalf::Upper) as u8) << 2
        | ((actual.hinge == EnumHingePosition::Right) as u8) << 3
        | (actual.open as u8) << 4
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
    fn both_halves_reconstruct_the_same_actual_properties() {
        let pos = BlockPos::new(0, 64, 0);
        let lower = IBlockState::fromGlobalStateId((64 << 4) | 4); // east, open
        let upper = IBlockState::fromGlobalStateId((64 << 4) | 11); // upper, right, powered
        let mut blocks = HashMap::new();
        blocks.insert(pos, lower);
        blocks.insert(pos.up(1), upper);
        let access = Access(blocks);
        let lowerActual = getActualState(lower, &access, pos);
        let upperActual = getActualState(upper, &access, pos.up(1));
        assert_eq!(lowerActual.facing, EnumFacing::East);
        assert!(lowerActual.open);
        assert_eq!(lowerActual.hinge, EnumHingePosition::Right);
        assert!(lowerActual.powered);
        assert_eq!(upperActual.facing, lowerActual.facing);
        assert_eq!(upperActual.open, lowerActual.open);
        assert_eq!(
            modelVariant(lower, &access, pos),
            "facing=east,half=lower,hinge=right,open=true"
        );
    }

    #[test]
    fn open_right_hinged_east_door_uses_north_plane() {
        let pos = BlockPos::new(0, 64, 0);
        let lower = IBlockState::fromGlobalStateId((64 << 4) | 4);
        let upper = IBlockState::fromGlobalStateId((64 << 4) | 9);
        let mut blocks = HashMap::new();
        blocks.insert(pos, lower);
        blocks.insert(pos.up(1), upper);
        let bounds = getBoundingBox(lower, &Access(blocks), pos);
        assert_eq!((bounds.min_z, bounds.max_z), (0.8125, 1.0));
    }
}
