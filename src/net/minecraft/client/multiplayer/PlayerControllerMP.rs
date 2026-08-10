use crate::net::minecraft::client::audio::LocalSoundEvent::LocalSoundEvent;
use crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::client::renderer::DestroyBlockProgress::DestroyBlockProgress;
use crate::net::minecraft::block::BlockButton;
use crate::net::minecraft::block::BlockDoor;
use crate::net::minecraft::block::BlockFenceGate;
use crate::net::minecraft::block::BlockJukebox;
use crate::net::minecraft::block::BlockRedstoneComparator;
use crate::net::minecraft::block::BlockRedstoneRepeater;
use crate::net::minecraft::block::BlockTrapDoor;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::SoundType::SoundType;
use crate::net::minecraft::item::Item::Item;
use crate::net::minecraft::item::ItemBlock::{ItemBlock, ItemBlockPlacement};
use crate::net::minecraft::item::ItemBucket::ItemBucket;
use crate::net::minecraft::item::ItemDoor::ItemDoor;
use crate::net::minecraft::item::ItemHoe::ItemHoe;
use crate::net::minecraft::item::ItemSign::ItemSign;
use crate::net::minecraft::item::ItemSkull::ItemSkull;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::play::client::CPacketEnchantItem::CPacketEnchantItem;
use crate::net::minecraft::network::play::client::CPacketHeldItemChange::CPacketHeldItemChange;
use crate::net::minecraft::network::play::client::CPacketPlayerDigging::{
    Action as DiggingAction, CPacketPlayerDigging,
};
use crate::net::minecraft::network::play::client::CPacketPlayerTryUseItem::CPacketPlayerTryUseItem;
use crate::net::minecraft::network::play::client::CPacketPlayerTryUseItemOnBlock::CPacketPlayerTryUseItemOnBlock;
use crate::net::minecraft::util::EnumActionResult::EnumActionResult;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::EnumHand::EnumHand;
use crate::net::minecraft::util::SoundCategory::SoundCategory;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::math::RayTraceResult::RayTraceResult;
use crate::net::minecraft::util::math::Vec3d::Vec3d;
use crate::net::minecraft::world::GameType::GameType;

/// Multiplayer interaction controller following MCP 1.12.2
/// `PlayerControllerMP` packet ordering and block-hit state.
///
/// Block hardness, held-item speed and harvestability are sourced from the
/// compiled MCP 1.12.2 registries. Completed block destruction is also applied
/// to WorldClient immediately, matching `onPlayerDestroyBlock`; later server
/// block packets remain authoritative corrections.
#[derive(Debug, Clone)]
pub struct BlockRightClickResult {
    pub packet: Option<RawPacket>,
    pub result: EnumActionResult,
    /// True when SUCCESS came from the held item's `onItemUse` branch rather
    /// than the clicked block's `onBlockActivated` branch. The historical field
    /// name is retained to avoid changing the established caller contract.
    pub usedItemBlock: bool,
    /// Exact local `Block#onBlockPlaced` result for source-backed ItemBlocks.
    /// The caller may apply this prediction; later server block packets remain
    /// authoritative and overwrite it.
    pub predictedPlacement: Option<ItemBlockPlacement>,
    /// Exact remote-world mutation performed by source `Block#onBlockActivated`
    /// implementations that do not guard themselves with `world.isRemote`.
    /// The expected-state snapshot prevents a late local prediction from
    /// overwriting a newer authoritative server block packet.
    pub predictedBlockState: Option<BlockStatePrediction>,
    /// Local sound emitted by an item `onItemUse` prediction, such as
    /// `ItemHoe#onItemUse`. The caller plays it after the source-equivalent
    /// result is known.
    pub sound: Option<(&'static str, f32, f32)>,
}

impl BlockRightClickResult {
    fn new(packet: Option<RawPacket>, result: EnumActionResult, usedItemBlock: bool) -> Self {
        Self {
            packet,
            result,
            usedItemBlock,
            predictedPlacement: None,
            predictedBlockState: None,
            sound: None,
        }
    }


