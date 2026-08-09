use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::util::EnumActionResult::EnumActionResult;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// Client-side subset of MCP 1.12.2 `ItemHoe`: the tillable check and the
/// local `ITEM_HOE_TILL` sound. World mutation and durability stay with the
/// authoritative server (`setBlock` guards them with `!world.isRemote`).
pub struct ItemHoe;

/// Till sound event name and its (volume, pitch).
pub const HOE_TILL_SOUND: (&str, f32, f32) = ("item.hoe.till", 1.0, 1.0);

impl ItemHoe {
    pub const fn isItemHoe(stack: &ItemStack) -> bool {
        matches!(stack.itemId, 290..=293)
    }

    /// MCP `ItemHoe#onItemUse` client branch: tills grass, grass path and
    /// plain dirt (coarse dirt becomes plain dirt) with air above, otherwise
    /// PASS so the off hand is still tried. Returns the local sound on success.
    pub fn predictOnItemUse(
        world: &WorldClient,
        pos: BlockPos,
        sideHit: EnumFacing,
        stack: &ItemStack,
    ) -> (EnumActionResult, Option<(&'static str, f32, f32)>) {
        if !Self::isItemHoe(stack) || sideHit == EnumFacing::Down {
            return (EnumActionResult::Pass, None);
        }
        // `hand != EnumFacing.DOWN && worldIn.getBlockState(pos.up()).getMaterial() == AIR`.
        if world.getBlockState(pos.up(1)).getBlockId() != 0 {
            return (EnumActionResult::Pass, None);
        }
        let target = world.getBlockState(pos);
        let tillable = match target.getBlockId() {
            2 | 208 => true, // grass / grass path -> farmland
            // Dirt: DIRT -> farmland, COARSE_DIRT -> dirt, podzol passes.
            3 => match target.getMetadata() & 3 {
                0 | 1 => true,
                _ => false,
            },
            _ => false,
        };
        if tillable {
            (EnumActionResult::Success, Some(HOE_TILL_SOUND))
        } else {
            (EnumActionResult::Pass, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack() -> ItemStack { ItemStack { itemId: 290, count: 1, itemDamage: 0, tagCompound: None } }

    #[test]
    fn hoe_tills_grass_plain_and_coarse_dirt_with_air_above() {
        let mut world = WorldClient::new(0);
        let pos = BlockPos::new(0, 60, 0);
        world.invalidateRegionAndSetBlock(pos, crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(2 << 4)).unwrap();
        let (result, sound) = ItemHoe::predictOnItemUse(&world, pos, EnumFacing::Up, &stack());
        assert_eq!(result, EnumActionResult::Success);
        assert_eq!(sound, Some(HOE_TILL_SOUND));

        // Plain dirt (variant 0) and coarse dirt (variant 1) both till.
        world.invalidateRegionAndSetBlock(pos, crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(3 << 4)).unwrap();
        assert_eq!(ItemHoe::predictOnItemUse(&world, pos, EnumFacing::Up, &stack()).0, EnumActionResult::Success);
        world.invalidateRegionAndSetBlock(pos, crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(3 << 4 | 1)).unwrap();
        assert_eq!(ItemHoe::predictOnItemUse(&world, pos, EnumFacing::Up, &stack()).0, EnumActionResult::Success);
        // Podzol (variant 2) passes.
        world.invalidateRegionAndSetBlock(pos, crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(3 << 4 | 2)).unwrap();
        assert_eq!(ItemHoe::predictOnItemUse(&world, pos, EnumFacing::Up, &stack()).0, EnumActionResult::Pass);
    }

    #[test]
    fn hoe_passes_covered_or_solid_targets() {
        let mut world = WorldClient::new(0);
        let pos = BlockPos::new(0, 60, 0);
        world.invalidateRegionAndSetBlock(pos, crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(2 << 4)).unwrap();
        world.invalidateRegionAndSetBlock(pos.up(1), crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(1 << 4)).unwrap();
        // Blocked above: PASS (vanilla returns PASS, the off hand is tried).
        assert_eq!(ItemHoe::predictOnItemUse(&world, pos, EnumFacing::Up, &stack()).0, EnumActionResult::Pass);
        // Down side: PASS even on grass.
        world.invalidateRegionAndSetBlock(pos.up(1), crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(0)).unwrap();
        assert_eq!(ItemHoe::predictOnItemUse(&world, pos, EnumFacing::Down, &stack()).0, EnumActionResult::Pass);
        // Stone: PASS.
        world.invalidateRegionAndSetBlock(pos, crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(1 << 4)).unwrap();
        assert_eq!(ItemHoe::predictOnItemUse(&world, pos, EnumFacing::Up, &stack()).0, EnumActionResult::Pass);
    }
}
