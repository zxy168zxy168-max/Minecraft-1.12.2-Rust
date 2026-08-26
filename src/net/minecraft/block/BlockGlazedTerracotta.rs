use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
pub const fn isBlockGlazedTerracotta(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 235..=250)
}
/// MCP `BlockGlazedTerracotta#onBlockPlaced`: FACING = horizontal opposite.
pub fn onBlockPlacedState(blockId: i32, placerYaw: f32) -> IBlockState {
    let facing = EnumFacing::fromAngle(placerYaw as f64).opposite();
    IBlockState::fromGlobalStateId((blockId << 4) | facing.horizontalIndex().unwrap_or(2) as i32)
}
