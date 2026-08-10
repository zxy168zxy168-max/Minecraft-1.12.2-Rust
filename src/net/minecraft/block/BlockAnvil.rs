use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

pub const fn isBlockAnvil(state:IBlockState)->bool{state.getBlockId()==145}

/// MCP `ItemAnvilBlock#getMetadata` followed by `BlockAnvil#onBlockPlaced`.
/// Invalid item damage follows the source catch branch and becomes intact.
pub fn onBlockPlacedState(placerYaw:f32,itemDamage:i16)->IBlockState{
    let facing=EnumFacing::fromAngle(placerYaw as f64).rotateY();
    let facingBits=facing.horizontalIndex().unwrap_or(0) as i32;
    let damage=if (0..=2).contains(&(itemDamage as i32)){itemDamage as i32}else{0};
    IBlockState::fromGlobalStateId((145<<4)|facingBits|(damage<<2))
}

#[cfg(test)]mod tests{use super::*;#[test]fn damage_and_rotated_facing_match_mcp(){let state=onBlockPlacedState(0.0,2);assert_eq!(state.getMetadata()>>2,2);assert_eq!(state.getMetadata()&3,3);assert_eq!(onBlockPlacedState(0.0,9).getMetadata()>>2,0);}}
