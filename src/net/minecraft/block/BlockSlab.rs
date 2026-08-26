use crate::net::minecraft::block::state::BlockFaceShape::BlockFaceShape;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

pub const fn isBlockSlab(state: IBlockState) -> bool {
    matches!(
        state.getBlockId(),
        43 | 44 | 125 | 126 | 181 | 182 | 204 | 205
    )
}

pub const fn isDouble(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 43 | 125 | 181 | 204)
}

pub const fn isTop(state: IBlockState) -> bool {
    !isDouble(state) && state.getMetadata() & 8 != 0
}

/// Port of `BlockSlab.func_193383_a` / `getBlockFaceShape`.
pub fn getBlockFaceShape(state: IBlockState, face: EnumFacing) -> BlockFaceShape {
    if isDouble(state)
        || (face == EnumFacing::Up && isTop(state))
        || (face == EnumFacing::Down && !isTop(state))
    {
        BlockFaceShape::SOLID
    } else {
        BlockFaceShape::UNDEFINED
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slab_solid_face_matches_half() {
        let bottom = IBlockState::fromGlobalStateId((44 << 4) | 0);
        let top = IBlockState::fromGlobalStateId((44 << 4) | 8);
        assert_eq!(
            getBlockFaceShape(bottom, EnumFacing::Down),
            BlockFaceShape::SOLID
        );
        assert_eq!(
            getBlockFaceShape(bottom, EnumFacing::Up),
            BlockFaceShape::UNDEFINED
        );
        assert_eq!(
            getBlockFaceShape(top, EnumFacing::Up),
            BlockFaceShape::SOLID
        );
    }
}
