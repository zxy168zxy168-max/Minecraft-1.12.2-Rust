use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

/// Client-visible state helpers from Minecraft 1.12.2 `BlockBed`.
pub struct BlockBed;

impl BlockBed {
    pub const BLOCK_ID: i32 = 26;
    pub const OCCUPIED_MASK: i32 = 0x4;
    pub const HEAD_MASK: i32 = 0x8;

    pub const fn isBlockBed(state: IBlockState) -> bool {
        state.getBlockId() == Self::BLOCK_ID
    }

    /// `BlockBed#getStateFromMeta`: horizontal indices are S-W-N-E.
    pub fn getFacing(state: IBlockState) -> EnumFacing {
        EnumFacing::getHorizontal(state.getMetadata() & 3)
    }

    pub const fn isHead(state: IBlockState) -> bool {
        (state.getMetadata() & Self::HEAD_MASK) != 0
    }

    pub const fn isOccupied(state: IBlockState) -> bool {
        (state.getMetadata() & Self::OCCUPIED_MASK) != 0
    }

    pub fn orientationDegrees(state: IBlockState) -> f32 {
        match Self::getFacing(state) {
            EnumFacing::South => 90.0,
            EnumFacing::West => 0.0,
            EnumFacing::North => 270.0,
            EnumFacing::East => 180.0,
            _ => 0.0,
        }
    }

    /// MCP `BlockBed#getSafeExitLocation` and `hasRoomForPlayer`.
    pub fn getSafeExitLocation<A: IBlockAccess>(
        world: &A,
        pos: BlockPos,
        mut tries: i32,
    ) -> Option<BlockPos> {
        let facing = Self::getFacing(world.getBlockState(pos));
        let (front_x, _, front_z) = facing.offsets();
        for layer in 0..=1 {
            let min_x = pos.x - front_x * layer - 1;
            let min_z = pos.z - front_z * layer - 1;
            for x in min_x..=min_x + 2 {
                for z in min_z..=min_z + 2 {
                    let candidate = BlockPos::new(x, pos.y, z);
                    if Self::hasRoomForPlayer(world, candidate) {
                        if tries <= 0 {
                            return Some(candidate);
                        }
                        tries -= 1;
                    }
                }
            }
        }
        None
    }

    pub fn hasRoomForPlayer<A: IBlockAccess>(world: &A, pos: BlockPos) -> bool {
        world.getBlockState(pos.down(1)).isTopSolid()
            && !world.getBlockState(pos).getBlock().materialIsSolid()
            && !world.getBlockState(pos.up(1)).getBlock().materialIsSolid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_uses_vanilla_horizontal_order() {
        for (meta, facing) in [
            (0, EnumFacing::South),
            (1, EnumFacing::West),
            (2, EnumFacing::North),
            (3, EnumFacing::East),
        ] {
            let state = IBlockState::fromGlobalStateId((BlockBed::BLOCK_ID << 4) | meta);
            assert_eq!(BlockBed::getFacing(state), facing);
        }
    }
}
