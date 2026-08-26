use crate::net::minecraft::block::state::BlockFaceShape::BlockFaceShape;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockStairs;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::math::Vec3d::Vec3d;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

/// Rust identity for the two MCP `MaterialLiquid` instances used by
/// `BlockLiquid`.  It is intentionally material-based: flowing and static
/// blocks of one liquid must compare equal throughout rendering and physics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LiquidMaterial {
    Water,
    Lava,
}

impl LiquidMaterial {
    pub const fn fromState(state: IBlockState) -> Option<Self> {
        match state.getBlockId() {
            8 | 9 => Some(Self::Water),
            10 | 11 => Some(Self::Lava),
            _ => None,
        }
    }

    pub const fn contains(self, state: IBlockState) -> bool {
        matches!(
            (self, state.getBlockId()),
            (Self::Water, 8 | 9) | (Self::Lava, 10 | 11)
        )
    }
}

pub const fn isLiquid(state: IBlockState) -> bool {
    LiquidMaterial::fromState(state).is_some()
}

pub const fn getLevel(state: IBlockState) -> i32 {
    state.getMetadata() & 15
}

/// Direct port of `BlockLiquid#getLiquidHeightPercent`.
pub fn getLiquidHeightPercent(mut level: i32) -> f32 {
    if level >= 8 {
        level = 0;
    }
    (level + 1) as f32 / 9.0
}

pub fn getDepth(state: IBlockState, material: LiquidMaterial) -> i32 {
    if material.contains(state) {
        getLevel(state)
    } else {
        -1
    }
}

pub fn getRenderedDepth(state: IBlockState, material: LiquidMaterial) -> i32 {
    let depth = getDepth(state, material);
    if depth >= 8 {
        0
    } else {
        depth
    }
}

/// `Material#blocksMovement` bridge needed by the flow-gradient calculation.
/// Every non-solid material already represented by `Block#materialIsSolid`
/// also does not block movement, with `Material.WEB` as the one vanilla
/// anonymous-material exception.
pub fn materialBlocksMovement(state: IBlockState) -> bool {
    state.getBlock().materialBlocksMovement()
}

fn isBlockSolid<A: IBlockAccess>(
    world: &A,
    pos: BlockPos,
    side: EnumFacing,
    material: LiquidMaterial,
) -> bool {
    let state = world.getBlockState(pos);
    if material.contains(state) {
        return false;
    }
    if side == EnumFacing::Up {
        return true;
    }
    // MCP only exempts Material.ICE here; packed ice is a distinct material.
    if matches!(state.getBlockId(), 79 | 212) {
        return false;
    }
    let block = state.getBlock();
    let excluded = block.func_193382_c() || BlockStairs::isBlockStairs(state);
    !excluded && state.getBlockFaceShape(world, pos, side) == BlockFaceShape::SOLID
}

/// Direct port of `BlockLiquid#getFlow` over the protocol-global state IDs.
pub fn getFlow<A: IBlockAccess>(world: &A, pos: BlockPos, state: IBlockState) -> Vec3d {
    let Some(material) = LiquidMaterial::fromState(state) else {
        return Vec3d::ZERO;
    };
    let mut x = 0.0_f64;
    let mut y = 0.0_f64;
    let mut z = 0.0_f64;
    let current = getRenderedDepth(state, material);

    for facing in [
        EnumFacing::North,
        EnumFacing::South,
        EnumFacing::West,
        EnumFacing::East,
    ] {
        let neighbour_pos = pos.offset(facing, 1);
        let neighbour_state = world.getBlockState(neighbour_pos);
        let mut neighbour = getRenderedDepth(neighbour_state, material);
        let (dx, dy, dz) = facing.offsets();
        if neighbour < 0 {
            if !materialBlocksMovement(neighbour_state) {
                neighbour = getRenderedDepth(world.getBlockState(neighbour_pos.down(1)), material);
                if neighbour >= 0 {
                    let difference = neighbour - (current - 8);
                    x += (dx * difference) as f64;
                    y += (dy * difference) as f64;
                    z += (dz * difference) as f64;
                }
            }
        } else {
            let difference = neighbour - current;
            x += (dx * difference) as f64;
            y += (dy * difference) as f64;
            z += (dz * difference) as f64;
        }
    }

    let mut flow = Vec3d::new(x, y, z);
    if getLevel(state) >= 8 {
        for facing in [
            EnumFacing::North,
            EnumFacing::South,
            EnumFacing::West,
            EnumFacing::East,
        ] {
            let neighbour = pos.offset(facing, 1);
            if isBlockSolid(world, neighbour, facing, material)
                || isBlockSolid(world, neighbour.up(1), facing, material)
            {
                flow = flow.normalize().add_vector(0.0, -6.0, 0.0);
                break;
            }
        }
    }
    flow.normalize()
}

/// Direct port of `BlockLiquid#getSlopeAngle`.
pub fn getSlopeAngle<A: IBlockAccess>(world: &A, pos: BlockPos, state: IBlockState) -> f32 {
    let flow = getFlow(world, pos, state);
    if flow.x == 0.0 && flow.z == 0.0 {
        -1000.0
    } else {
        flow.z.atan2(flow.x) as f32 - std::f32::consts::FRAC_PI_2
    }
}

/// Direct port of `BlockLiquid#func_190973_f`, the occupied fraction used for
/// eye-submersion and fluid-volume tests.
pub fn getFilledPercentage<A: IBlockAccess>(state: IBlockState, world: &A, pos: BlockPos) -> f32 {
    let level = getLevel(state);
    if level & 7 == 0
        && LiquidMaterial::fromState(state) == Some(LiquidMaterial::Water)
        && LiquidMaterial::Water.contains(world.getBlockState(pos.up(1)))
    {
        1.0
    } else {
        1.0 - getLiquidHeightPercent(level)
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
    fn liquid_height_percent_resets_falling_levels() {
        assert!((getLiquidHeightPercent(0) - 1.0 / 9.0).abs() < f32::EPSILON);
        assert!((getLiquidHeightPercent(7) - 8.0 / 9.0).abs() < f32::EPSILON);
        assert!((getLiquidHeightPercent(8) - 1.0 / 9.0).abs() < f32::EPSILON);
    }

    #[test]
    fn flowing_water_points_toward_greater_rendered_depth() {
        let origin = BlockPos::new(0, 64, 0);
        let mut states = HashMap::new();
        states.insert(origin, IBlockState::fromGlobalStateId((8 << 4) | 1));
        states.insert(origin.east(1), IBlockState::fromGlobalStateId((8 << 4) | 4));
        let flow = getFlow(
            &Access(states),
            origin,
            IBlockState::fromGlobalStateId((8 << 4) | 1),
        );
        assert!(flow.x > 0.99);
        assert!(flow.z.abs() < 1.0e-6);
    }

    #[test]
    fn full_water_column_reports_full_occupied_height() {
        let origin = BlockPos::new(0, 64, 0);
        let source = IBlockState::fromGlobalStateId(9 << 4);
        let mut states = HashMap::new();
        states.insert(origin, source);
        states.insert(origin.up(1), source);
        assert_eq!(getFilledPercentage(source, &Access(states), origin), 1.0);
    }
}
