use crate::net::minecraft::block::Block::Block;
use crate::net::minecraft::block::{BlockAnvil, BlockButton, BlockEndPortalFrame, BlockGlazedTerracotta, BlockLadder, BlockLever, BlockLog, BlockPumpkin, BlockQuartz, BlockRailBase, BlockRotatedPillar, BlockStairs, BlockTorch, BlockTrapDoor, BlockEndRod, BlockVine};
use crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::item::ItemRegistryData::definition;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::util::EnumActionResult::EnumActionResult;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

/// Source-owned Minecraft 1.12.2 `ItemBlock` client interaction path. The
/// preview is intentionally limited to block families whose placement state is
/// confirmed from MCP; the caller applies that state immediately and later
/// server block packets remain authoritative corrections.
pub struct ItemBlock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemBlockPlacement {
    pub pos: BlockPos,
    pub state: IBlockState,
    /// Snapshot of the source stack used to guard the write-locked prediction
    /// against a concurrent held-item update between ray tracing and mutation.
    pub sourceItemId: i16,
    pub sourceItemDamage: i16,
}

impl ItemBlock {
    /// In protocol 340, `registerItemBlock` registers the item under the
    /// block's registry ID. Comparing both exact registry names avoids treating
    /// gaps such as water, fire or piston extension as placeable ItemBlocks.
    pub fn isItemBlock(stack: &ItemStack) -> bool {
        if stack.isEmpty() || !(1..=255).contains(&(stack.itemId as i32)) {
            return false;
        }
        let block = Block::getBlockById(stack.itemId as i32);
        if block.isAir() { return false; }
        let itemName = definition(stack.itemId).registryName;
        itemName.strip_prefix("minecraft:") == Some(block.getRegistryPath())
    }

    pub fn getBlock(stack: &ItemStack) -> Option<Block> {
        Self::isItemBlock(stack).then(|| Block::getBlockById(stack.itemId as i32))
    }

    /// Port of `ItemBlock#canPlaceBlockOnSide` through the currently available
    /// WorldClient block/entity state.
    pub fn canPlaceBlockOnSide(
        world: &WorldClient,
        pos: BlockPos,
        mut side: EnumFacing,
        player: &EntityPlayerSP,
        stack: &ItemStack,
    ) -> bool {
        let Some(blockToPlace) = Self::getBlock(stack) else { return false; };
        let clicked = world.getBlockState(pos);
        let target = if clicked.getBlockId() == 78 {
            // `Blocks.SNOW_LAYER` forces UP before the replaceable test.
            side = EnumFacing::Up;
            pos
        } else if world.isBlockReplaceable(pos) {
            pos
        } else {
            pos.offset(side, 1)
        };

        let candidate = IBlockState::fromGlobalStateId(Block::getIdFromBlock(blockToPlace) << 4);
        let sourceAllowsPlacement = if BlockTorch::isBlockTorch(candidate) {
            BlockTorch::canPlaceBlockAt(world, target)
        } else if BlockLadder::isBlockLadder(candidate) {
            BlockLadder::canPlaceBlockOnSide(world, target, side)
        } else if BlockRailBase::isRailBlock(candidate) {
            BlockRailBase::canPlaceBlockAt(world, target)
        } else if BlockTrapDoor::isBlockTrapDoor(candidate) {
            BlockTrapDoor::canPlaceBlockOnSide()
        } else {
            true
        };

        sourceAllowsPlacement && world.mayPlace(blockToPlace, target, false, side, Some(player))
    }

