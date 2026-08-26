use crate::net::minecraft::block::state::BlockFaceShape::BlockFaceShape;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::{Axis, EnumFacing};
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

pub const fn isBlockFenceGate(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 107 | 183 | 184 | 185 | 186 | 187)
}

/// `EnumFacing.getHorizontal(meta)` in 1.12.2: SOUTH, WEST, NORTH, EAST.
pub const fn facing(state: IBlockState) -> EnumFacing {
    match state.getMetadata() & 3 {
        0 => EnumFacing::South,
        1 => EnumFacing::West,
        2 => EnumFacing::North,
        _ => EnumFacing::East,
    }
}

pub const fn isOpen(state: IBlockState) -> bool {
    state.getMetadata() & 4 != 0
}

/// Exact state result of `BlockFenceGate#onBlockActivated`. Opening from the
/// back also changes FACING to the player's horizontal facing.
pub fn onBlockActivatedState(state: IBlockState, playerYaw: f32) -> Option<IBlockState> {
    if !isBlockFenceGate(state) {
        return None;
    }
    let metadata = if isOpen(state) {
        state.getMetadata() & !4
    } else {
        let playerFacing = EnumFacing::fromAngle(playerYaw as f64);
        let facingBits = if facing(state) == playerFacing.opposite() {
            playerFacing.horizontalIndex().unwrap_or(0) as i32
        } else {
            state.getMetadata() & 3
        };
        (state.getMetadata() & !7) | facingBits | 4
    };
    Some(IBlockState::fromGlobalStateId(
        (state.getBlockId() << 4) | metadata,
    ))
}

/// Port of `BlockFenceGate.func_193383_a` / `getBlockFaceShape`.
pub fn getBlockFaceShape(state: IBlockState, face: EnumFacing) -> BlockFaceShape {
    if face.axis() == Axis::Y {
        BlockFaceShape::UNDEFINED
    } else if facing(state).axis() == face.rotateY().axis() {
        BlockFaceShape::MIDDLE_POLE
    } else {
        BlockFaceShape::UNDEFINED
    }
}

/// Port of the closed-gate collision branch in
/// `BlockFenceGate.getCollisionBoundingBox`.
pub fn getCollisionBoxes(state: IBlockState) -> Vec<AxisAlignedBB> {
    if isOpen(state) {
        return Vec::new();
    }
    if facing(state).axis() == Axis::Z {
        vec![AxisAlignedBB::new(0.0, 0.0, 0.375, 1.0, 1.5, 0.625)]
    } else {
        vec![AxisAlignedBB::new(0.375, 0.0, 0.0, 0.625, 1.5, 1.0)]
    }
}

/// `BlockFenceGate#getBoundingBox` after `getActualState`. Unlike collision,
/// an open gate still has a selectable visible model. A gate embedded in a
/// cobblestone wall is shortened to 0.8125 exactly as the source AABBs.
pub fn getBoundingBox<A: IBlockAccess>(
    state: IBlockState,
    world: &A,
    pos: BlockPos,
) -> AxisAlignedBB {
    let axis = facing(state).axis();
    let inWall = if axis == Axis::Z {
        world.getBlockState(pos.west(1)).getBlockId() == 139
            || world.getBlockState(pos.east(1)).getBlockId() == 139
    } else {
        world.getBlockState(pos.north(1)).getBlockId() == 139
            || world.getBlockState(pos.south(1)).getBlockId() == 139
    };
    let maxY = if inWall { 0.8125 } else { 1.0 };
    if axis == Axis::X {
        AxisAlignedBB::new(0.375, 0.0, 0.0, 0.625, maxY, 1.0)
    } else {
        AxisAlignedBB::new(0.0, 0.0, 0.375, 1.0, maxY, 0.625)
    }
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
    fn open_gate_has_no_collision() {
        let open = IBlockState::fromGlobalStateId((107 << 4) | 4);
        assert!(getCollisionBoxes(open).is_empty());
    }

    #[test]
    fn gate_face_shape_uses_cross_axis() {
        let south = IBlockState::fromGlobalStateId(107 << 4);
        assert_eq!(
            getBlockFaceShape(south, EnumFacing::East),
            BlockFaceShape::MIDDLE_POLE
        );
        assert_eq!(
            getBlockFaceShape(south, EnumFacing::South),
            BlockFaceShape::UNDEFINED
        );
    }

    #[test]
    fn open_gate_remains_selectable_and_in_wall_is_shorter() {
        let pos = BlockPos::new(0, 64, 0);
        let openSouth = IBlockState::fromGlobalStateId((107 << 4) | 4);
        let plain = getBoundingBox(openSouth, &Access(HashMap::new()), pos);
        assert_eq!(plain.max_y, 1.0);
        let mut blocks = HashMap::new();
        blocks.insert(pos.west(1), IBlockState::fromGlobalStateId(139 << 4));
        let embedded = getBoundingBox(openSouth, &Access(blocks), pos);
        assert_eq!(embedded.max_y, 0.8125);
    }
}

/// MCP `BlockFenceGate#getActualState` IN_WALL property.
pub fn isInWall<A: IBlockAccess>(state: IBlockState, world: &A, pos: BlockPos) -> bool {
    let axis = facing(state).axis();
    if axis == Axis::Z {
        world.getBlockState(pos.west(1)).getBlockId() == 139
            || world.getBlockState(pos.east(1)).getBlockId() == 139
    } else {
        world.getBlockState(pos.north(1)).getBlockId() == 139
            || world.getBlockState(pos.south(1)).getBlockId() == 139
    }
}

/// State-map key. POWERED is deliberately ignored by MCP BlockModelShapes.
pub fn modelVariant<A: IBlockAccess>(state: IBlockState, world: &A, pos: BlockPos) -> String {
    let facingName = match facing(state) {
        EnumFacing::North => "north",
        EnumFacing::South => "south",
        EnumFacing::West => "west",
        EnumFacing::East => "east",
        _ => "south",
    };
    format!(
        "facing={facingName},in_wall={},open={}",
        isInWall(state, world, pos),
        isOpen(state),
    )
}

pub fn modelKey<A: IBlockAccess>(state: IBlockState, world: &A, pos: BlockPos) -> u8 {
    (state.getMetadata() as u8 & 3)
        | ((isOpen(state) as u8) << 2)
        | ((isInWall(state, world, pos) as u8) << 3)
}

pub fn modelVariantFromKey(key: u8) -> String {
    let facingName = match key & 3 {
        0 => "south",
        1 => "west",
        2 => "north",
        _ => "east",
    };
    format!(
        "facing={facingName},in_wall={},open={}",
        key & 8 != 0,
        key & 4 != 0,
    )
}
