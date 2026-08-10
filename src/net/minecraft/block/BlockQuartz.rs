use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::EnumFacing::{Axis,EnumFacing};

pub const fn isBlockQuartz(state:IBlockState)->bool{state.getBlockId()==155}

/// MCP `BlockQuartz#onBlockPlaced`. Item metadata 2 is the Y-line pillar and
/// rotates into X/Z variants from the clicked face axis; only chiseled (1) and
/// the line item (2) survive as item metadata.
pub const fn onBlockPlacedState(facing:EnumFacing,itemMeta:i32)->IBlockState{
    let meta=if itemMeta==2{match facing.axis(){Axis::Z=>4,Axis::X=>3,Axis::Y=>2}}else if itemMeta==1{1}else{0};
    IBlockState::fromGlobalStateId((155<<4)|meta)
}

#[cfg(test)]mod tests{use super::*;#[test]fn line_quartz_rotates_from_face_axis(){assert_eq!(onBlockPlacedState(EnumFacing::Up,2).getMetadata(),2);assert_eq!(onBlockPlacedState(EnumFacing::East,2).getMetadata(),3);assert_eq!(onBlockPlacedState(EnumFacing::North,2).getMetadata(),4);}}