    fn withSound(mut self, sound: Option<(&'static str, f32, f32)>) -> Self {
        self.sound = sound;
        self
    }

    fn withPredictedPlacement(mut self, placement: Option<ItemBlockPlacement>) -> Self {
        self.predictedPlacement = placement;
        self
    }

    fn withPredictedBlockState(mut self, prediction: Option<BlockStatePrediction>) -> Self {
        self.predictedBlockState = prediction;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockStatePrediction {
    pub pos: BlockPos,
    pub expectedState: IBlockState,
    pub state: IBlockState,
}

/// Source-equivalent result of `PlayerControllerMP#processRightClick` after
/// the held item's `onItemRightClick` client-side branch has been evaluated.
#[derive(Debug, Clone, PartialEq)]
pub struct AirRightClickResult {
    /// MCP spectator branch returns before sync/send; normal use carries the
    /// CPacketPlayerTryUseItem packet here.
    pub packet: Option<RawPacket>,
    pub result: EnumActionResult,
    pub fillBucket: Option<crate::net::minecraft::item::ItemBucket::BucketFill>,
    pub emptyBucket: Option<crate::net::minecraft::item::ItemBucket::BucketEmpty>,
    pub thrown: Option<Thrown>,
}

impl AirRightClickResult {
    pub fn new(packet: Option<RawPacket>, result: EnumActionResult) -> Self {
        Self { packet, result, fillBucket: None, emptyBucket: None, thrown: None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thrown {
    pub sound: &'static str,
    pub category: SoundCategory,
    pub pitch: f32,
}

#[derive(Debug, Clone)]
pub struct PlayerControllerMP {
    currentBlock: BlockPos,
    curBlockDamageMP: f32,
    stepSoundTickCounter: f32,
    blockHitDelay: i32,
    isHittingBlock: bool,
    currentGameType: GameType,
    currentPlayerItem: i32,
    currentItemHittingBlock: ItemStack,
    pendingDestroyEffect: Option<(BlockPos, IBlockState)>,
    pendingHitSound: Option<LocalSoundEvent>,
}

impl Default for PlayerControllerMP {
    fn default() -> Self {
        Self {
            currentBlock: BlockPos::new(-1, -1, -1),
            curBlockDamageMP: 0.0,
            stepSoundTickCounter: 0.0,
            blockHitDelay: 0,
            isHittingBlock: false,
            currentGameType: GameType::Survival,
            currentPlayerItem: -1,
            currentItemHittingBlock: ItemStack::EMPTY,
            pendingDestroyEffect: None,
            pendingHitSound: None,
        }
    }
}

impl PlayerControllerMP {
    pub fn new() -> Self { Self::default() }

    pub fn setGameType(&mut self, typeIn: GameType) {
        self.currentGameType = typeIn;
    }

    pub const fn getCurrentGameType(&self) -> GameType { self.currentGameType }

    pub const fn getBlockReachDistance(&self) -> f64 {
        if self.currentGameType.isCreative() { 5.0 } else { 4.5 }
    }

    pub const fn extendedReach(&self) -> bool { self.currentGameType.isCreative() }

    /// MCP `PlayerControllerMP#sendEnchantPacket`. Network ownership remains
    /// with the caller in this Rust port, so the controller returns the exact
    /// protocol packet for the active connection to send.
    pub fn sendEnchantPacket(&self, windowId: i32, button: i32) -> RawPacket {
        CPacketEnchantItem::new(windowId, button).writePacketData()
    }

    /// Direct port of `PlayerControllerMP.syncCurrentPlayItem`. The caller
    /// sends the returned packet before an interaction or during the controller
    /// update phase.
    pub fn syncCurrentPlayItem(&mut self, currentItem: i32) -> Option<RawPacket> {
        if currentItem == self.currentPlayerItem {
            return None;
        }
        self.currentPlayerItem = currentItem;
        Some(CPacketHeldItemChange::new(currentItem).writePacketData())
    }

    pub fn clickBlock(
        &mut self,
        world: &WorldClient,
        player: &EntityPlayerSP,
        loc: BlockPos,
        face: EnumFacing,
    ) -> Vec<RawPacket> {
        // Adventure CanDestroy and world-border checks require the corresponding
        // capability/border ports. Until then, do not pretend adventure editing
        // is valid. Survival and creative follow the original packet state.
        if self.currentGameType.isAdventure() || world.getBlockState(loc).getBlock().isAir() {
            return Vec::new();
        }

        let mut packets = Vec::with_capacity(2);
        if self.currentGameType.isCreative() {
            packets.push(CPacketPlayerDigging::new(
                DiggingAction::StartDestroyBlock,
                loc,
                face,
            ).writePacketData());
            let held = player.inventory.getCurrentItem();
            if !is_creative_sword(held) {
                self.pendingDestroyEffect = Some((loc, world.getBlockState(loc)));
            }
            self.blockHitDelay = 5;
            return packets;
        }

        let held = player.inventory.getCurrentItem();
        if !self.isHittingBlock || !self.isHittingPosition(loc, held) {
            if self.isHittingBlock {
                packets.push(CPacketPlayerDigging::new(
                    DiggingAction::AbortDestroyBlock,
                    self.currentBlock,
                    face,
                ).writePacketData());
            }
            let state = world.getBlockState(loc);
            packets.push(CPacketPlayerDigging::new(
                DiggingAction::StartDestroyBlock,
                loc,
                face,
            ).writePacketData());

            // Vanilla destroys hardness>=1 blocks locally after START and does
            // not emit a synthetic STOP packet. The caller consumes this exact
            // state through `takeBlockDestroyEffect` and applies AIR locally.
            if state.getPlayerRelativeBlockHardness(world, player) >= 1.0 {
                self.pendingDestroyEffect = Some((loc, state));
                self.isHittingBlock = false;
                self.currentBlock = BlockPos::new(self.currentBlock.x, -1, self.currentBlock.z);
                self.curBlockDamageMP = 0.0;
            } else {
                self.isHittingBlock = true;
                self.currentBlock = loc;
                self.currentItemHittingBlock = held.clone();
                self.curBlockDamageMP = 0.0;
                self.stepSoundTickCounter = 0.0;
            }
        }
        packets
    }

    /// MCP `resetBlockRemoving` network branch.
    pub fn resetBlockRemoving(&mut self) -> Vec<RawPacket> {
        if !self.isHittingBlock {
            return Vec::new();
        }
        let packet = CPacketPlayerDigging::new(
            DiggingAction::AbortDestroyBlock,
            self.currentBlock,
            EnumFacing::Down,
        ).writePacketData();
        self.isHittingBlock = false;
        self.curBlockDamageMP = 0.0;
        self.currentItemHittingBlock = ItemStack::EMPTY;
        vec![packet]
    }

    /// Direct port of the packet/progress state in
    /// `PlayerControllerMP.onPlayerDamageBlock`. The boolean reports whether
    /// Minecraft should swing the main hand for this client tick.
    pub fn onPlayerDamageBlock(
        &mut self,
        world: &WorldClient,
        player: &EntityPlayerSP,
        posBlock: BlockPos,
        directionFacing: EnumFacing,
    ) -> (Vec<RawPacket>, bool) {
        if self.blockHitDelay > 0 {
            self.blockHitDelay -= 1;
            return (Vec::new(), true);
        }

        if self.currentGameType.isCreative() {
            let state = world.getBlockState(posBlock);
            if state.isAir() { return (Vec::new(), false); }
            let held = player.inventory.getCurrentItem();
            if !is_creative_sword(held) {
                self.pendingDestroyEffect = Some((posBlock, state));
            }
            self.blockHitDelay = 5;
            return (vec![CPacketPlayerDigging::new(
                DiggingAction::StartDestroyBlock,
                posBlock,
                directionFacing,
            ).writePacketData()], true);
        }

        let held = player.inventory.getCurrentItem();
        if self.isHittingPosition(posBlock, held) {
            let state = world.getBlockState(posBlock);
            if state.isAir() {
                self.isHittingBlock = false;
                self.currentItemHittingBlock = ItemStack::EMPTY;
                return (Vec::new(), false);
            }

            self.curBlockDamageMP += state.getPlayerRelativeBlockHardness(world, player);
            if self.stepSoundTickCounter % 4.0 == 0.0 {
                let soundType = SoundType::forBlockId(state.getBlockId());
                self.pendingHitSound = Some(LocalSoundEvent::positioned(
                    soundType.getHitSound().to_string(),
                    SoundCategory::Neutral,
                    [
                        posBlock.x as f32 + 0.5,
                        posBlock.y as f32 + 0.5,
                        posBlock.z as f32 + 0.5,
                    ],
                    (soundType.getVolume() + 1.0) / 8.0,
                    soundType.getPitch() * 0.5,
                ));
            }
            self.stepSoundTickCounter += 1.0;
            if self.curBlockDamageMP >= 1.0 {
                self.pendingDestroyEffect = Some((posBlock, state));
                self.isHittingBlock = false;
                self.currentBlock = BlockPos::new(self.currentBlock.x, -1, self.currentBlock.z);
                self.curBlockDamageMP = 0.0;
                self.stepSoundTickCounter = 0.0;
                self.blockHitDelay = 5;
                self.currentItemHittingBlock = ItemStack::EMPTY;
                return (vec![CPacketPlayerDigging::new(
                    DiggingAction::StopDestroyBlock,
                    posBlock,
                    directionFacing,
                ).writePacketData()], true);
            }
            (Vec::new(), true)
        } else {
            let packets = self.clickBlock(world, player, posBlock, directionFacing);
            let swing = !packets.is_empty();
            (packets, swing)
        }
    }

    fn isHittingPosition(&self, pos: BlockPos, held: &ItemStack) -> bool {
        let both_empty = self.currentItemHittingBlock.isEmpty() && held.isEmpty();
        let same_item = !self.currentItemHittingBlock.isEmpty()
            && !held.isEmpty()
            && held.itemId == self.currentItemHittingBlock.itemId
            && ItemStack::areItemStackTagsEqual(held, &self.currentItemHittingBlock)
            && (held.isItemStackDamageable()
                || held.itemDamage == self.currentItemHittingBlock.itemDamage);
        pos == self.currentBlock && (both_empty || same_item)
    }

    pub const fn getCurBlockDamageMP(&self) -> f32 { self.curBlockDamageMP }

    /// Source-backed client result path for MCP
    /// `PlayerControllerMP#processRightClickBlock`. The serverbound packet is
    /// retained separately from SUCCESS/PASS/FAIL so Minecraft can preserve
    /// the original two-hand loop and swing only on SUCCESS.
    pub fn processRightClickBlock(
        &self,
        world: &WorldClient,
        player: &EntityPlayerSP,
        hit: RayTraceResult,
        hand: EnumHand,
    ) -> BlockRightClickResult {
        let pos = hit.getBlockPos();
        let stack = player.getHeldItem(hand);
        let state = world.getBlockState(pos);
        let bothHandsEmpty = player.getHeldItem(EnumHand::MainHand).isEmpty()
            && player.getHeldItem(EnumHand::OffHand).isEmpty();
        let sourceActivationSuccess = match state.getBlockId() {
            // Comparator/repeater return false when the player cannot edit.
            93 | 94 | 149 | 150 => player.capabilities.allowEdit,
            // Jukebox consumes the click only when HAS_RECORD is true.
            84 => BlockJukebox::hasRecord(state),
            _ => state.getBlock().predictsActivationSuccess(),
        };
        let activated = self.currentGameType != GameType::Spectator
            && (!player.entity.sneaking || bothHandsEmpty)
            && sourceActivationSuccess;
        let activationPrediction = if activated {
            predictedActivationState(world, player, pos, state)
        } else {
            None
        };

        if !activated
            && ItemBlock::isItemBlock(stack)
            && !ItemBlock::canPlaceBlockOnSide(world, pos, hit.sideHit, player, stack)
        {
            // Vanilla returns FAIL before constructing/sending the packet when
            // ItemBlock#canPlaceBlockOnSide rejects the target.
            return BlockRightClickResult::new(None, EnumActionResult::Fail, false);
        }

        let packet = CPacketPlayerTryUseItemOnBlock::new(
            pos,
            hit.sideHit,
            hand,
            (hit.hitVec.x - pos.x as f64) as f32,
            (hit.hitVec.y - pos.y as f64) as f32,
            (hit.hitVec.z - pos.z as f64) as f32,
        ).writePacketData();

        if activated || self.currentGameType == GameType::Spectator {
            return BlockRightClickResult::new(Some(packet), EnumActionResult::Success, false)
                .withPredictedBlockState(activationPrediction);
        }
        if stack.isEmpty() {
            return BlockRightClickResult::new(Some(packet), EnumActionResult::Pass, false);
        }
        // MCP `PlayerControllerMP#processRightClickBlock`: the packet has
        // already been sent at this point, but item use is skipped while the
        // held item is on the player's CooldownTracker.
        if player.getCooldownTracker().hasCooldown(stack.itemId) {
            return BlockRightClickResult::new(Some(packet), EnumActionResult::Pass, false);
        }

        if ItemBlock::isItemBlock(stack) {
            // Player permission levels are not synchronized yet. Preserve the
            // vanilla safety branch by refusing command/structure ItemBlocks
            // rather than pretending a non-op client can place them.
            if matches!(stack.itemId as i32, 137 | 210 | 211 | 255) {
                return BlockRightClickResult::new(Some(packet), EnumActionResult::Fail, false);
            }
            let result = ItemBlock::predictOnItemUse(world, player, pos, hit.sideHit, stack);
            let placement = if result == EnumActionResult::Success {
                ItemBlock::placementPreview(
                    world,
                    player,
                    pos,
                    hit.sideHit,
                    (hit.hitVec.y - pos.y as f64) as f32,
                    stack,
                )
            } else {
                None
            };
            return BlockRightClickResult::new(
                Some(packet),
                result,
                result == EnumActionResult::Success,
            ).withPredictedPlacement(placement);
        }

        if ItemDoor::isItemDoor(stack) {
            let result = ItemDoor::predictOnItemUse(world, player, pos, hit.sideHit, stack);
            return BlockRightClickResult::new(
                Some(packet),
                result,
                result == EnumActionResult::Success,
            );
        }

        if ItemSign::isItemSign(stack) {
            let result = ItemSign::predictOnItemUse(world, player, pos, hit.sideHit, stack);
            return BlockRightClickResult::new(
                Some(packet),
                result,
                result == EnumActionResult::Success,
            );
        }

        if ItemSkull::isItemSkull(stack) {
            let result = ItemSkull::predictOnItemUse(world, player, pos, hit.sideHit, stack);
            return BlockRightClickResult::new(
                Some(packet),
                result,
                result == EnumActionResult::Success,
            );
        }

        // MCP `ItemHoe#onItemUse`: the remote client performs the target
        // checks and plays the till sound; durability and authoritative world
        // mutation stay server-side.
        if ItemHoe::isItemHoe(stack) {
            // MCP ItemHoe#onItemUse checks canPlayerEdit(pos.offset(side), side, stack)
            // before testing the tillable block.
            if !player.canPlayerEdit(world, pos.offset(hit.sideHit, 1), hit.sideHit, stack) {
                return BlockRightClickResult::new(Some(packet), EnumActionResult::Fail, false);
            }
            let (result, sound) = ItemHoe::predictOnItemUse(world, pos, hit.sideHit, stack);
            return BlockRightClickResult::new(Some(packet), result, false).withSound(sound);
        }

        // Concrete special-item onItemUse ports remain separate. PASS is
        // essential here: Minecraft may continue to the air-use branch and
        // then the off hand instead of consuming the click. Buckets do not
        // override onItemUse in 1.12.2; their logic runs through
        // ItemBucket#onItemRightClick below.
        BlockRightClickResult::new(Some(packet), EnumActionResult::Pass, false)
    }

    /// MCP `PlayerControllerMP#processRightClick`: sends the hand packet and
    /// evaluates the source-backed client branch of `Item#onItemRightClick`.
    pub fn processRightClick(
        &self,
        world: &WorldClient,
        player: &mut EntityPlayerSP,
        hand: EnumHand,
    ) -> AirRightClickResult {
        // MCP PlayerControllerMP#processRightClick returns PASS immediately in
        // spectator mode, before syncCurrentPlayItem/CPacketPlayerTryUseItem.
        if self.currentGameType == GameType::Spectator {
            return AirRightClickResult::new(None, EnumActionResult::Pass);
        }
        let packet = CPacketPlayerTryUseItem::new(hand).writePacketData();
        let stack = player.getHeldItem(hand).clone();
        // MCP sends CPacketPlayerTryUseItem first, then checks the item
        // cooldown before invoking Item#onItemRightClick.
        if !stack.isEmpty() && player.getCooldownTracker().hasCooldown(stack.itemId) {
            return AirRightClickResult::new(Some(packet), EnumActionResult::Pass);
        }
        match stack.itemId {
            crate::net::minecraft::item::ItemBucket::BUCKET => {
                match itemRayTrace(world, player, true) {
                    None => AirRightClickResult::new(Some(packet), EnumActionResult::Pass),
                    Some(hit) => {
                        let blockPos = hit.getBlockPos();
                        // MCP ItemBucket empty-bucket branch checks editing the
                        // adjacent side before consuming a liquid source.
                        if !player.canPlayerEdit(world, blockPos.offset(hit.sideHit, 1), hit.sideHit, &stack) {
                            return AirRightClickResult::new(Some(packet), EnumActionResult::Fail);
                        }
                        let target = (blockPos, world.getBlockState(blockPos));
                        match ItemBucket::predictFill(Some(target)) {
                            Some(fill) => AirRightClickResult {
                                packet: Some(packet), result: EnumActionResult::Success,
                                fillBucket: Some(fill), emptyBucket: None, thrown: None,
                            },
                            None => AirRightClickResult::new(Some(packet), EnumActionResult::Fail),
                        }
                    }
                }
            }
            crate::net::minecraft::item::ItemBucket::WATER_BUCKET
            | crate::net::minecraft::item::ItemBucket::LAVA_BUCKET => {
                match itemRayTrace(world, player, false) {
                    None => AirRightClickResult::new(Some(packet), EnumActionResult::Pass),
                    Some(hit) => match ItemBucket::predictEmpty(
                        world, hit.getBlockPos(), hit.sideHit, stack.itemId,
                    ) {
                        Some(empty) => {
                            // MCP full-bucket branch checks the actual liquid
                            // destination, not merely the originally hit block.
                            if !player.canPlayerEdit(world, empty.destination, hit.sideHit, &stack) {
                                AirRightClickResult::new(Some(packet), EnumActionResult::Fail)
                            } else {
                                AirRightClickResult {
                                    packet: Some(packet), result: EnumActionResult::Success,
                                    fillBucket: None, emptyBucket: Some(empty), thrown: None,
                                }
                            }
                        }
                        None => AirRightClickResult::new(Some(packet), EnumActionResult::Fail),
                    },
                }
            }
            // ItemSnowball / ItemEgg / ItemEnderPearl.
            332 | 344 | 368 => {
                let (sound, category) = match stack.itemId {
                    332 => ("entity.snowball.throw", SoundCategory::Neutral),
                    344 => ("entity.egg.throw", SoundCategory::Players),
                    _ => ("entity.enderpearl.throw", SoundCategory::Neutral),
                };
                let pitch = 0.4 / (Item::nextItemRandomF32() * 0.4 + 0.8);
                // MCP `ItemEnderPearl#onItemRightClick`: client and server
                // both place the pearl item on a 20-tick cooldown immediately.
                if stack.itemId == 368 {
                    player.getCooldownTrackerMut().setCooldown(368, 20);
                }
                AirRightClickResult {
                    packet: Some(packet), result: EnumActionResult::Success,
                    fillBucket: None, emptyBucket: None,
                    thrown: Some(Thrown { sound, category, pitch }),
                }
            }
            _ => AirRightClickResult::new(Some(packet), EnumActionResult::Pass),
        }
    }

    /// MCP `World.sendBlockBreakProgress` value for the local breaker. Stage
    /// `-1` is deliberately represented as `None`, so no destroy texture is
    /// drawn before the first positive hardness increment.
    pub fn getDestroyBlockProgress(&self, breakerId: i32, cloudTick: i32) -> Option<DestroyBlockProgress> {
        if !self.isHittingBlock {
            return None;
        }
        let stage = (self.curBlockDamageMP * 10.0) as i32 - 1;
        if !(0..10).contains(&stage) {
            return None;
        }
        let mut progress = DestroyBlockProgress::new(breakerId, self.currentBlock);
        progress.setPartialBlockDamage(stage);
        progress.setCloudUpdateTick(cloudTick);
        Some(progress)
    }

    pub fn takeBlockDestroyEffect(&mut self) -> Option<(BlockPos, IBlockState)> {
        self.pendingDestroyEffect.take()
    }

    pub fn takeBlockHitSound(&mut self) -> Option<LocalSoundEvent> {
        self.pendingHitSound.take()
    }

    pub const fn getIsHittingBlock(&self) -> bool { self.isHittingBlock }
}

/// Source-equivalent client-world state changes for concrete activation
/// implementations. This deliberately excludes blocks such as levers whose
/// MCP method returns early on `world.isRemote`, and iron doors/trapdoors whose
/// method returns false. Server block packets remain the final authority.
fn predictedActivationState(
    world: &WorldClient,
    player: &EntityPlayerSP,
    pos: BlockPos,
    state: IBlockState,
) -> Option<BlockStatePrediction> {
    if let Some((target, expectedState, predictedState)) =
        BlockDoor::onBlockActivatedState(world, pos, state)
    {
        return Some(BlockStatePrediction {
            pos: target,
            expectedState,
            state: predictedState,
        });
    }

    if let Some(predictedState) = BlockTrapDoor::onBlockActivatedState(state) {
        return Some(BlockStatePrediction {
            pos,
            expectedState: state,
            state: predictedState,
        });
    }

    if let Some(predictedState) =
        BlockFenceGate::onBlockActivatedState(state, player.entity.rotationYaw)
    {
        return Some(BlockStatePrediction {
            pos,
            expectedState: state,
            state: predictedState,
        });
    }

    if let Some(predictedState) = BlockButton::onBlockActivatedState(state) {
        return Some(BlockStatePrediction {
            pos,
            expectedState: state,
            state: predictedState,
        });
    }

    if let Some(predictedState) = BlockRedstoneComparator::onBlockActivatedState(state) {
        return Some(BlockStatePrediction {
            pos,
            expectedState: state,
            state: predictedState,
        });
    }

    if let Some(predictedState) = BlockRedstoneRepeater::onBlockActivatedState(state) {
        return Some(BlockStatePrediction {
            pos,
            expectedState: state,
            state: predictedState,
        });
    }

    if let Some(predictedState) = BlockJukebox::onBlockActivatedState(state) {
        return Some(BlockStatePrediction {
            pos,
            expectedState: state,
            state: predictedState,
        });
    }

    None
}

/// MCP `Item#rayTrace`: a fresh 5-block trace from the player's eyes.
fn itemRayTrace(
    world: &WorldClient,
    player: &EntityPlayerSP,
    useLiquids: bool,
) -> Option<RayTraceResult> {
    let eyes = player.getPositionEyes(1.0);
    let look = player.getLook(1.0);
    world.rayTraceBlocks(
        eyes,
        Vec3d::new(eyes.x + look.x * 5.0, eyes.y + look.y * 5.0, eyes.z + look.z * 5.0),
        useLiquids,
        !useLiquids,
        false,
    )
}

fn is_creative_sword(stack: &ItemStack) -> bool {
    !stack.isEmpty() && matches!(stack.itemId, 267 | 268 | 272 | 276 | 283)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::util::math::Vec3d::Vec3d;

    #[test]
    fn held_item_sync_only_emits_on_change() {
        let mut controller = PlayerControllerMP::new();
        let first = controller.syncCurrentPlayItem(0).expect("initial slot sync");
        assert_eq!(first.id, 0x1A);
        assert!(controller.syncCurrentPlayItem(0).is_none());
        assert_eq!(controller.syncCurrentPlayItem(8).unwrap().payload, vec![0, 8]);
    }

    #[test]
    fn creative_and_survival_reach_match_mcp() {
        let mut controller = PlayerControllerMP::new();
        assert_eq!(controller.getBlockReachDistance(), 4.5);
        controller.setGameType(GameType::Creative);
        assert_eq!(controller.getBlockReachDistance(), 5.0);
    }

    #[test]
    fn right_click_offsets_are_relative_to_hit_block() {
        let controller = PlayerControllerMP::new();
        let world = WorldClient::new(0);
        let player = EntityPlayerSP::new(1);
        let interaction = controller.processRightClickBlock(
            &world,
            &player,
            RayTraceResult::block(
                Vec3d::new(10.25, 64.5, -3.75),
                EnumFacing::Up,
                BlockPos::new(10, 64, -4),
            ),
            EnumHand::MainHand,
        );
        let packet = interaction.packet.expect("air block hit still sends the use-on-block packet");
        assert_eq!(interaction.result, EnumActionResult::Pass);
        assert_eq!(packet.id, 0x1F);
        assert_eq!(packet.payload.len(), 22);
    }

    #[test]
    fn wooden_door_activation_targets_and_toggles_lower_half() {
        let controller = PlayerControllerMP::new();
        let mut world = WorldClient::new(0);
        let player = EntityPlayerSP::new(1);
        let lowerPos = BlockPos::new(3, 64, 4);
        let lower = IBlockState::fromGlobalStateId((64 << 4) | 1);
        let upper = IBlockState::fromGlobalStateId((64 << 4) | 8);
        world.invalidateRegionAndSetBlock(lowerPos, lower).unwrap();
        world.invalidateRegionAndSetBlock(lowerPos.up(1), upper).unwrap();

        let result = controller.processRightClickBlock(
            &world,
            &player,
            RayTraceResult::block(
                Vec3d::new(3.5, 65.5, 4.5),
                EnumFacing::North,
                lowerPos.up(1),
            ),
            EnumHand::MainHand,
        );
        let prediction = result.predictedBlockState.expect("wood door predicts client toggle");
        assert_eq!(prediction.pos, lowerPos);
        assert_eq!(prediction.expectedState, lower);
        assert_eq!(prediction.state.getMetadata(), lower.getMetadata() ^ 4);
    }

    #[test]
    fn gate_activation_uses_player_facing_when_opening_from_back() {
        let controller = PlayerControllerMP::new();
        let mut world = WorldClient::new(0);
        let mut player = EntityPlayerSP::new(1);
        player.entity.rotationYaw = 0.0; // SOUTH
        let pos = BlockPos::new(0, 64, 0);
        let northFacingClosed = IBlockState::fromGlobalStateId((107 << 4) | 2);
        world.invalidateRegionAndSetBlock(pos, northFacingClosed).unwrap();

        let result = controller.processRightClickBlock(
            &world,
            &player,
            RayTraceResult::block(Vec3d::new(0.5, 64.5, 0.5), EnumFacing::North, pos),
            EnumHand::MainHand,
        );
        let prediction = result.predictedBlockState.expect("gate predicts client toggle");
        assert_eq!(prediction.state.getMetadata() & 3, 0); // SOUTH
        assert_ne!(prediction.state.getMetadata() & 4, 0);
    }

    #[test]
    fn comparator_and_repeater_cycle_exact_metadata_bits() {
        let controller = PlayerControllerMP::new();
        let mut world = WorldClient::new(0);
        let player = EntityPlayerSP::new(1);
        let comparatorPos = BlockPos::new(0, 64, 0);
        let repeaterPos = BlockPos::new(1, 64, 0);
        let comparator = IBlockState::fromGlobalStateId((149 << 4) | 10);
        let repeater = IBlockState::fromGlobalStateId((93 << 4) | 14);
        world.invalidateRegionAndSetBlock(comparatorPos, comparator).unwrap();
        world.invalidateRegionAndSetBlock(repeaterPos, repeater).unwrap();

        let comparatorResult = controller.processRightClickBlock(
            &world,
            &player,
            RayTraceResult::block(
                Vec3d::new(0.5, 64.5, 0.5),
                EnumFacing::Up,
                comparatorPos,
            ),
            EnumHand::MainHand,
        );
        assert_eq!(
            comparatorResult.predictedBlockState.unwrap().state.getMetadata(),
            14,
        );

        let repeaterResult = controller.processRightClickBlock(
            &world,
            &player,
            RayTraceResult::block(
                Vec3d::new(1.5, 64.5, 0.5),
                EnumFacing::Up,
                repeaterPos,
            ),
            EnumHand::MainHand,
        );
        assert_eq!(
            repeaterResult.predictedBlockState.unwrap().state.getMetadata(),
            2,
        );
    }

    #[test]
    fn occupied_jukebox_clears_has_record_on_client() {
        let controller = PlayerControllerMP::new();
        let mut world = WorldClient::new(0);
        let player = EntityPlayerSP::new(1);
        let pos = BlockPos::new(0, 64, 0);
        let occupied = IBlockState::fromGlobalStateId((84 << 4) | 1);
        world.invalidateRegionAndSetBlock(pos, occupied).unwrap();

        let result = controller.processRightClickBlock(
            &world,
            &player,
            RayTraceResult::block(Vec3d::new(0.5, 64.5, 0.5), EnumFacing::Up, pos),
            EnumHand::MainHand,
        );
        assert_eq!(result.result, EnumActionResult::Success);
        let prediction = result.predictedBlockState.expect("jukebox clears locally");
        assert_eq!(prediction.expectedState, occupied);
        assert_eq!(prediction.state.getMetadata(), 0);
    }

    #[test]
    fn lever_success_does_not_invent_remote_world_toggle() {
        let controller = PlayerControllerMP::new();
        let mut world = WorldClient::new(0);
        let player = EntityPlayerSP::new(1);
        let pos = BlockPos::new(0, 64, 0);
        world
            .invalidateRegionAndSetBlock(pos, IBlockState::fromGlobalStateId(69 << 4))
            .unwrap();

        let result = controller.processRightClickBlock(
            &world,
            &player,
            RayTraceResult::block(Vec3d::new(0.5, 64.5, 0.5), EnumFacing::North, pos),
            EnumHand::MainHand,
        );
        assert_eq!(result.result, EnumActionResult::Success);
        assert!(result.predictedBlockState.is_none());
    }
}
