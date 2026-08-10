use crate::net::minecraft::block::state::BlockFaceShape::BlockFaceShape;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

pub const fn isBlockVine(state: IBlockState) -> bool { state.getBlockId() == 106 }
pub const fn south(state: IBlockState) -> bool { state.getMetadata() & 1 != 0 }
pub const fn west(state: IBlockState) -> bool { state.getMetadata() & 2 != 0 }
pub const fn north(state: IBlockState) -> bool { state.getMetadata() & 4 != 0 }
pub const fn east(state: IBlockState) -> bool { state.getMetadata() & 8 != 0 }
fn prohibitedSupport(id:i32)->bool{matches!(id,219..=234|138|118|20|95|29|33|34|96|167)}
fn canAttach<A:IBlockAccess>(world:&A,pos:BlockPos,direction:EnumFacing)->bool{
    let state=world.getBlockState(pos);
    state.getBlockFaceShape(world,pos,direction)==BlockFaceShape::SOLID&&!prohibitedSupport(state.getBlockId())
}
/// MCP `BlockVine#canPlaceBlockOnSide`/`func_193395_a`.
pub fn canPlaceBlockOnSide<A:IBlockAccess>(world:&A,pos:BlockPos,side:EnumFacing)->bool{
    if matches!(side,EnumFacing::Down|EnumFacing::Up){return false;}
    let above=world.getBlockState(pos.up(1));
    canAttach(world,pos.offset(side.opposite(),1),side)&&(above.isAir()||above.getBlockId()==106||canAttach(world,pos.up(1),EnumFacing::Up))
}
/// MCP `BlockVine#onBlockPlaced` metadata form.
pub fn onBlockPlacedState(side:EnumFacing)->IBlockState{
    let meta=match side{EnumFacing::North=>1,EnumFacing::South=>4,EnumFacing::West=>8,EnumFacing::East=>2,_=>0};
    IBlockState::fromGlobalStateId((106<<4)|meta)
}


/// MCP `BlockVine#getActualState`: UP is derived from the lower face of the
/// block above and is not serialized in legacy metadata.
pub fn up<A: IBlockAccess>(world: &A, pos: BlockPos) -> bool {
    let above = pos.up(1);
    world.getBlockState(above).getBlockFaceShape(world, above, EnumFacing::Down)
        == BlockFaceShape::SOLID
}

/// Exact single-face selection boxes from `BlockVine#getBoundingBox`. Vanilla
/// returns FULL_BLOCK_AABB when more than one face is present.
pub fn getBoundingBox<A: IBlockAccess>(state: IBlockState, world: &A, pos: BlockPos) -> AxisAlignedBB {
    let mut count = 0;
    let mut bounds = AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
    if up(world, pos) {
        bounds = AxisAlignedBB::new(0.0, 0.9375, 0.0, 1.0, 1.0, 1.0);
        count += 1;
    }
    if north(state) {
        bounds = AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 0.0625);
        count += 1;
    }
    if east(state) {
        bounds = AxisAlignedBB::new(0.9375, 0.0, 0.0, 1.0, 1.0, 1.0);
        count += 1;
    }
    if south(state) {
        bounds = AxisAlignedBB::new(0.0, 0.0, 0.9375, 1.0, 1.0, 1.0);
        count += 1;
    }
    if west(state) {
        bounds = AxisAlignedBB::new(0.0, 0.0, 0.0, 0.0625, 1.0, 1.0);
        count += 1;
    }
    if count == 1 { bounds } else { AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0) }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Access;
    impl IBlockAccess for Access {
        fn getBlockState(&self, _pos: BlockPos) -> IBlockState { IBlockState::default() }
    }
    #[test]
    fn single_north_face_uses_thin_north_plane() {
        let state = IBlockState::fromGlobalStateId((106 << 4) | 4);
        let bounds = getBoundingBox(state, &Access, BlockPos::ORIGIN);
        assert_eq!((bounds.min_z, bounds.max_z), (0.0, 0.0625));
    }
}
