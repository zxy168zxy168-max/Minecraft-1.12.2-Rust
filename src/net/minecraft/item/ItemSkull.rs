use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::nbt::NBTBase::{TAG_COMPOUND, TAG_STRING};
use crate::net::minecraft::tileentity::TileEntitySkull::TileEntitySkull;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumActionResult::EnumActionResult;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

/// Client-side result branch of MCP 1.12.2 `ItemSkull#onItemUse`. The remote
/// world returns SUCCESS before creating the block/TileEntity, so this class
/// never fabricates skull NBT or consumes the stack locally.
pub struct ItemSkull;

impl ItemSkull {
    pub const ITEM_ID: i16 = 397;

    pub fn isItemSkull(stack: &ItemStack) -> bool {
        !stack.isEmpty() && stack.itemId == Self::ITEM_ID
    }

    /// `LayerCustomHead` profile extraction. Compound owners preserve UUID,
    /// name and signed texture properties. Legacy string owners deliberately
    /// remain name-only until the profile-completion service resolves them,
    /// matching `TileEntitySkull#updateGameprofile`.
    pub fn getPlayerProfile(stack: &ItemStack) -> Option<GameProfile> {
        if !Self::isItemSkull(stack) || stack.itemDamage != 3 {
            return None;
        }
        let tag = stack.tagCompound.as_ref()?;
        if tag.hasKeyWithType("SkullOwner", TAG_COMPOUND) {
            return TileEntitySkull::readGameProfileFromNBT(&tag.getCompoundTag("SkullOwner"));
        }
        if tag.hasKeyWithType("SkullOwner", TAG_STRING) {
            let name = tag.getString("SkullOwner");
            if !name.trim().is_empty() {
                return Some(GameProfile::new(None, name));
            }
        }
        None
    }

    pub fn predictOnItemUse(
        world: &WorldClient,
        player: &EntityPlayerSP,
        clickedPos: BlockPos,
        side: EnumFacing,
        stack: &ItemStack,
    ) -> EnumActionResult {
        if !Self::isItemSkull(stack) || side == EnumFacing::Down {
            return EnumActionResult::Fail;
        }
        let replaceable = world.isBlockReplaceable(clickedPos);
        let target = if replaceable {
            clickedPos
        } else {
            if !world.getBlockState(clickedPos).getBlock().materialIsSolid() {
                return EnumActionResult::Fail;
            }
            clickedPos.offset(side, 1)
        };
        if player.capabilities.allowEdit && world.isBlockReplaceable(target) {
            EnumActionResult::Success
        } else {
            EnumActionResult::Fail
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::block::state::IBlockState::IBlockState;

    fn skull() -> ItemStack {
        ItemStack {
            itemId: 397,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        }
    }

    #[test]
    fn down_face_fails_and_solid_side_succeeds() {
        let mut world = WorldClient::new(0);
        let pos = BlockPos::new(0, 64, 0);
        world
            .invalidateRegionAndSetBlock(pos, IBlockState::fromGlobalStateId(1 << 4))
            .unwrap();
        let player = EntityPlayerSP::new(1);
        assert_eq!(
            ItemSkull::predictOnItemUse(&world, &player, pos, EnumFacing::Down, &skull()),
            EnumActionResult::Fail
        );
        assert_eq!(
            ItemSkull::predictOnItemUse(&world, &player, pos, EnumFacing::North, &skull()),
            EnumActionResult::Success
        );
    }

    #[test]
    fn custom_head_reads_compound_and_legacy_owner_profiles() {
        use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;

        let mut owner = NBTTagCompound::new();
        owner.setString("Name", "Alex");
        owner.setString("Id", "ec561538-f3fd-461d-aff5-086b22154bce");
        let mut compound = NBTTagCompound::new();
        compound.setCompoundTag("SkullOwner", owner);
        let stack = ItemStack {
            itemId: 397,
            count: 1,
            itemDamage: 3,
            tagCompound: Some(compound),
        };
        let profile = ItemSkull::getPlayerProfile(&stack).expect("compound profile");
        assert_eq!(profile.getName(), "Alex");
        assert!(profile.getId().is_some());

        let mut legacy = NBTTagCompound::new();
        legacy.setString("SkullOwner", "Steve");
        let stack = ItemStack {
            itemId: 397,
            count: 1,
            itemDamage: 3,
            tagCompound: Some(legacy),
        };
        let profile = ItemSkull::getPlayerProfile(&stack).expect("legacy profile");
        assert_eq!(profile.getName(), "Steve");
        assert!(profile.getId().is_none());
    }
}
