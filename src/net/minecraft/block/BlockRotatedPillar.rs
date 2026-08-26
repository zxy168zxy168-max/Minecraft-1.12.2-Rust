use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::EnumFacing::{Axis, EnumFacing};

/// Vanilla `BlockRotatedPillar` placement families whose base metadata is only
/// the AXIS property. Logs keep their separate species bits in `BlockLog`.
pub const fn isSimpleRotatedPillar(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 170 | 202 | 216)
}

/// MCP `BlockRotatedPillar#onBlockPlaced`/`getMetaFromState`.
pub const fn onBlockPlacedState(blockId: i32, facing: EnumFacing) -> IBlockState {
    let axisBits = match facing.axis() {
        Axis::X => 4,
        Axis::Z => 8,
        Axis::Y => 0,
    };
    IBlockState::fromGlobalStateId((blockId << 4) | axisBits)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn axis_bits_match_mcp() {
        assert_eq!(onBlockPlacedState(170, EnumFacing::Up).getMetadata(), 0);
        assert_eq!(onBlockPlacedState(170, EnumFacing::East).getMetadata(), 4);
        assert_eq!(onBlockPlacedState(170, EnumFacing::North).getMetadata(), 8);
    }
}
