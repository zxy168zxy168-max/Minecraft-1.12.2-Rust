use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

pub const fn isBlockButton(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 77 | 143)
}

/// Exact `BlockButton#getStateFromMeta` facing decode.
pub const fn facing(state: IBlockState) -> EnumFacing {
    match state.getMetadata() & 7 {
        0 => EnumFacing::Down,
        1 => EnumFacing::East,
        2 => EnumFacing::West,
        3 => EnumFacing::South,
        4 => EnumFacing::North,
        _ => EnumFacing::Up,
    }
}

pub const fn isPowered(state: IBlockState) -> bool { state.getMetadata() & 8 != 0 }
/// MCP `BlockButton#canPlaceBlock`, including the 1.12.2 face-shape
/// exceptions used by buttons and `BlockLever`.
pub fn canPlaceBlock<A: IBlockAccess>(world:&A,pos:BlockPos,direction:EnumFacing)->bool {
    let supportPos=pos.offset(direction.opposite(),1);
    let support=world.getBlockState(supportPos);
    let solid=support.getBlockFaceShape(world,supportPos,direction)==crate::net::minecraft::block::state::BlockFaceShape::BlockFaceShape::SOLID;
    let id=support.getBlockId();
    let blockedTop=matches!(id,18|161|96|167|138|118|20|89|79|169|95|219..=234);
    let blockedSide=blockedTop||matches!(id,29|33|34);
    if direction==EnumFacing::Up { id==154 || (!blockedTop&&solid) } else { !blockedSide&&solid }
}

/// MCP `BlockButton#onBlockPlaced`; POWERED is false at placement.
pub fn onBlockPlacedState<A:IBlockAccess>(blockId:i32,world:&A,pos:BlockPos,facingIn:EnumFacing)->IBlockState{
    let facing=if canPlaceBlock(world,pos,facingIn){facingIn}else{EnumFacing::Down};
    let meta=match facing{EnumFacing::Down=>0,EnumFacing::East=>1,EnumFacing::West=>2,EnumFacing::South=>3,EnumFacing::North=>4,EnumFacing::Up=>5};
    IBlockState::fromGlobalStateId((blockId<<4)|meta)
}


/// Exact state mutation in `BlockButton#onBlockActivated`. An already
/// powered button still consumes the click but does not change state.
pub fn onBlockActivatedState(state: IBlockState) -> Option<IBlockState> {
    if !isBlockButton(state) || isPowered(state) {
        return None;
    }
    Some(IBlockState::fromGlobalStateId(
        (state.getBlockId() << 4) | (state.getMetadata() | 8),
    ))
}

/// Exact local-space result of `BlockButton#getBoundingBox`, including the
/// depressed 1/16-depth powered forms.
pub fn getBoundingBox(state: IBlockState) -> AxisAlignedBB {
    let powered = isPowered(state);
    match facing(state) {
        EnumFacing::East => {
            if powered {
                AxisAlignedBB::new(0.0, 0.375, 0.3125, 0.0625, 0.625, 0.6875)
            } else {
                AxisAlignedBB::new(0.0, 0.375, 0.3125, 0.125, 0.625, 0.6875)
            }
        }
        EnumFacing::West => {
            if powered {
                AxisAlignedBB::new(0.9375, 0.375, 0.3125, 1.0, 0.625, 0.6875)
            } else {
                AxisAlignedBB::new(0.875, 0.375, 0.3125, 1.0, 0.625, 0.6875)
            }
        }
        EnumFacing::South => {
            if powered {
                AxisAlignedBB::new(0.3125, 0.375, 0.0, 0.6875, 0.625, 0.0625)
            } else {
                AxisAlignedBB::new(0.3125, 0.375, 0.0, 0.6875, 0.625, 0.125)
            }
        }
        EnumFacing::North => {
            if powered {
                AxisAlignedBB::new(0.3125, 0.375, 0.9375, 0.6875, 0.625, 1.0)
            } else {
                AxisAlignedBB::new(0.3125, 0.375, 0.875, 0.6875, 0.625, 1.0)
            }
        }
        EnumFacing::Up => {
            if powered {
                AxisAlignedBB::new(0.3125, 0.0, 0.375, 0.6875, 0.0625, 0.625)
            } else {
                AxisAlignedBB::new(0.3125, 0.0, 0.375, 0.6875, 0.125, 0.625)
            }
        }
        EnumFacing::Down => {
            if powered {
                AxisAlignedBB::new(0.3125, 0.9375, 0.375, 0.6875, 1.0, 0.625)
            } else {
                AxisAlignedBB::new(0.3125, 0.875, 0.375, 0.6875, 1.0, 0.625)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn powered_button_uses_depressed_bounds() {
        let off = IBlockState::fromGlobalStateId((77 << 4) | 4);
        let on = IBlockState::fromGlobalStateId((77 << 4) | 12);
        assert_eq!(facing(off), EnumFacing::North);
        assert_eq!(getBoundingBox(off).min_z, 0.875);
        assert_eq!(getBoundingBox(on).min_z, 0.9375);
    }
}
