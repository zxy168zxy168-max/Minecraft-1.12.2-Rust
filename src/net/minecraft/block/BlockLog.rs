use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::EnumFacing::{Axis, EnumFacing};

/// MCP 1.12.2 `BlockLog` metadata/placement ownership shared by old/new logs.
pub const fn isBlockLog(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 17 | 162)
}

/// `BlockLog#onBlockPlaced`: preserve variant bits from item metadata and set
/// LOG_AXIS from the clicked face axis. In 1.12.2 X=4, Z=8, Y=0.
pub fn onBlockPlacedState(blockId: i32, itemMeta: i32, face: EnumFacing) -> IBlockState {
    let variant = match blockId {
        17 => itemMeta & 3,
        162 => itemMeta & 1,
        _ => itemMeta & 3,
    };
    let axisBits = match face.axis() {
        Axis::Y => 0,
        Axis::X => 4,
        Axis::Z => 8,
    };
    IBlockState::fromGlobalStateId((blockId << 4) | variant | axisBits)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn face_axis_matches_log_metadata() {
        assert_eq!(onBlockPlacedState(17, 2, EnumFacing::East).getMetadata(), 6);
        assert_eq!(
            onBlockPlacedState(17, 2, EnumFacing::North).getMetadata(),
            10
        );
        assert_eq!(onBlockPlacedState(162, 1, EnumFacing::Up).getMetadata(), 1);
    }
}
