use crate::net::minecraft::block::state::BlockFaceShape::BlockFaceShape;
use crate::net::minecraft::block::Block::Block;
use crate::net::minecraft::block::{
    BlockFence, BlockFenceGate, BlockPane, BlockSlab, BlockStairs, BlockWall,
};
use crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

/// Compact protocol identity for an MCP `IBlockState`. Concrete property
/// containers are ported incrementally; the protocol-global ID remains exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IBlockState {
    globalStateId: i32,
}
impl IBlockState {
    pub const fn fromGlobalStateId(globalStateId: i32) -> Self {
        Self {
            globalStateId: if globalStateId < 0 { 0 } else { globalStateId },
        }
    }
    pub const fn getGlobalStateId(self) -> i32 {
        self.globalStateId
    }
    pub const fn getBlockId(self) -> i32 {
        self.globalStateId >> 4
    }
    pub const fn getMetadata(self) -> i32 {
        self.globalStateId & 15
    }
    pub fn getBlock(self) -> Block {
        Block::getBlockById(self.getBlockId())
    }
    pub const fn isAir(self) -> bool {
        self.globalStateId == 0
    }
    pub fn getLightOpacity(self) -> i32 {
        self.getBlock().getLightOpacity()
    }

    /// Direct MCP `IBlockState#isTopSolid` delegation.
    pub fn isTopSolid(self) -> bool {
        self.getBlock().isFullyOpaque(self)
    }

    /// MCP `IBlockState.getPlayerRelativeBlockHardness`, delegating the
    /// player's exact dig-speed and harvest checks through `Block`.
    pub fn getPlayerRelativeBlockHardness(
        self,
        world: &WorldClient,
        player: &EntityPlayerSP,
    ) -> f32 {
        let hardness = self.getBlock().getBlockHardness();
        if hardness < 0.0 {
            0.0
        } else if player.canHarvestBlock(self) {
            player.getDigSpeed(world, self) / hardness / 30.0
        } else {
            player.getDigSpeed(world, self) / hardness / 100.0
        }
    }

    /// Delegation point corresponding to MCP `IBlockState#getBlockFaceShape`.
    /// Concrete source-confirmed overrides used by fence/pane/wall actual-state
    /// resolution are dispatched to their ported block classes.
    pub fn getBlockFaceShape<A: IBlockAccess>(
        self,
        world: &A,
        pos: BlockPos,
        face: EnumFacing,
    ) -> BlockFaceShape {
        if self.isAir() {
            BlockFaceShape::UNDEFINED
        } else if BlockStairs::isBlockStairs(self) {
            BlockStairs::getBlockFaceShape(self, world, pos, face)
        } else if BlockSlab::isBlockSlab(self) {
            BlockSlab::getBlockFaceShape(self, face)
        } else if BlockFenceGate::isBlockFenceGate(self) {
            BlockFenceGate::getBlockFaceShape(self, face)
        } else if BlockFence::isBlockFence(self) {
            BlockFence::getBlockFaceShape(face)
        } else if BlockPane::isBlockPane(self) {
            BlockPane::getBlockFaceShape(face)
        } else if BlockWall::isBlockWall(self) {
            BlockWall::getBlockFaceShape(face)
        } else if self.getBlock().isOpaqueCube() || matches!(self.getBlockId(), 20 | 95 | 165) {
            // Default `Block#getBlockFaceShape` is SOLID. Glass/stained glass
            // and slime retain that default despite not being opaque cubes.
            BlockFaceShape::SOLID
        } else {
            // Unported special-shape block classes remain conservative rather
            // than being allowed to create false fence/pane connections.
            BlockFaceShape::UNDEFINED
        }
    }
}