    /// Source-backed `Block#onBlockPlaced` preview for the concrete stateful
    /// families already ported. This does not mutate WorldClient; it carries
    /// the exact state into the controller result so Minecraft can apply the
    /// local prediction without re-deriving metadata.
    pub fn placementPreview(
        world: &WorldClient,
        _player: &EntityPlayerSP,
        pos: BlockPos,
        mut side: EnumFacing,
        _hitY: f32,
        stack: &ItemStack,
    ) -> Option<ItemBlockPlacement> {
        let block = Self::getBlock(stack)?;
        let clicked = world.getBlockState(pos);
        let target = if clicked.getBlockId() == 78 {
            side = EnumFacing::Up;
            pos
        } else if world.isBlockReplaceable(pos) {
            pos
        } else {
            pos.offset(side, 1)
        };
        let blockId = Block::getIdFromBlock(block);
        let base = IBlockState::fromGlobalStateId(blockId << 4);
        let state = if BlockTorch::isBlockTorch(base) {
            BlockTorch::onBlockPlacedState(blockId, world, target, side)
        } else if BlockLadder::isBlockLadder(base) {
            BlockLadder::onBlockPlacedState(world, target, side)
        } else if BlockRailBase::isRailBlock(base) {
            BlockRailBase::onBlockPlacedState(blockId, stack.itemDamage as i32)
        } else if BlockEndPortalFrame::BlockEndPortalFrame::isBlockEndPortalFrame(base) {
            BlockEndPortalFrame::BlockEndPortalFrame::onBlockPlacedState(_player)
        } else if BlockTrapDoor::isBlockTrapDoor(base) {
            // `BlockTrapDoor#onBlockPlaced` also queries World#isBlockPowered.
            // Until the concrete redstone power graph is ported, carrying a
            // hard-coded false state would be less faithful than waiting for
            // the server's block change. The exact constructor remains in
            // BlockTrapDoor and requires an explicit powered value.
            return None;
        } else if usesDefaultPlacementState(blockId) {
            // MCP `Block#onBlockPlaced` delegates to getStateFromMeta(meta).
            // These IDs are the conservative source-audited subset whose
            // concrete block class does not replace onBlockPlaced or add a
            // multi-block onBlockPlacedBy sequence.
            IBlockState::fromGlobalStateId(
                (blockId << 4) | itemBlockMetadata(stack, blockId),
            )
        } else {
            return None;
        };
        Some(ItemBlockPlacement {
            pos: target,
            state,
            sourceItemId: stack.itemId,
            sourceItemDamage: stack.itemDamage,
        })
    }


