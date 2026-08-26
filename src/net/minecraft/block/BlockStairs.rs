use crate::net::minecraft::block::state::BlockFaceShape::BlockFaceShape;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnumShape {
    Straight,
    InnerLeft,
    InnerRight,
    OuterLeft,
    OuterRight,
}

impl EnumShape {
    pub const VALUES: [Self; 5] = [
        Self::Straight,
        Self::InnerLeft,
        Self::InnerRight,
        Self::OuterLeft,
        Self::OuterRight,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Straight => "straight",
            Self::InnerLeft => "inner_left",
            Self::InnerRight => "inner_right",
            Self::OuterLeft => "outer_left",
            Self::OuterRight => "outer_right",
        }
    }
}

pub const fn isBlockStairs(state: IBlockState) -> bool {
    matches!(
        state.getBlockId(),
        53 | 67 | 108 | 109 | 114 | 128 | 134 | 135 | 136 | 156 | 163 | 164 | 180 | 203
    )
}

pub const fn facing(state: IBlockState) -> EnumFacing {
    match state.getMetadata() & 3 {
        0 => EnumFacing::East,
        1 => EnumFacing::West,
        2 => EnumFacing::South,
        _ => EnumFacing::North,
    }
}

pub const fn isTop(state: IBlockState) -> bool {
    state.getMetadata() & 4 != 0
}

/// MCP `BlockStairs#onBlockPlaced`: FACING follows the placer; HALF is top
/// only for a DOWN click or a horizontal click above 0.5. SHAPE is an
/// unpersisted actual/property state and starts STRAIGHT.
pub fn onBlockPlacedState(
    blockId: i32,
    clickedFace: EnumFacing,
    hitY: f32,
    placerYaw: f32,
) -> IBlockState {
    let facing = EnumFacing::fromAngle(placerYaw as f64);
    let facingBits = match facing {
        EnumFacing::East => 0,
        EnumFacing::West => 1,
        EnumFacing::South => 2,
        _ => 3,
    };
    let top =
        clickedFace == EnumFacing::Down || (clickedFace != EnumFacing::Up && (hitY as f64) > 0.5);
    IBlockState::fromGlobalStateId((blockId << 4) | facingBits | if top { 4 } else { 0 })
}

/// Port of `BlockStairs.func_193383_a` / `getBlockFaceShape`, including
/// neighbour-derived `SHAPE` through `getActualState`.
pub fn getBlockFaceShape<A: IBlockAccess>(
    state: IBlockState,
    world: &A,
    pos: BlockPos,
    face: EnumFacing,
) -> BlockFaceShape {
    if face.axis() == crate::net::minecraft::util::EnumFacing::Axis::Y {
        return if (face == EnumFacing::Up) == isTop(state) {
            BlockFaceShape::SOLID
        } else {
            BlockFaceShape::UNDEFINED
        };
    }

    let shape = getStairsShape(state, world, pos);
    if matches!(shape, EnumShape::OuterLeft | EnumShape::OuterRight) {
        return BlockFaceShape::UNDEFINED;
    }
    let stairFacing = facing(state);
    let solid = match shape {
        EnumShape::InnerRight => stairFacing == face || stairFacing == face.rotateYCCW(),
        EnumShape::InnerLeft => stairFacing == face || stairFacing == face.rotateY(),
        EnumShape::Straight => stairFacing == face,
        EnumShape::OuterLeft | EnumShape::OuterRight => false,
    };
    if solid {
        BlockFaceShape::SOLID
    } else {
        BlockFaceShape::UNDEFINED
    }
}

pub fn getCollisionBoxList(state: IBlockState, shape: EnumShape) -> Vec<AxisAlignedBB> {
    let top = isTop(state);
    let mut boxes = vec![if top {
        AxisAlignedBB::new(0.0, 0.5, 0.0, 1.0, 1.0, 1.0)
    } else {
        AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)
    }];

    if matches!(
        shape,
        EnumShape::Straight | EnumShape::InnerLeft | EnumShape::InnerRight
    ) {
        boxes.push(quarter_box(state));
    }
    if shape != EnumShape::Straight {
        boxes.push(eighth_box(state, shape));
    }
    boxes
}

fn quarter_box(state: IBlockState) -> AxisAlignedBB {
    let top = isTop(state);
    let (min_y, max_y) = if top { (0.0, 0.5) } else { (0.5, 1.0) };
    match facing(state) {
        EnumFacing::North => AxisAlignedBB::new(0.0, min_y, 0.0, 1.0, max_y, 0.5),
        EnumFacing::South => AxisAlignedBB::new(0.0, min_y, 0.5, 1.0, max_y, 1.0),
        EnumFacing::West => AxisAlignedBB::new(0.0, min_y, 0.0, 0.5, max_y, 1.0),
        EnumFacing::East => AxisAlignedBB::new(0.5, min_y, 0.0, 1.0, max_y, 1.0),
        _ => unreachable!("stair facing is horizontal"),
    }
}

