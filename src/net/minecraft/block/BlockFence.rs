use crate::net::minecraft::block::state::BlockFaceShape::BlockFaceShape;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockFenceGate;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::{Axis, EnumFacing};
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

pub const fn isBlockFence(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 85 | 113 | 188 | 189 | 190 | 191 | 192)
}

pub const fn isWoodFence(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 85 | 188 | 189 | 190 | 191 | 192)
}

pub const fn sameMaterial(left: IBlockState, right: IBlockState) -> bool {
    (isWoodFence(left) && isWoodFence(right))
        || (left.getBlockId() == 113 && right.getBlockId() == 113)
}

pub fn getBlockFaceShape(face: EnumFacing) -> BlockFaceShape {
    if face.axis() == Axis::Y {
        BlockFaceShape::CENTER
    } else {
        BlockFaceShape::MIDDLE_POLE
    }
}

/// Exact exclusion set from `Block.func_193382_c` plus the four additional
/// blocks in `BlockFence.func_194142_e`.
pub const fn excludesSolidConnection(blockId: i32) -> bool {
    matches!(
        blockId,
        18 | 29
            | 33
            | 34
            | 79
            | 86
            | 89
            | 91
            | 95
            | 96
            | 103
            | 118
            | 138
            | 161
            | 166
            | 167
            | 169
            | 219..=234 | 20
    )
}

pub fn canConnectTo<A: IBlockAccess>(
    fence: IBlockState,
    world: &A,
    neighbourPos: BlockPos,
    neighbourFace: EnumFacing,
) -> bool {
    let neighbour = world.getBlockState(neighbourPos);
    let shape = neighbour.getBlockFaceShape(world, neighbourPos, neighbourFace);
    let pole = shape == BlockFaceShape::MIDDLE_POLE
        && (sameMaterial(fence, neighbour) || BlockFenceGate::isBlockFenceGate(neighbour));
    (!excludesSolidConnection(neighbour.getBlockId()) && shape == BlockFaceShape::SOLID) || pole
}

pub fn connectionMask<A: IBlockAccess>(state: IBlockState, world: &A, pos: BlockPos) -> u8 {
    let mut mask = 0;
    for (bit, direction) in [
        (1, EnumFacing::North),
        (2, EnumFacing::East),
        (4, EnumFacing::South),
        (8, EnumFacing::West),
    ] {
        let neighbourPos = pos.offset(direction, 1);
        if canConnectTo(state, world, neighbourPos, direction.opposite()) {
            mask |= bit;
        }
    }
    mask
}

/// Port of `BlockFence.addCollisionBoxToList` after `getActualState`.
pub fn getCollisionBoxes(mask: u8) -> Vec<AxisAlignedBB> {
    let mut boxes = vec![AxisAlignedBB::new(0.375, 0.0, 0.375, 0.625, 1.5, 0.625)];
    if mask & 1 != 0 {
        boxes.push(AxisAlignedBB::new(0.375, 0.0, 0.0, 0.625, 1.5, 0.375));
    }
    if mask & 2 != 0 {
        boxes.push(AxisAlignedBB::new(0.625, 0.0, 0.375, 1.0, 1.5, 0.625));
    }
    if mask & 4 != 0 {
        boxes.push(AxisAlignedBB::new(0.375, 0.0, 0.625, 0.625, 1.5, 1.0));
    }
    if mask & 8 != 0 {
        boxes.push(AxisAlignedBB::new(0.0, 0.0, 0.375, 0.375, 1.5, 0.625));
    }
    boxes
}

/// `BlockFence.BOUNDING_BOXES[getBoundingBoxIdx(actualState)]`.
/// Selection/ray bounds are one 1-block-high union, distinct from the
/// 1.5-block-high entity collision post and arms above.
pub fn getBoundingBox(mask: u8) -> AxisAlignedBB {
    AxisAlignedBB::new(
        if mask & 8 != 0 { 0.0 } else { 0.375 },
        0.0,
        if mask & 1 != 0 { 0.0 } else { 0.375 },
        if mask & 2 != 0 { 1.0 } else { 0.625 },
        1.0,
        if mask & 4 != 0 { 1.0 } else { 0.625 },
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

    fn state(id: i32) -> IBlockState {
        IBlockState::fromGlobalStateId(id << 4)
    }

    #[test]
    fn wood_fences_share_material_but_do_not_join_nether_fence() {
        let pos = BlockPos::new(0, 64, 0);
        let mut blocks = HashMap::new();
        blocks.insert(pos.north(1), state(188));
        blocks.insert(pos.south(1), state(113));
        let mask = connectionMask(state(85), &Access(blocks), pos);
        assert_ne!(mask & 1, 0);
        assert_eq!(mask & 4, 0);
    }

    #[test]
    fn connected_fence_collision_keeps_tall_post_and_arm() {
        let boxes = getCollisionBoxes(1);
        assert_eq!(boxes.len(), 2);
        assert_eq!(boxes[0].max_y, 1.5);
        assert_eq!(boxes[1].min_z, 0.0);
    }

    #[test]
    fn selected_bounds_use_the_one_block_high_union() {
        let box_ = getBoundingBox(1 | 2);
        assert_eq!((box_.min_x, box_.max_x), (0.375, 1.0));
        assert_eq!((box_.min_z, box_.max_z), (0.0, 0.625));
        assert_eq!(box_.max_y, 1.0);
    }
}