    /// Server-authoritative counterpart of the source-backed placement preview.
    /// This uses only placement rules already ported from MCP; unsupported
    /// concrete onBlockPlaced/onBlockPlacedBy families return None rather than
    /// fabricating a state.
    pub fn serverPlacementState<A: IBlockAccess>(
        world: &A,
        pos: BlockPos,
        mut side: EnumFacing,
        hitY: f32,
        stack: &ItemStack,
        placerYaw: f32,
        poweredAtTarget: Option<bool>,
    ) -> Option<ItemBlockPlacement> {
        let block = Self::getBlock(stack)?;
        let clicked = world.getBlockState(pos);
        let target = if clicked.getBlockId() == 78 {
            side = EnumFacing::Up;
            pos
        } else if isReplaceableState(clicked) {
            pos
        } else {
            pos.offset(side, 1)
        };
        let blockId = Block::getIdFromBlock(block);
        let base = IBlockState::fromGlobalStateId(blockId << 4);
        let state = if BlockTorch::isBlockTorch(base) {
            if !BlockTorch::canPlaceBlockAt(world, target) { return None; }
            BlockTorch::onBlockPlacedState(blockId, world, target, side)
        } else if BlockLadder::isBlockLadder(base) {
            if !BlockLadder::canPlaceBlockOnSide(world, target, side) { return None; }
            BlockLadder::onBlockPlacedState(world, target, side)
        } else if BlockRailBase::isRailBlock(base) {
            if !BlockRailBase::canPlaceBlockAt(world, target) { return None; }
            BlockRailBase::onBlockPlacedState(blockId, stack.itemDamage as i32)
        } else if BlockStairs::isBlockStairs(base) {
            BlockStairs::onBlockPlacedState(blockId, side, hitY, placerYaw)
        } else if BlockLog::isBlockLog(base) {
            BlockLog::onBlockPlacedState(blockId, itemBlockMetadata(stack, blockId), side)
        } else if BlockEndRod::isBlockEndRod(base) {
            BlockEndRod::onBlockPlacedState(world, target, side)
        } else if BlockPumpkin::isBlockPumpkin(base) {
            BlockPumpkin::onBlockPlacedState(blockId, placerYaw)
        } else if BlockGlazedTerracotta::isBlockGlazedTerracotta(base) {
            BlockGlazedTerracotta::onBlockPlacedState(blockId, placerYaw)
        } else if BlockRotatedPillar::isSimpleRotatedPillar(base) {
            BlockRotatedPillar::onBlockPlacedState(blockId, side)
        } else if BlockQuartz::isBlockQuartz(base) {
            BlockQuartz::onBlockPlacedState(side, itemBlockMetadata(stack, blockId))
        } else if BlockAnvil::isBlockAnvil(base) {
            BlockAnvil::onBlockPlacedState(placerYaw, stack.itemDamage)
        } else if BlockButton::isBlockButton(base) {
            if !BlockButton::canPlaceBlock(world,target,side) { return None; }
            BlockButton::onBlockPlacedState(blockId,world,target,side)
        } else if BlockLever::isBlockLever(base) {
            if !EnumFacing::VALUES.into_iter().any(|f|BlockButton::canPlaceBlock(world,target,f)){return None;}
            BlockLever::onBlockPlacedState(world,target,side,placerYaw)
        } else if BlockVine::isBlockVine(base) {
            if !BlockVine::canPlaceBlockOnSide(world,target,side){return None;}
            BlockVine::onBlockPlacedState(side)
        } else if BlockEndPortalFrame::BlockEndPortalFrame::isBlockEndPortalFrame(base) {
            let facing = EnumFacing::fromAngle(placerYaw as f64).opposite();
            let meta = facing.horizontalIndex().unwrap_or(2) as i32;
            IBlockState::fromGlobalStateId((blockId << 4) | meta)
        } else if BlockTrapDoor::isBlockTrapDoor(base) {
            let powered = poweredAtTarget?;
            let horizontal = EnumFacing::fromAngle(placerYaw as f64);
            BlockTrapDoor::onBlockPlacedState(blockId, side, hitY, horizontal, powered)
        } else if usesDefaultPlacementState(blockId) {
            IBlockState::fromGlobalStateId((blockId << 4) | itemBlockMetadata(stack, blockId))
        } else {
            return None;
        };
        Some(ItemBlockPlacement { pos: target, state, sourceItemId: stack.itemId, sourceItemDamage: stack.itemDamage })
    }

    /// `ItemBlock#onItemUse` returns SUCCESS after all edit/may-place checks,
    /// even if the subsequent client-side setBlockState does not change the
    /// server-authoritative world. This function deliberately predicts only
    /// the result and never fabricates a local block.
    pub fn predictOnItemUse(
        world: &WorldClient,
        player: &EntityPlayerSP,
        pos: BlockPos,
        side: EnumFacing,
        stack: &ItemStack,
    ) -> EnumActionResult {
        if stack.isEmpty() || !player.capabilities.allowEdit {
            return EnumActionResult::Fail;
        }
        if Self::canPlaceBlockOnSide(world, pos, side, player, stack) {
            EnumActionResult::Success
        } else {
            EnumActionResult::Fail
        }
    }
}


/// MCP `Block#isReplaceable` for the source-confirmed vanilla overrides used
/// by ItemBlock placement. Shared by client prediction and server authority.
pub const fn isReplaceableState(state: IBlockState) -> bool {
    match state.getBlockId() {
        0 | 31 | 32 | 106 => true,
        78 => state.getMetadata() & 7 == 0,
        _ => false,
    }
}

/// Exact `Item#getMetadata` behavior needed by the default-placement subset.
/// ItemLeaves is the one included subclass that adds a persistent bit; ordinary
/// subtype ItemBlocks pass item damage through and base ItemBlock returns zero.
pub(crate) fn itemBlockMetadata(stack: &ItemStack, blockId: i32) -> i32 {
    let damage = stack.itemDamage.max(0) as i32;
    match blockId {
        18 | 161 => (damage | 4) & 15, // ItemLeaves#getMetadata
        _ if definition(stack.itemId).hasSubtypes => damage & 15,
        _ => 0,
    }
}

