use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumActionResult::EnumActionResult;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

pub struct ItemHoe;
pub const HOE_TILL_SOUND: (&str, f32, f32) = ("item.hoe.till", 1.0, 1.0);

impl ItemHoe {
    pub const fn isItemHoe(stack: &ItemStack) -> bool {
        matches!(stack.itemId, 290..=294)
    }

    pub fn predictOnItemUse(
        world: &WorldClient,
        pos: BlockPos,
        sideHit: EnumFacing,
        stack: &ItemStack,
    ) -> (EnumActionResult, Option<(&'static str, f32, f32)>) {
        if !Self::isItemHoe(stack) || sideHit == EnumFacing::Down {
            return (EnumActionResult::Pass, None);
        }
        if world.getBlockState(pos.up(1)).getBlockId() != 0 {
            return (EnumActionResult::Pass, None);
        }
        let target = world.getBlockState(pos);
        let tillable = match target.getBlockId() {
            2 | 208 => true,
            3 => matches!(target.getMetadata() & 3, 0 | 1),
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

    fn stack() -> ItemStack {
        ItemStack {
            itemId: 290,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        }
    }

    #[test]
    fn golden_hoe_is_in_vanilla_hoe_registry_range() {
        let gold = ItemStack {
            itemId: 294,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        };
        assert!(ItemHoe::isItemHoe(&gold));
    }

    #[test]
    fn hoe_tills_grass_plain_and_coarse_dirt_with_air_above() {
        let mut world = WorldClient::new(0);
        let pos = BlockPos::new(0, 60, 0);
        world
            .invalidateRegionAndSetBlock(
                pos,
                crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(
                    2 << 4,
                ),
            )
            .unwrap();
        let (result, sound) = ItemHoe::predictOnItemUse(&world, pos, EnumFacing::Up, &stack());
        assert_eq!(result, EnumActionResult::Success);
        assert_eq!(sound, Some(HOE_TILL_SOUND));
        world
            .invalidateRegionAndSetBlock(
                pos,
                crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(
                    3 << 4,
                ),
            )
            .unwrap();
        assert_eq!(
            ItemHoe::predictOnItemUse(&world, pos, EnumFacing::Up, &stack()).0,
            EnumActionResult::Success
        );
        world
            .invalidateRegionAndSetBlock(
                pos,
                crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(
                    3 << 4 | 1,
                ),
            )
            .unwrap();
        assert_eq!(
            ItemHoe::predictOnItemUse(&world, pos, EnumFacing::Up, &stack()).0,
            EnumActionResult::Success
        );
        world
            .invalidateRegionAndSetBlock(
                pos,
                crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(
                    3 << 4 | 2,
                ),
            )
            .unwrap();
        assert_eq!(
            ItemHoe::predictOnItemUse(&world, pos, EnumFacing::Up, &stack()).0,
            EnumActionResult::Pass
        );
    }

    #[test]
    fn hoe_passes_covered_or_solid_targets() {
        let mut world = WorldClient::new(0);
        let pos = BlockPos::new(0, 60, 0);
        world
            .invalidateRegionAndSetBlock(
                pos,
                crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(
                    2 << 4,
                ),
            )
            .unwrap();
        world
            .invalidateRegionAndSetBlock(
                pos.up(1),
                crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(
                    1 << 4,
                ),
            )
            .unwrap();
        assert_eq!(
            ItemHoe::predictOnItemUse(&world, pos, EnumFacing::Up, &stack()).0,
            EnumActionResult::Pass
        );
        world
            .invalidateRegionAndSetBlock(
                pos.up(1),
                crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(0),
            )
            .unwrap();
        assert_eq!(
            ItemHoe::predictOnItemUse(&world, pos, EnumFacing::Down, &stack()).0,
            EnumActionResult::Pass
        );
        world
            .invalidateRegionAndSetBlock(
                pos,
                crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(
                    1 << 4,
                ),
            )
            .unwrap();
        assert_eq!(
            ItemHoe::predictOnItemUse(&world, pos, EnumFacing::Up, &stack()).0,
            EnumActionResult::Pass
        );
    }
}
