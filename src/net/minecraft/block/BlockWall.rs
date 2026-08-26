use crate::net::minecraft::block::state::BlockFaceShape::BlockFaceShape;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockFence;
use crate::net::minecraft::block::BlockFenceGate;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::{Axis, EnumFacing};
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

pub const fn isBlockWall(state: IBlockState) -> bool {
    state.getBlockId() == 139
}

pub fn getBlockFaceShape(face: EnumFacing) -> BlockFaceShape {
    if face.axis() == Axis::Y {
        BlockFaceShape::CENTER_BIG
    } else {
        BlockFaceShape::MIDDLE_POLE_THICK
    }
}

pub fn canConnectTo<A: IBlockAccess>(
    world: &A,
    neighbourPos: BlockPos,
    neighbourFace: EnumFacing,
) -> bool {
    let neighbour = world.getBlockState(neighbourPos);
    let shape = neighbour.getBlockFaceShape(world, neighbourPos, neighbourFace);
    let pole = shape == BlockFaceShape::MIDDLE_POLE_THICK
        || (shape == BlockFaceShape::MIDDLE_POLE && BlockFenceGate::isBlockFenceGate(neighbour));
    (!BlockFence::excludesSolidConnection(neighbour.getBlockId()) && shape == BlockFaceShape::SOLID)
        || pole
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
    let north = mask & 1 != 0;
    let east = mask & 2 != 0;
    let south = mask & 4 != 0;
    let west = mask & 8 != 0;
    let straight = (north && south && !east && !west) || (!north && !south && east && west);
    if !straight || !world.getBlockState(pos.up(1)).isAir() {
        mask |= 16;
    }
    mask
}

/// Port of `BlockWall.CLIP_AABB_BY_INDEX` selected by actual-state mask.
pub fn getCollisionBoxes(mask: u8) -> Vec<AxisAlignedBB> {
    let north = mask & 1 != 0;
    let east = mask & 2 != 0;
    let south = mask & 4 != 0;
    let west = mask & 8 != 0;
    if north && south && !east && !west {
        return vec![AxisAlignedBB::new(0.3125, 0.0, 0.0, 0.6875, 1.5, 1.0)];
    }
    if east && west && !north && !south {
        return vec![AxisAlignedBB::new(0.0, 0.0, 0.3125, 1.0, 1.5, 0.6875)];
    }
    let minX = if west { 0.0 } else { 0.25 };
    let maxX = if east { 1.0 } else { 0.75 };
    let minZ = if north { 0.0 } else { 0.25 };
    let maxZ = if south { 1.0 } else { 0.75 };
    vec![AxisAlignedBB::new(minX, 0.0, minZ, maxX, 1.5, maxZ)]
}

/// `BlockWall.AABB_BY_INDEX[getAABBIndex(actualState)]`. The two straight,
/// post-less shapes are 0.875 high; every other selected shape is 1.0 high.
/// This must not reuse `CLIP_AABB_BY_INDEX`, whose collision height is 1.5.
pub fn getBoundingBox(mask: u8) -> AxisAlignedBB {
    let north = mask & 1 != 0;
    let east = mask & 2 != 0;
    let south = mask & 4 != 0;
    let west = mask & 8 != 0;
    if north && south && !east && !west {
        return AxisAlignedBB::new(0.3125, 0.0, 0.0, 0.6875, 0.875, 1.0);
    }
    if east && west && !north && !south {
        return AxisAlignedBB::new(0.0, 0.0, 0.3125, 1.0, 0.875, 0.6875);
    }
    AxisAlignedBB::new(
        if west { 0.0 } else { 0.25 },
        0.0,
        if north { 0.0 } else { 0.25 },
        if east { 1.0 } else { 0.75 },
        1.0,
        if south { 1.0 } else { 0.75 },
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
    fn straight_wall_without_block_above_hides_center_post() {
        let pos = BlockPos::new(0, 64, 0);
        let mut blocks = HashMap::new();
        blocks.insert(pos.north(1), IBlockState::fromGlobalStateId(139 << 4));
        blocks.insert(pos.south(1), IBlockState::fromGlobalStateId(139 << 4));
        let mask = connectionMask(&Access(blocks), pos);
        assert_eq!(mask & 16, 0);
        let box_ = getCollisionBoxes(mask)[0];
        assert_eq!((box_.min_x, box_.max_x), (0.3125, 0.6875));
    }

    #[test]
    fn straight_wall_selected_box_is_not_the_collision_clip_box() {
        let box_ = getBoundingBox(1 | 4);
        assert_eq!(box_.max_y, 0.875);
        assert_eq!((box_.min_x, box_.max_x), (0.3125, 0.6875));
    }
}