/// Conservative table audited against MCP 1.12.2 `Block.registerBlocks`, each
/// concrete class's `onBlockPlaced`/`onBlockPlacedBy`, and Item.java's
/// registerItemBlock calls. Directional, powered, layered, slab and multi-block
/// families remain in their concrete ports rather than receiving guessed meta.
pub(crate) const fn usesDefaultPlacementState(blockId: i32) -> bool {
    matches!(
        blockId,
        1..=5
            | 7
            | 12..=16
            | 18..=22
            | 24
            | 30
            | 35
            | 41..=42
            | 45
            | 47..=49
            | 56..=58
            | 73
            | 79..=80
            | 82
            | 84..=85
            | 87..=89
            | 95
            | 97..=98
            | 101..=103
            | 110
            | 112..=113
            | 121
            | 123
            | 129
            | 133
            | 139
            | 152..=153
            | 159..=161
            | 165..=166
            | 168..=169
            | 172..=174
            | 179
            | 188..=192
            | 201
            | 206
            | 208
            | 213..=215
            | 217
            | 251
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(id: i16) -> ItemStack {
        ItemStack { itemId: id, count: 1, itemDamage: 0, tagCompound: None }
    }

    #[test]
    fn torch_preview_uses_clicked_face_and_resolved_target() {
        let mut world = WorldClient::new(0);
        let clicked = BlockPos::new(0, 64, 0);
        world.invalidateRegionAndSetBlock(clicked, IBlockState::fromGlobalStateId(1 << 4)).unwrap();
        let preview = ItemBlock::placementPreview(
            &world,
            &EntityPlayerSP::new(1),
            clicked,
            EnumFacing::Up,
            1.0,
            &stack(50),
        ).unwrap();
        assert_eq!(preview.pos, clicked.up(1));
        assert_eq!(preview.state.getMetadata(), 5);
    }

    #[test]
    fn registry_identity_distinguishes_item_blocks_from_registry_gaps() {
        assert!(ItemBlock::isItemBlock(&stack(1)));
        assert!(ItemBlock::isItemBlock(&stack(54)));
        assert!(ItemBlock::isItemBlock(&stack(219)));
        assert!(!ItemBlock::isItemBlock(&stack(8)));
        assert!(!ItemBlock::isItemBlock(&stack(259)));
        assert!(!ItemBlock::isItemBlock(&ItemStack::EMPTY));
    }

    #[test]
    fn default_block_preview_uses_item_metadata_only_for_subtype_items() {
        let world = WorldClient::new(0);
        let player = EntityPlayerSP::new(1);
        let target = BlockPos::new(0, 64, 0);
        let stone = ItemStack { itemId: 1, count: 1, itemDamage: 3, tagCompound: None };
        let cobblestone = ItemStack { itemId: 4, count: 1, itemDamage: 9, tagCompound: None };

        let stonePreview = ItemBlock::placementPreview(
            &world, &player, target, EnumFacing::Up, 0.5, &stone,
        ).unwrap();
        let cobblePreview = ItemBlock::placementPreview(
            &world, &player, target, EnumFacing::Up, 0.5, &cobblestone,
        ).unwrap();
        assert_eq!(stonePreview.state.getMetadata(), 3);
        assert_eq!(cobblePreview.state.getMetadata(), 0);
    }

    #[test]
    fn leaves_preview_preserves_itemleaves_decay_check_bit() {
        let world = WorldClient::new(0);
        let player = EntityPlayerSP::new(1);
        let preview = ItemBlock::placementPreview(
            &world,
            &player,
            BlockPos::new(0, 64, 0),
            EnumFacing::Up,
            0.5,
            &ItemStack { itemId: 18, count: 1, itemDamage: 2, tagCompound: None },
        ).unwrap();
        assert_eq!(preview.state.getMetadata(), 6);
    }

    #[test]
    fn directional_and_side_effect_blocks_are_not_given_generic_prediction() {
        let world = WorldClient::new(0);
        let player = EntityPlayerSP::new(1);
        let target = BlockPos::new(0, 64, 0);
        for itemId in [23_i16, 46_i16] {
            assert!(ItemBlock::placementPreview(
                &world,
                &player,
                target,
                EnumFacing::Up,
                0.5,
                &stack(itemId),
            )
            .is_none());
        }
    }
}
