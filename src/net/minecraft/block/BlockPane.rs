use crate::net::minecraft::block::state::BlockFaceShape::BlockFaceShape;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::{Axis, EnumFacing};
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

pub const fn isBlockPane(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 101 | 102 | 160)
}

pub fn getBlockFaceShape(face: EnumFacing) -> BlockFaceShape {
    if face.axis() == Axis::Y {
        BlockFaceShape::CENTER_SMALL
    } else {
        BlockFaceShape::MIDDLE_POLE_THIN
    }
}

/// Exact exclusion set from `BlockPane.func_193394_e`.
pub const fn excludesSolidConnection(blockId: i32) -> bool {
    matches!(
        blockId,
        18 | 29 | 33 | 34 | 79 | 86 | 89 | 91 | 103 | 118 | 138 | 161 | 166 | 169 | 219..=234
    )
}

pub fn canConnectTo<A: IBlockAccess>(
    world: &A,
    neighbourPos: BlockPos,
    neighbourFace: EnumFacing,
) -> bool {
    let neighbour = world.getBlockState(neighbourPos);
    let shape = neighbour.getBlockFaceShape(world, neighbourPos, neighbourFace);
    (!excludesSolidConnection(neighbour.getBlockId()) && shape == BlockFaceShape::SOLID)
        || shape == BlockFaceShape::MIDDLE_POLE_THIN
}

pub fn connectionMask<A: IBlockAccess>(world: &A, pos: BlockPos) -> u8 {
    let mut mask = 0;
    for (bit, direction) in [
        (1, EnumFacing::North),
        (2, EnumFacing::East),
        (4, EnumFacing::South),
        (8, EnumFacing::West),
    ] {
        let neighbourPos = pos.offset(direction, 1);
        if canConnectTo(world, neighbourPos, direction.opposite()) {
            mask |= bit;
        }
    }
    mask
}

/// Port of `BlockPane.addCollisionBoxToList` after `getActualState`.
pub fn getCollisionBoxes(mask: u8) -> Vec<AxisAlignedBB> {
    let mut boxes = vec![AxisAlignedBB::new(0.4375, 0.0, 0.4375, 0.5625, 1.0, 0.5625)];
    if mask & 1 != 0 {
        boxes.push(AxisAlignedBB::new(0.4375, 0.0, 0.0, 0.5625, 1.0, 0.4375));
    }
    if mask & 2 != 0 {
        boxes.push(AxisAlignedBB::new(0.5625, 0.0, 0.4375, 1.0, 1.0, 0.5625));
    }
    if mask & 4 != 0 {
        boxes.push(AxisAlignedBB::new(0.4375, 0.0, 0.5625, 0.5625, 1.0, 1.0));
    }
    if mask & 8 != 0 {
        boxes.push(AxisAlignedBB::new(0.0, 0.0, 0.4375, 0.4375, 1.0, 0.5625));
    }
    boxes
}

/// `BlockPane.AABB_BY_INDEX[getBoundingBoxIndex(actualState)]`.
pub fn getBoundingBox(mask: u8) -> AxisAlignedBB {
    AxisAlignedBB::new(
        if mask & 8 != 0 { 0.0 } else { 0.4375 },
        0.0,
        if mask & 1 != 0 { 0.0 } else { 0.4375 },
        if mask & 2 != 0 { 1.0 } else { 0.5625 },
        1.0,
        if mask & 4 != 0 { 1.0 } else { 0.5625 },
    )
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
    fn pane_connects_to_vanilla_glass_face() {
        let pos = BlockPos::new(0, 64, 0);
        let mut blocks = HashMap::new();
        blocks.insert(pos.east(1), IBlockState::fromGlobalStateId(20 << 4));
        assert_ne!(connectionMask(&Access(blocks), pos) & 2, 0);
    }

    #[test]
    fn pane_selected_bounds_are_the_actual_state_union() {
        let box_ = getBoundingBox(2 | 8);
        assert_eq!((box_.min_x, box_.max_x), (0.0, 1.0));
        assert_eq!((box_.min_z, box_.max_z), (0.4375, 0.5625));
    }
}
