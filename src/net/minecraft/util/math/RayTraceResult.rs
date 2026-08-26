use super::BlockPos::BlockPos;
use super::EnumFacing;
use super::Vec3d::Vec3d;

/// Rust port of MCP 1.12.2 `RayTraceResult` for block and miss hits.
/// Entity hits are added with the multiplayer entity subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Miss,
    Block,
    Entity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayTraceResult {
    pub typeOfHit: Type,
    pub sideHit: EnumFacing,
    pub hitVec: Vec3d,
    blockPos: BlockPos,
}

impl RayTraceResult {
    pub const fn block(hitVecIn: Vec3d, sideHitIn: EnumFacing, blockPosIn: BlockPos) -> Self {
        Self {
            typeOfHit: Type::Block,
            sideHit: sideHitIn,
            hitVec: hitVecIn,
            blockPos: blockPosIn,
        }
    }

    pub const fn miss(hitVecIn: Vec3d, sideHitIn: EnumFacing, blockPosIn: BlockPos) -> Self {
        Self {
            typeOfHit: Type::Miss,
            sideHit: sideHitIn,
            hitVec: hitVecIn,
            blockPos: blockPosIn,
        }
    }

    pub const fn getBlockPos(self) -> BlockPos {
        self.blockPos
    }
}
