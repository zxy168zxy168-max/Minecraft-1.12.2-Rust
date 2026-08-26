use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

/// Client-visible actual-state contract of MCP 1.12.2 `BlockFire`.
/// The five attachment booleans are not encoded in protocol metadata; they
/// are reconstructed from neighbouring flammable blocks at render time.
pub struct BlockFire;

impl BlockFire {
    pub const BLOCK_ID: i32 = 51;

    pub const fn isBlockFire(state: IBlockState) -> bool {
        state.getBlockId() == Self::BLOCK_ID
    }

    /// Blocks registered by `BlockFire#init` with non-zero encouragement.
    pub const fn canCatchFire(state: IBlockState) -> bool {
        matches!(
            state.getBlockId(),
            5 | 17
                | 18
                | 31
                | 32
                | 35
                | 37
                | 38
                | 46
                | 47
                | 53
                | 85
                | 106
                | 107
                | 125
                | 126
                | 134
                | 135
                | 136
                | 161
                | 162
                | 163
                | 164
                | 170
                | 171
                | 173
                | 175
                | 183..=192
        )
    }

    /// Bit order used by the Vulkan baked-model cache: north, east, south,
    /// west, upper. This is the exact predicate in `BlockFire#getActualState`.
    pub fn actualStateMask(world: &impl IBlockAccess, pos: BlockPos) -> u8 {
        let down = world.getBlockState(pos.down(1));
        if down.isTopSolid() || Self::canCatchFire(down) {
            return 0;
        }
        let mut mask = 0_u8;
        if Self::canCatchFire(world.getBlockState(pos.north(1))) {
            mask |= 1;
        }
        if Self::canCatchFire(world.getBlockState(pos.east(1))) {
            mask |= 2;
        }
        if Self::canCatchFire(world.getBlockState(pos.south(1))) {
            mask |= 4;
        }
        if Self::canCatchFire(world.getBlockState(pos.west(1))) {
            mask |= 8;
        }
        if Self::canCatchFire(world.getBlockState(pos.up(1))) {
            mask |= 16;
        }
        mask
    }

    pub fn modelVariant(age: i32, mask: u8) -> String {
        let north = mask & 1 != 0;
        let east = mask & 2 != 0;
        let south = mask & 4 != 0;
        let west = mask & 8 != 0;
        let upper = mask & 16 != 0;
        format!(
            "age={},east={},north={},south={},up={},west={}",
            age.clamp(0, 15),
            east,
            north,
            south,
            upper,
            west,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flammability_registry_matches_vanilla_examples() {
        assert!(BlockFire::canCatchFire(IBlockState::fromGlobalStateId(
            5 << 4
        )));
        assert!(BlockFire::canCatchFire(IBlockState::fromGlobalStateId(
            46 << 4
        )));
        assert!(!BlockFire::canCatchFire(IBlockState::fromGlobalStateId(
            1 << 4
        )));
    }
}
