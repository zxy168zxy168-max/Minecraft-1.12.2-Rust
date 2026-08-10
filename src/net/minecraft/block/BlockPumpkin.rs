use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
pub const fn isBlockPumpkin(state:IBlockState)->bool{matches!(state.getBlockId(),86|91)}
/// MCP `BlockPumpkin#onBlockPlaced`: face opposite the placer's horizontal facing.
pub fn onBlockPlacedState(blockId:i32,placerYaw:f32)->IBlockState{let facing=EnumFacing::fromAngle(placerYaw as f64).opposite();IBlockState::fromGlobalStateId((blockId<<4)|facing.horizontalIndex().unwrap_or(2) as i32)}
