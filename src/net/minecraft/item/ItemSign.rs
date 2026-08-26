use crate::net::minecraft::block::BlockSign;
use crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumActionResult::EnumActionResult;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

/// MCP `ItemSign` client-side result path. Vanilla returns SUCCESS on the
/// remote world before mutating a block; the server later places the sign,
/// creates `TileEntitySign`, and sends `SPacketSignEditorOpen`.
pub struct ItemSign;

impl ItemSign {
    pub const ITEM_ID: i16 = 323;

    pub const fn isItemSign(stack: &ItemStack) -> bool {
        !stack.isEmpty() && stack.itemId == Self::ITEM_ID
    }

    pub fn predictOnItemUse(
        world: &WorldClient,
        player: &EntityPlayerSP,
        pos: BlockPos,
        side: EnumFacing,
        stack: &ItemStack,
    ) -> EnumActionResult {
        if !Self::isItemSign(stack) {
            return EnumActionResult::Pass;
        }

        let clicked = world.getBlockState(pos);
        let replaceable = world.isBlockReplaceable(pos);
        if side == EnumFacing::Down
            || (!clicked.getBlock().materialIsSolid() && !replaceable)
            || (replaceable && side != EnumFacing::Up)
        {
            return EnumActionResult::Fail;
        }

        let target = pos.offset(side, 1);
        if !player.capabilities.allowEdit || !BlockSign::canPlaceBlockAt(world, target) {
            return EnumActionResult::Fail;
        }

        // Exact `if (worldIn.isRemote) return SUCCESS` branch.
        EnumActionResult::Success
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::block::state::IBlockState::IBlockState;

    fn stack() -> ItemStack {
        ItemStack {
            itemId: 323,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        }
    }

    #[test]
    fn downward_face_is_rejected() {
        let mut world = WorldClient::new(0);
        let pos = BlockPos::new(0, 64, 0);
        world
            .invalidateRegionAndSetBlock(pos, IBlockState::fromGlobalStateId(1 << 4))
            .unwrap();
        assert_eq!(
            ItemSign::predictOnItemUse(
                &world,
                &EntityPlayerSP::new(1),
                pos,
                EnumFacing::Down,
                &stack()
            ),
            EnumActionResult::Fail,
        );
    }

    #[test]
    fn solid_clicked_block_and_empty_target_succeed_client_side() {
        let mut world = WorldClient::new(0);
        let pos = BlockPos::new(0, 64, 0);
        world
            .invalidateRegionAndSetBlock(pos, IBlockState::fromGlobalStateId(1 << 4))
            .unwrap();
        assert_eq!(
            ItemSign::predictOnItemUse(
                &world,
                &EntityPlayerSP::new(1),
                pos,
                EnumFacing::Up,
                &stack()
            ),
            EnumActionResult::Success,
        );
    }
}
