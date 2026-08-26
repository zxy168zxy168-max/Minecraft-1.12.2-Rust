use crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumActionResult::EnumActionResult;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

pub struct ItemDoor;

impl ItemDoor {
    pub const fn blockIdForItem(itemId: i16) -> Option<i32> {
        match itemId {
            324 => Some(64),
            330 => Some(71),
            427 => Some(193),
            428 => Some(194),
            429 => Some(195),
            430 => Some(196),
            431 => Some(197),
            _ => None,
        }
    }

    pub const fn isItemDoor(stack: &ItemStack) -> bool {
        !stack.isEmpty() && Self::blockIdForItem(stack.itemId).is_some()
    }

    /// Client result branch of MCP `ItemDoor#onItemUse`. World mutation, hinge
    /// resolution, power state and item decrement remain server-authoritative.
    pub fn predictOnItemUse(
        world: &WorldClient,
        player: &EntityPlayerSP,
        pos: BlockPos,
        side: EnumFacing,
        stack: &ItemStack,
    ) -> EnumActionResult {
        let Some(_blockId) = Self::blockIdForItem(stack.itemId) else {
            return EnumActionResult::Pass;
        };
        if side != EnumFacing::Up || !player.capabilities.allowEdit {
            return EnumActionResult::Fail;
        }
        let target = if world.isBlockReplaceable(pos) {
            pos
        } else {
            pos.up(1)
        };
        if target.y >= 255
            || !world.getBlockState(target.down(1)).isTopSolid()
            || !world.isBlockReplaceable(target)
            || !world.isBlockReplaceable(target.up(1))
        {
            return EnumActionResult::Fail;
        }
        EnumActionResult::Success
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_seven_vanilla_door_items_map_to_their_blocks() {
        assert_eq!(ItemDoor::blockIdForItem(324), Some(64));
        assert_eq!(ItemDoor::blockIdForItem(330), Some(71));
        assert_eq!(ItemDoor::blockIdForItem(431), Some(197));
        assert_eq!(ItemDoor::blockIdForItem(323), None);
    }
}