fn eighth_box(state: IBlockState, shape: EnumShape) -> AxisAlignedBB {
    let stairFacing = facing(state);
    let cornerFacing = match shape {
        EnumShape::OuterLeft => stairFacing,
        EnumShape::OuterRight => stairFacing.rotateY(),
        EnumShape::InnerRight => stairFacing.opposite(),
        EnumShape::InnerLeft => stairFacing.rotateYCCW(),
        EnumShape::Straight => stairFacing,
    };
    let top = isTop(state);
    let (min_y, max_y) = if top { (0.0, 0.5) } else { (0.5, 1.0) };
    match cornerFacing {
        EnumFacing::North => AxisAlignedBB::new(0.0, min_y, 0.0, 0.5, max_y, 0.5),
        EnumFacing::South => AxisAlignedBB::new(0.5, min_y, 0.5, 1.0, max_y, 1.0),
        EnumFacing::West => AxisAlignedBB::new(0.0, min_y, 0.5, 0.5, max_y, 1.0),
        EnumFacing::East => AxisAlignedBB::new(0.5, min_y, 0.0, 1.0, max_y, 0.5),
        _ => unreachable!("stair corner facing is horizontal"),
    }
}

pub fn getStairsShape<A: IBlockAccess>(state: IBlockState, world: &A, pos: BlockPos) -> EnumShape {
    let stateFacing = facing(state);
    let front = offset(pos, stateFacing);
    let frontState = world.getBlockState(front);

    if isBlockStairs(frontState) && isTop(state) == isTop(frontState) {
        let frontFacing = facing(frontState);
        if frontFacing.axis() != stateFacing.axis()
            && isDifferentStairs(state, world, pos, frontFacing.opposite())
        {
            return if frontFacing == stateFacing.rotateYCCW() {
                EnumShape::OuterLeft
            } else {
                EnumShape::OuterRight
            };
        }
    }

    let back = offset(pos, stateFacing.opposite());
    let backState = world.getBlockState(back);
    if isBlockStairs(backState) && isTop(state) == isTop(backState) {
        let backFacing = facing(backState);
        if backFacing.axis() != stateFacing.axis()
            && isDifferentStairs(state, world, pos, backFacing)
        {
            return if backFacing == stateFacing.rotateYCCW() {
                EnumShape::InnerLeft
            } else {
                EnumShape::InnerRight
            };
        }
    }

    EnumShape::Straight
}

fn isDifferentStairs<A: IBlockAccess>(
    state: IBlockState,
    world: &A,
    pos: BlockPos,
    side: EnumFacing,
) -> bool {
    let neighbour = world.getBlockState(offset(pos, side));
    !isBlockStairs(neighbour)
        || facing(neighbour) != facing(state)
        || isTop(neighbour) != isTop(state)
}

const fn offset(pos: BlockPos, side: EnumFacing) -> BlockPos {
    let (x, y, z) = side.offsets();
    BlockPos::new(pos.x + x, pos.y + y, pos.z + z)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    struct Access(HashMap<BlockPos, IBlockState>);
    impl IBlockAccess for Access {
        fn getBlockState(&self, pos: BlockPos) -> IBlockState {
            self.0.get(&pos).copied().unwrap_or_default()
        }
    }

    fn stair(meta: i32) -> IBlockState {
        IBlockState::fromGlobalStateId((53 << 4) | meta)
    }

    #[test]
    fn straight_without_neighbour_stair() {
        assert_eq!(
            getStairsShape(stair(0), &Access(HashMap::new()), BlockPos::new(0, 0, 0)),
            EnumShape::Straight
        );
    }

    #[test]
    fn front_perpendicular_stair_forms_outer_left() {
        let pos = BlockPos::new(0, 64, 0);
        let mut blocks = HashMap::new();
        blocks.insert(pos.east(1), stair(3)); // north-facing stair in front of east-facing stair
        assert_eq!(
            getStairsShape(stair(0), &Access(blocks), pos),
            EnumShape::OuterLeft
        );
    }

    #[test]
    fn rear_perpendicular_stair_forms_inner_left() {
        let pos = BlockPos::new(0, 64, 0);
        let mut blocks = HashMap::new();
        blocks.insert(pos.west(1), stair(3)); // north-facing stair behind east-facing stair
        assert_eq!(
            getStairsShape(stair(0), &Access(blocks), pos),
            EnumShape::InnerLeft
        );
    }

    #[test]
    fn collision_boxes_follow_actual_shape() {
        assert_eq!(getCollisionBoxList(stair(0), EnumShape::Straight).len(), 2);
        assert_eq!(getCollisionBoxList(stair(0), EnumShape::OuterLeft).len(), 2);
        assert_eq!(getCollisionBoxList(stair(0), EnumShape::InnerLeft).len(), 3);
    }
}
