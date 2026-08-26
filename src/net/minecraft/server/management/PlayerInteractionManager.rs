use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::entity::player::EntityPlayerMP::EntityPlayerMP;
use crate::net::minecraft::item::ItemBlock::ItemBlock;
use crate::net::minecraft::network::play::server::SPacketBlockChange::SPacketBlockChange;
use crate::net::minecraft::network::NetworkManager::NetworkManager;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumActionResult::EnumActionResult;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::EnumHand::EnumHand;
use crate::net::minecraft::world::GameType::GameType;
use crate::net::minecraft::world::WorldServer::WorldServer;

/// MCP 1.12.2 `PlayerInteractionManager` at the first server-authoritative
/// block-edit boundary. The class owns game-mode placement/harvest decisions;
/// unsupported block-specific activation/multi-block callbacks remain pending
/// rather than being collapsed into generic setBlockState calls.
#[derive(Debug, Clone)]
pub struct PlayerInteractionManager {
    gameType: GameType,
}

impl PlayerInteractionManager {
    pub const fn new(gameType: GameType) -> Self {
        Self { gameType }
    }
    pub const fn getGameType(&self) -> GameType {
        self.gameType
    }
    pub const fn isCreative(&self) -> bool {
        self.gameType.isCreative()
    }
    pub fn setGameType(&mut self, gameType: GameType) {
        self.gameType = gameType;
    }

    /// Source creative branch of `onBlockClicked`: creative players remove the
    /// target immediately. Survival/adventure progressive hardness requires the
    /// remaining common World/player-effect runtime and is not fabricated.
    pub fn onBlockClicked(
        &mut self,
        network: &mut NetworkManager,
        world: &mut WorldServer,
        player: &mut EntityPlayerMP,
        pos: BlockPos,
        _side: EnumFacing,
    ) -> Result<bool, String> {
        if !self.isCreative() || !player.capabilities.allowEdit {
            return Ok(false);
        }
        if !(0..256).contains(&pos.y) {
            return Ok(false);
        }
        let current = world.getBlockStateAt(pos)?;
        if current.isAir() || current.getBlock().getBlockHardness() < 0.0 {
            return Ok(false);
        }
        world.setBlockStateAt(pos, IBlockState::default())?;
        Self::sendBlockChange(network, world, pos)?;
        Ok(true)
    }

    /// Source-shaped `processRightClickBlock` ItemBlock branch. It preserves
    /// the ItemBlock target resolution/state algorithms already ported from
    /// MCP and lets the server Chunk/Anvil layer become authoritative.
    pub fn processRightClickBlock(
        &mut self,
        network: &mut NetworkManager,
        world: &mut WorldServer,
        player: &mut EntityPlayerMP,
        hand: EnumHand,
        pos: BlockPos,
        side: EnumFacing,
        hit_x: f32,
        hit_y: f32,
        hit_z: f32,
    ) -> Result<EnumActionResult, String> {
        let _ = (hit_x, hit_z);
        if !(0..256).contains(&pos.y) || !player.capabilities.allowEdit {
            return Ok(EnumActionResult::Fail);
        }
        // Vanilla first gives the clicked block its activation opportunity. At
        // this boundary the source-confirmed unconditional activators are kept
        // authoritative by refusing to reinterpret their click as placement;
        // their GUI/state callbacks are separate concrete block tranches.
        let clicked = world.getBlockStateAt(pos)?;
        let main_empty = player.getHeldItem(EnumHand::MainHand).isEmpty();
        let off_empty = player.getHeldItem(EnumHand::OffHand).isEmpty();
        // MCP PlayerInteractionManager#processRightClickBlock gives the block
        // activation priority unless the player is sneaking with a held item.
        // Concrete activation callbacks are still separate ports; this branch
        // deliberately prevents an activation click from being reinterpreted
        // as placement.
        let allow_block_activation = !player.entity.sneaking || (main_empty && off_empty);
        if allow_block_activation && clicked.getBlock().predictsActivationSuccess() {
            Self::sendBlockChange(network, world, pos)?;
            Self::sendBlockChange(network, world, pos.offset(side, 1))?;
            return Ok(EnumActionResult::Success);
        }

        let held = player.getHeldItem(hand).clone();
        if held.isEmpty() || !ItemBlock::isItemBlock(&held) {
            return Ok(EnumActionResult::Pass);
        }
        // `World#mayPlace` resolves the actual target through World, which in
        // turn loads/provides the target Chunk. Do that before passing the
        // read-only IBlockAccess view to the source placement-state helpers so
        // a cross-chunk placement can never treat an unloaded target as air.
        let target = if clicked.getBlockId() == 78
            || crate::net::minecraft::item::ItemBlock::isReplaceableState(clicked)
        {
            pos
        } else {
            pos.offset(side, 1)
        };
        if (0..256).contains(&target.y) {
            let _ = world.getBlockStateAt(target)?;
        }
        let Some(preview) = ItemBlock::serverPlacementState(
            world,
            pos,
            side,
            hit_y,
            &held,
            player.entity.rotationYaw,
            None,
        ) else {
            Self::sendBlockChange(network, world, pos)?;
            Self::sendBlockChange(network, world, pos.offset(side, 1))?;
            return Ok(EnumActionResult::Fail);
        };
        if !world.mayPlaceStateForPlayer(preview.state, preview.pos, player.entity.boundingBox) {
            Self::sendBlockChange(network, world, pos)?;
            Self::sendBlockChange(network, world, preview.pos)?;
            return Ok(EnumActionResult::Fail);
        }
        let old = world.setBlockStateAt(preview.pos, preview.state)?;
        if old != preview.state && !self.isCreative() {
            player.getHeldItemMut(hand).shrink(1);
        }
        Self::sendBlockChange(network, world, pos)?;
        Self::sendBlockChange(network, world, preview.pos)?;
        Ok(EnumActionResult::Success)
    }

    pub fn sendBlockChange(
        network: &mut NetworkManager,
        world: &mut WorldServer,
        pos: BlockPos,
    ) -> Result<(), String> {
        let state = world.getBlockStateAt(pos)?;
        network
            .sendPacket(&SPacketBlockChange::new(pos, state).writePacketData())
            .map_err(|e| e.to_string())
    }
}
