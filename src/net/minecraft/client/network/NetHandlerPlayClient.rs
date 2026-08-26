use crate::net::minecraft::block::BlockBed::BlockBed;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
    use crate::net::minecraft::block::SoundType::SoundType;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use crate::compat::Java::JavaRandom;

use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP;
use crate::net::minecraft::client::audio::LocalSoundEvent::LocalSoundEvent;
use crate::net::minecraft::client::entity::EntityOtherPlayerMP::EntityOtherPlayerMP;
use crate::net::minecraft::client::entity::EntityOtherClient::{
    ClientEntityKind, EntityOtherClient, MobEntityType, ObjectSpawnType,
};
use crate::net::minecraft::client::network::NetworkPlayerInfo::NetworkPlayerInfo;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::client::renderer::DestroyBlockProgress::DestroyBlockProgress;
use crate::net::minecraft::client::particle::ParticleSpawnRequest::ParticleSpawnRequest;
use crate::net::minecraft::entity::player::EntityPlayer::EnumChatVisibility;
use crate::net::minecraft::network::NetworkManager::{NetworkManager, NetworkManagerError};
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::item::ItemBlock::ItemBlockPlacement;
use crate::net::minecraft::inventory::ClickType::ClickType;
use crate::net::minecraft::inventory::Container::Container;
use crate::net::minecraft::inventory::ContainerChest::ContainerChest;
use crate::net::minecraft::inventory::ContainerShulkerBox::ContainerShulkerBox;
use crate::net::minecraft::inventory::ContainerEnchantment::ContainerEnchantment;
use crate::net::minecraft::inventory::ContainerFurnace::ContainerFurnace;
use crate::net::minecraft::inventory::ContainerRepair::ContainerRepair;
use crate::net::minecraft::inventory::ContainerWorkbench::ContainerWorkbench;
use crate::net::minecraft::inventory::ContainerHopper::ContainerHopper;
use crate::net::minecraft::inventory::ContainerMerchant::ContainerMerchant;
use crate::net::minecraft::inventory::ContainerHorseInventory::{ContainerHorseInventory, HorseInventoryKind, HorseInventorySpec};
use crate::net::minecraft::inventory::ContainerBrewingStand::ContainerBrewingStand;
use crate::net::minecraft::inventory::ContainerDispenser::ContainerDispenser;
use crate::net::minecraft::inventory::ContainerBeacon::ContainerBeacon;
use crate::net::minecraft::inventory::OpenContainer::OpenContainer;
use crate::net::minecraft::network::play::client::CPacketClickWindow::CPacketClickWindow;
use crate::net::minecraft::network::play::client::CPacketCreativeInventoryAction::CPacketCreativeInventoryAction;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::write_string;
use crate::net::minecraft::network::play::client::CPacketClientSettings::CPacketClientSettings;
use crate::net::minecraft::network::play::client::CPacketConfirmTeleport::CPacketConfirmTeleport;
use crate::net::minecraft::network::play::client::CPacketConfirmTransaction::CPacketConfirmTransaction;
use crate::net::minecraft::network::play::client::CPacketCustomPayload::CPacketCustomPayload;
use crate::net::minecraft::network::play::client::CPacketKeepAlive::CPacketKeepAlive;
use crate::net::minecraft::network::play::client::CPacketSteerBoat::CPacketSteerBoat;
use crate::net::minecraft::network::play::client::CPacketVehicleMove::CPacketVehicleMove;
use crate::net::minecraft::network::play::client::CPacketPlayer::PositionRotation;
use crate::net::minecraft::network::play::server::SPacketBlockAction::SPacketBlockAction;
use crate::net::minecraft::network::play::server::SPacketBlockChange::SPacketBlockChange;
use crate::net::minecraft::network::play::server::SPacketBlockBreakAnim::SPacketBlockBreakAnim;
use crate::net::minecraft::network::play::server::SPacketAnimation::SPacketAnimation;
use crate::net::minecraft::network::play::server::SPacketDestroyEntities::SPacketDestroyEntities;
use crate::net::minecraft::network::play::server::SPacketRemoveEntityEffect::SPacketRemoveEntityEffect;
use crate::net::minecraft::network::play::server::SPacketEntity::SPacketEntity;
use crate::net::minecraft::network::play::server::SPacketEntityHeadLook::SPacketEntityHeadLook;
use crate::net::minecraft::network::play::server::SPacketEntityMetadata::SPacketEntityMetadata;
use crate::net::minecraft::network::play::server::SPacketEntityTeleport::SPacketEntityTeleport;
use crate::net::minecraft::network::play::server::SPacketEntityVelocity::SPacketEntityVelocity;
use crate::net::minecraft::network::play::server::SPacketEntityProperties::SPacketEntityProperties;
use crate::net::minecraft::network::play::server::SPacketEntityEffect::SPacketEntityEffect;
use crate::net::minecraft::network::play::server::SPacketPlayerListItem::{Action as PlayerListAction, SPacketPlayerListItem};
use crate::net::minecraft::network::play::server::SPacketPlayerListHeaderFooter::SPacketPlayerListHeaderFooter;
use crate::net::minecraft::network::play::server::SPacketTabComplete::SPacketTabComplete;
use crate::net::minecraft::network::play::server::SPacketTitle::SPacketTitle;
use crate::net::minecraft::network::play::server::SPacketUpdateBossInfo::SPacketUpdateBossInfo;
use crate::net::minecraft::network::play::server::SPacketChat::SPacketChat;
use crate::net::minecraft::network::play::server::SPacketCombatEvent::{Event as CombatEvent, SPacketCombatEvent};
use crate::net::minecraft::network::play::server::SPacketCustomSound::SPacketCustomSound;
use crate::net::minecraft::network::play::server::SPacketEffect::SPacketEffect;
use crate::net::minecraft::network::play::server::SPacketSoundEffect::SPacketSoundEffect;
use crate::net::minecraft::network::play::server::SPacketDisplayObjective::SPacketDisplayObjective;
use crate::net::minecraft::network::play::server::SPacketScoreboardObjective::SPacketScoreboardObjective;
use crate::net::minecraft::network::play::server::SPacketTeams::SPacketTeams;
use crate::net::minecraft::network::play::server::SPacketUpdateScore::{Action as UpdateScoreAction, SPacketUpdateScore};
use crate::net::minecraft::network::play::server::SPacketSpawnPlayer::SPacketSpawnPlayer;
use crate::net::minecraft::network::play::server::SPacketSpawnObject::SPacketSpawnObject;
use crate::net::minecraft::network::play::server::SPacketSpawnExperienceOrb::SPacketSpawnExperienceOrb;
use crate::net::minecraft::network::play::server::SPacketSpawnGlobalEntity::SPacketSpawnGlobalEntity;
use crate::net::minecraft::network::play::server::SPacketSpawnMob::SPacketSpawnMob;
use crate::net::minecraft::network::play::server::SPacketSpawnPainting::SPacketSpawnPainting;
use crate::net::minecraft::network::play::server::SPacketWindowItems::SPacketWindowItems;
use crate::net::minecraft::network::play::server::SPacketWindowProperty::SPacketWindowProperty;
use crate::net::minecraft::network::play::server::SPacketOpenWindow::SPacketOpenWindow;
use crate::net::minecraft::network::play::server::SPacketCloseWindow::SPacketCloseWindow;
use crate::net::minecraft::network::play::server::SPacketSetSlot::SPacketSetSlot;
use crate::net::minecraft::network::play::server::SPacketEntityStatus::SPacketEntityStatus;
use crate::net::minecraft::network::play::server::SPacketHeldItemChange::SPacketHeldItemChange;
use crate::net::minecraft::network::play::server::SPacketEntityAttach::SPacketEntityAttach;
use crate::net::minecraft::network::play::server::SPacketEntityEquipment::SPacketEntityEquipment;
use crate::net::minecraft::network::play::server::SPacketUpdateHealth::SPacketUpdateHealth;
use crate::net::minecraft::network::play::server::SPacketSetExperience::SPacketSetExperience;
use crate::net::minecraft::network::play::server::SPacketSignEditorOpen::SPacketSignEditorOpen;
use crate::net::minecraft::network::play::server::SPacketPlayerAbilities::SPacketPlayerAbilities;
use crate::net::minecraft::network::play::server::SPacketChangeGameState::SPacketChangeGameState;
use crate::net::minecraft::network::play::server::SPacketServerDifficulty::SPacketServerDifficulty;
use crate::net::minecraft::network::play::server::SPacketSpawnPosition::SPacketSpawnPosition;
use crate::net::minecraft::network::play::server::SPacketCooldown::SPacketCooldown;
use crate::net::minecraft::network::play::server::SPacketSetPassengers::SPacketSetPassengers;
use crate::net::minecraft::network::play::server::SPacketMoveVehicle::SPacketMoveVehicle;
use crate::net::minecraft::network::play::server::SPacketMaps::SPacketMaps;
use crate::net::minecraft::network::play::server::SPacketParticles::SPacketParticles;
use crate::net::minecraft::network::play::server::SPacketUseBed::SPacketUseBed;
use crate::net::minecraft::network::play::server::SPacketCustomPayload::SPacketCustomPayload;
use crate::net::minecraft::network::play::server::SPacketRecipeBook::{SPacketRecipeBook, State as RecipeBookState};
use crate::net::minecraft::network::play::server::SPacketPlaceGhostRecipe::SPacketPlaceGhostRecipe;
use crate::net::minecraft::village::MerchantRecipeList::MerchantRecipeList;
use crate::net::minecraft::inventory::EntityEquipmentSlot::EntityEquipmentSlot;
use crate::net::minecraft::entity::effect::EntityLightningBolt::EntityLightningBolt;
use crate::net::minecraft::network::play::server::SPacketChunkData::SPacketChunkData;
use crate::net::minecraft::network::play::server::SPacketConfirmTransaction::SPacketConfirmTransaction;
use crate::net::minecraft::network::play::server::SPacketDisconnect::SPacketDisconnect;
use crate::net::minecraft::network::play::server::SPacketJoinGame::SPacketJoinGame;
use crate::net::minecraft::network::play::server::SPacketRespawn::SPacketRespawn;
use crate::net::minecraft::network::play::server::SPacketKeepAlive::SPacketKeepAlive;
use crate::net::minecraft::network::play::server::SPacketMultiBlockChange::SPacketMultiBlockChange;
use crate::net::minecraft::network::play::server::SPacketPlayerPosLook::{
    EnumFlags, SPacketPlayerPosLook,
};
use crate::net::minecraft::network::play::server::SPacketUnloadChunk::SPacketUnloadChunk;
use crate::net::minecraft::network::play::server::SPacketTimeUpdate::SPacketTimeUpdate;
use crate::net::minecraft::network::play::server::SPacketUpdateTileEntity::SPacketUpdateTileEntity;
use crate::net::minecraft::util::EnumHand::EnumHand;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::util::SoundCategory::SoundCategory;
use crate::net::minecraft::util::EnumParticleTypes::EnumParticleTypes;
use crate::net::minecraft::util::EnumHandSide::EnumHandSide;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::MovementInputFromOptions::MovementKeyState;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;
use crate::net::minecraft::util::text::ChatType::ChatType;
use crate::net::minecraft::scoreboard::Scoreboard::Scoreboard;
use crate::net::minecraft::world::GameType::GameType;
use crate::net::minecraft::world::storage::MapData::MapData;
use crate::net::minecraft::potion::PotionEffect::PotionEffect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSettingsSnapshot {
    pub language: String,
    pub renderDistanceChunks: i32,
    pub chatVisibility: EnumChatVisibility,
    pub chatColours: bool,
    pub modelPartFlags: u8,
    pub mainHand: EnumHandSide,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerPositionState {
    pub posX: f64,
    pub posY: f64,
    pub posZ: f64,
    pub rotationYaw: f32,
    pub rotationPitch: f32,
    pub eyeHeight: f32,
}

impl Default for PlayerPositionState {
    fn default() -> Self {
        Self {
            posX: 0.0,
            posY: 0.0,
            posZ: 0.0,
            rotationYaw: 0.0,
            rotationPitch: 0.0,
            eyeHeight: 1.62,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceivedChatMessage {
    pub serial: u64,
    pub component: ITextComponent,
    pub chatType: ChatType,
}

#[derive(Debug, Clone)]
pub struct PlayClientState {
    pub worldClient: Option<WorldClient>,
    pub thePlayer: Option<EntityPlayerSP>,
    /// Authenticated profile supplied by Login Success. This can differ from
    /// the launcher Session profile when a protocol proxy performs the online
    /// login, so all local-player texture lookup must prefer this identity.
    pub localGameProfile: Option<GameProfile>,
    /// Cached `AbstractClientPlayer#playerInfo` for the local EntityPlayerSP.
    /// It intentionally survives REMOVE_PLAYER just like the Java field.
    pub localPlayerInfo: Option<NetworkPlayerInfo>,
    pub playerPosition: PlayerPositionState,
    pub doneLoadingTerrain: bool,
    pub currentServerMaxPlayers: u8,
    pub gameType: GameType,
    pub hardcoreMode: bool,
    /// `NetworkManager#isEncrypted` snapshot. The tab overlay uses this exact
    /// vanilla gate before showing downloaded player heads.
    pub networkEncrypted: bool,
    pub playerInfoMap: HashMap<Uuid, NetworkPlayerInfo>,
    pub playerListHeader: Option<ITextComponent>,
    pub playerListFooter: Option<ITextComponent>,
    pub chatMessages: Vec<ReceivedChatMessage>,
    pub tabCompleteMatches: Vec<Vec<String>>,
    pub pendingTitlePackets: Vec<SPacketTitle>,
    pub pendingBossInfoPackets: Vec<SPacketUpdateBossInfo>,
    pub nextChatSerial: u64,
    pub actionBarMessage: Option<ITextComponent>,
    pub actionBarUpdatedTick: i32,
    pub scoreboard: Scoreboard,
    /// Rust equivalent of `RenderGlobal.cloudTickCounter`, advanced once per
    /// world client tick and used for damaged-block expiry timestamps.
    pub cloudTickCounter: i32,
    pub damagedBlocks: HashMap<i32, DestroyBlockProgress>,
    /// MCP `EntityPlayerSP#openEditSign` hand-off. The network handler records
    /// the exact server-selected sign position; Minecraft consumes it when the
    /// concrete GuiEditSign/TileEntitySign lifecycle is available.
    pub pendingSignEditorPosition: Option<BlockPos>,
    /// Server-requested ghost placement for the active recipe-book GUI.
    /// `GuiRecipeBook` consumes this only when its container window matches.
    pub pendingGhostRecipe: Option<(i8, i32)>,
    /// Client-side `World#getMapData("map_<id>")` storage updated by SPacketMaps.
    pub mapData: HashMap<i32, MapData>,
    pub revision: u64,
}

impl Default for PlayClientState {
    fn default() -> Self {
        Self {
            worldClient: None,
            thePlayer: None,
            localGameProfile: None,
            localPlayerInfo: None,
            playerPosition: PlayerPositionState::default(),
            doneLoadingTerrain: false,
            currentServerMaxPlayers: 20,
            gameType: GameType::NotSet,
            hardcoreMode: false,
            networkEncrypted: false,
            playerInfoMap: HashMap::new(),
            playerListHeader: None,
            playerListFooter: None,
            chatMessages: Vec::new(),
            tabCompleteMatches: Vec::new(),
            pendingTitlePackets: Vec::new(),
            pendingBossInfoPackets: Vec::new(),
            nextChatSerial: 0,
            actionBarMessage: None,
            actionBarUpdatedTick: 0,
            scoreboard: Scoreboard::new(),
            cloudTickCounter: 0,
            damagedBlocks: HashMap::new(),
            pendingSignEditorPosition: None,
            pendingGhostRecipe: None,
            mapData: HashMap::new(),
            revision: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreativePlayerContainerClick {
    pub beforeCursor: ItemStack,
    pub afterCursor: ItemStack,
    pub beforeSlots: Vec<ItemStack>,
    pub afterSlots: Vec<ItemStack>,
    pub originalSlotStack: ItemStack,
    pub quickCraftFinished: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SharedPlayClientState {
    inner: Arc<RwLock<PlayClientState>>,
}

impl SharedPlayClientState {
    pub fn new() -> Self { Self::default() }

    pub fn snapshot(&self) -> PlayClientState {
        self.inner
            .read()
            .unwrap_or_else(|poison| poison.into_inner())
            .clone()
    }

    pub fn withRead<R>(&self, operation: impl FnOnce(&PlayClientState) -> R) -> R {
        let state = self
            .inner
            .read()
            .unwrap_or_else(|poison| poison.into_inner());
        operation(&state)
    }

    /// Write-locked counterpart used when an input event must preserve one
    /// atomic MCP interaction sequence across WorldClient reads and local
    /// EntityPlayerSP state changes (for example rightClickMouse hand order).
    pub fn withWrite<R>(&self, operation: impl FnOnce(&mut PlayClientState) -> R) -> R {
        let mut state = self
            .inner
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        operation(&mut state)
    }

    pub fn revision(&self) -> u64 {
        self.inner
            .read()
            .unwrap_or_else(|poison| poison.into_inner())
            .revision
    }

    /// `RenderGlobal#drawBlockDamageTexture` removes remote progress records
    /// once their block is more than 32 blocks from the interpolated render
    /// entity. Keeping the mutation here preserves that render-time lifecycle
    /// even though Vulkan capture otherwise uses a read lock.
    pub fn pruneDamagedBlocksForRender(&self, partialTicks: f32) {
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        let Some(player) = state.thePlayer.as_ref() else { return; };
        let partial = partialTicks.clamp(0.0, 1.0) as f64;
        let x = player.entity.prevPosX + (player.entity.posX - player.entity.prevPosX) * partial;
        let y = player.entity.prevPosY + (player.entity.posY - player.entity.prevPosY) * partial;
        let z = player.entity.prevPosZ + (player.entity.posZ - player.entity.prevPosZ) * partial;
        let before = state.damagedBlocks.len();
        state.damagedBlocks.retain(|_, progress| {
            let position = progress.getPosition();
            let dx = position.x as f64 - x;
            let dy = position.y as f64 - y;
            let dz = position.z as f64 - z;
            dx * dx + dy * dy + dz * dz <= 1024.0
        });
        if state.damagedBlocks.len() != before {
            state.revision = state.revision.wrapping_add(1);
        }
    }

    pub fn takePendingSignEditorPosition(&self) -> Option<BlockPos> {
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        state.pendingSignEditorPosition.take()
    }

    pub fn currentHotbarSlot(&self) -> Option<i32> {
        self.inner
            .read()
            .unwrap_or_else(|poison| poison.into_inner())
            .thePlayer
            .as_ref()
            .map(|player| player.inventory.currentItem)
    }

    pub fn setCurrentHotbarSlot(&self, index: i32) -> bool {
        if !(0..9).contains(&index) {
            return false;
        }
        let mut state = self
            .inner
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        let Some(player) = state.thePlayer.as_mut() else {
            return false;
        };
        if player.inventory.currentItem == index {
            return false;
        }
        player.inventory.currentItem = index;
        state.revision = state.revision.wrapping_add(1);
        true
    }

    pub fn queueLocalPlayerSound(&self, sound: LocalSoundEvent) -> bool {
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        let Some(player) = state.thePlayer.as_mut() else { return false; };
        player.queueSoundEvent(sound);
        true
    }

    /// Drains all client-originated sounds after the world/player tick. This
    /// mirrors SoundHandler ownership without mixing them into packet events.
    pub fn takeLocalPlayerSoundEvents(&self) -> Vec<LocalSoundEvent> {
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        let mut sounds = state.thePlayer.as_mut().map_or_else(Vec::new, EntityPlayerSP::takeSoundEvents);
        if let Some(world) = state.worldClient.as_mut() {
            sounds.extend(world.takeSoundEvents());
        }
        sounds
    }

    pub fn closeOpenContainer(&self, windowId: i32) -> bool {
        let mut state = self
            .inner
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        let Some(player) = state.thePlayer.as_mut() else { return false; };
        if !player.openContainer.as_ref().is_some_and(|container| container.windowId() == windowId) {
            return false;
        }
        player.openContainer = None;
        player.inventory.setItemStack(ItemStack::EMPTY);
        state.revision = state.revision.wrapping_add(1);
        true
    }

    /// Local half of MCP `GuiContainerCreative#handleMouseClick` for a slot
    /// backed by the temporary 45-slot creative inventory. Catalog clicks do
    /// not use CPacketClickWindow; only direct hotbar replacement and drops
    /// produce `CPacketCreativeInventoryAction`.
    pub fn clickCreativeCatalogStack(
        &self,
        catalogStack: &ItemStack,
        mouseButton: i32,
        clickType: ClickType,
    ) -> Result<Vec<RawPacket>, crate::net::minecraft::network::PacketBuffer::CodecError> {
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        if state.gameType != GameType::Creative { return Ok(Vec::new()); }
        let Some(player) = state.thePlayer.as_mut() else { return Ok(Vec::new()); };
        let mut packets = Vec::new();
        let mut cursor = player.inventory.getItemStack().clone();
        let source = catalogStack.clone();

        match clickType {
            ClickType::Swap => {
                if !source.isEmpty() && (0..9).contains(&mouseButton) {
                    let mut replacement = source.copy();
                    replacement.setCount(replacement.getMaxStackSize());
                    player.inventory.setInventorySlotContents(mouseButton, replacement.clone())?;
                    player.inventoryContainer.putStackInSlot(36 + mouseButton, replacement.clone())?;
                    packets.push(CPacketCreativeInventoryAction::new(36 + mouseButton, &replacement).writePacketData()?);
                }
            }
            ClickType::Clone => {
                if cursor.isEmpty() && !source.isEmpty() {
                    let mut clone = source.copy();
                    clone.setCount(clone.getMaxStackSize());
                    player.inventory.setItemStack(clone);
                }
            }
            ClickType::Throw => {
                if !source.isEmpty() {
                    let mut dropped = source.copy();
                    dropped.setCount(if mouseButton == 0 { 1 } else { dropped.getMaxStackSize() });
                    packets.push(CPacketCreativeInventoryAction::new(-1, &dropped).writePacketData()?);
                }
            }
            ClickType::Pickup | ClickType::QuickMove => {
                let quickMove = clickType == ClickType::QuickMove;
                if !cursor.isEmpty()
                    && !source.isEmpty()
                    && cursor.isItemEqual(&source)
                    && ItemStack::areItemStackTagsEqual(&cursor, &source)
                {
                    if mouseButton == 0 {
                        if quickMove {
                            cursor.setCount(cursor.getMaxStackSize());
                        } else if cursor.getCount() < cursor.getMaxStackSize() {
                            cursor.grow(1);
                        }
                    } else {
                        cursor.shrink(1);
                    }
                } else if !source.isEmpty() && cursor.isEmpty() {
                    cursor = source.copy();
                    if quickMove { cursor.setCount(cursor.getMaxStackSize()); }
                } else if mouseButton == 0 {
                    cursor = ItemStack::EMPTY;
                } else {
                    cursor.shrink(1);
                }
                player.inventory.setItemStack(cursor);
            }
            ClickType::QuickCraft | ClickType::PickupAll => {}
        }
        state.revision = state.revision.wrapping_add(1);
        Ok(packets)
    }

    /// MCP creative outside click: drop the whole cursor stack with left click
    /// or split and drop one item with right click.
    pub fn dropCreativeCursor(
        &self,
        mouseButton: i32,
    ) -> Result<Option<RawPacket>, crate::net::minecraft::network::PacketBuffer::CodecError> {
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        if state.gameType != GameType::Creative { return Ok(None); }
        let Some(player) = state.thePlayer.as_mut() else { return Ok(None); };
        let mut cursor = player.inventory.getItemStack().clone();
        if cursor.isEmpty() { return Ok(None); }
        let dropped = if mouseButton == 0 {
            let complete = cursor.clone();
            cursor = ItemStack::EMPTY;
            complete
        } else if mouseButton == 1 {
            cursor.splitStack(1)
        } else {
            return Ok(None);
        };
        player.inventory.setItemStack(cursor);
        state.revision = state.revision.wrapping_add(1);
        CPacketCreativeInventoryAction::new(-1, &dropped).writePacketData().map(Some)
    }

    pub fn clearCreativeCursor(&self) -> bool {
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        if state.gameType != GameType::Creative { return false; }
        let Some(player) = state.thePlayer.as_mut() else { return false; };
        if player.inventory.getItemStack().isEmpty() { return false; }
        player.inventory.setItemStack(ItemStack::EMPTY);
        state.revision = state.revision.wrapping_add(1);
        true
    }

    /// Shift-clicking the creative destroy slot sends one empty creative action
    /// for every `ContainerPlayer#getInventory` entry, matching the Java loop.
    pub fn clearCreativePlayerContainer(
        &self,
    ) -> Result<Vec<RawPacket>, crate::net::minecraft::network::PacketBuffer::CodecError> {
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        if state.gameType != GameType::Creative { return Ok(Vec::new()); }
        let Some(player) = state.thePlayer.as_mut() else { return Ok(Vec::new()); };
        let mut packets = Vec::with_capacity(46);
        for slot in 0..46 {
            player.inventoryContainer.putStackInSlot(slot, ItemStack::EMPTY)?;
            if slot != 0 {
                player.inventory.applyContainerPlayerSlot(slot, ItemStack::EMPTY)?;
            }
            packets.push(CPacketCreativeInventoryAction::new(slot, &ItemStack::EMPTY).writePacketData()?);
        }
        player.inventory.syncFromContainerPlayer(&player.inventoryContainer);
        state.revision = state.revision.wrapping_add(1);
        Ok(packets)
    }

    /// Snapshot used to translate a normal local `ContainerPlayer#slotClick`
    /// prediction into the creative packet stream without sending a forbidden
    /// CPacketClickWindow for window 0.
    pub fn creativePlayerContainerSnapshot(&self) -> Option<(ItemStack, Vec<ItemStack>)> {
        self.withRead(|state| {
            let player = state.thePlayer.as_ref()?;
            Some((player.inventory.getItemStack().clone(), player.inventoryContainer.slots().to_vec()))
        })
    }

    pub fn changeCurrentHotbarItem(&self, direction: i32) -> bool {
        if direction == 0 {
            return false;
        }
        let mut state = self
            .inner
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        let Some(player) = state.thePlayer.as_mut() else {
            return false;
        };
        let previous = player.inventory.currentItem;
        player.inventory.changeCurrentItem(direction);
        if player.inventory.currentItem == previous {
            return false;
        }
        state.revision = state.revision.wrapping_add(1);
        true
    }

    /// Applies the remote-world half of MCP `ItemBlock#onItemUse` after the
    /// use-on-block packet has been queued. Only source-backed placement states
    /// reach this method. A second replaceability check and held-stack identity
    /// guard prevent stale input snapshots from overwriting a server update.
    pub fn applyPredictedItemBlockPlacement(
        &self,
        placement: ItemBlockPlacement,
        hand: EnumHand,
    ) -> bool {
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        let heldMatches = state.thePlayer.as_ref().is_some_and(|player| {
            let held = player.getHeldItem(hand);
            !held.isEmpty()
                && held.itemId == placement.sourceItemId
                && held.itemDamage == placement.sourceItemDamage
        });
        if !heldMatches { return false; }

        let applied = match state.worldClient.as_mut() {
            Some(world) if world.isBlockReplaceable(placement.pos) => world
                .invalidateRegionAndSetBlock(placement.pos, placement.state)
                .is_ok(),
            _ => false,
        };
        if !applied { return false; }

        if state.gameType != GameType::Creative {
            let Some(player) = state.thePlayer.as_mut() else { return false; };
            // The identity guard above makes failure here possible only if the
            // local inventory structure is malformed; the world prediction is
            // still left for the authoritative server packet to correct.
            let _ = player.consumeHeldItemForPlacement(
                hand,
                placement.sourceItemId,
                placement.sourceItemDamage,
            );
        }
        // MCP ItemBlock#onItemUse invokes EntityPlayer#playSound after the
        // remote-world setBlockState succeeds. WorldClient accepts only the
        // local player here, so this is a client-owned sound rather than a
        // server SPacketSoundEffect.
        let soundType = SoundType::forBlockId(placement.state.getBlockId());
        if let Some(player) = state.thePlayer.as_mut() {
            player.queueSoundAt(
                soundType.getPlaceSound().to_string(),
                SoundCategory::Blocks,
                [
                    placement.pos.x as f32 + 0.5,
                    placement.pos.y as f32 + 0.5,
                    placement.pos.z as f32 + 0.5,
                ],
                (soundType.getVolume() + 1.0) / 2.0,
                soundType.getPitch() * 0.8,
            );
        }
        state.revision = state.revision.wrapping_add(1);
        true
    }

    /// Client-side world mutation from MCP
    /// `PlayerControllerMP#onPlayerDestroyBlock`. The expected-state guard is
    /// the Rust equivalent of operating on the same interaction snapshot; a
    /// newer authoritative block packet wins instead of being overwritten.
    pub fn applyPredictedBlockDestruction(
        &self,
        pos: BlockPos,
        expectedState: IBlockState,
    ) -> bool {
        if expectedState.isAir() {
            return false;
        }
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        let applied = match state.worldClient.as_mut() {
            Some(world) if world.getBlockState(pos) == expectedState => world
                .invalidateRegionAndSetBlock(pos, IBlockState::fromGlobalStateId(0))
                .is_ok(),
            _ => false,
        };
        if applied {
            state.revision = state.revision.wrapping_add(1);
        }
        applied
    }

    /// Applies a concrete source-backed `Block#onBlockActivated` client-world
    /// mutation after its use-on-block packet has been sent. The exact-state
    /// guard is required because network handling may have already installed
    /// a newer authoritative state between input sampling and packet send.
    pub fn applyPredictedBlockState(
        &self,
        pos: BlockPos,
        expectedState: IBlockState,
        predictedState: IBlockState,
    ) -> bool {
        if expectedState == predictedState {
            return false;
        }
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        let applied = match state.worldClient.as_mut() {
            Some(world) if world.getBlockState(pos) == expectedState => world
                .invalidateRegionAndSetBlock(pos, predictedState)
                .is_ok(),
            _ => false,
        };
        if applied {
            state.revision = state.revision.wrapping_add(1);
        }
        applied
    }

    pub fn swingLocalArm(&self, hand: EnumHand) -> bool {
        let mut state = self
            .inner
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        let Some(player) = state.thePlayer.as_mut() else {
            return false;
        };
        player.swingArm(hand);
        state.revision = state.revision.wrapping_add(1);
        true
    }

    pub fn resetLocalAttackCooldown(&self) -> bool {
        let mut state = self
            .inner
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        let Some(player) = state.thePlayer.as_mut() else {
            return false;
        };
        player.resetCooldown();
        state.revision = state.revision.wrapping_add(1);
        true
    }

    /// Successful knockback branch of
    /// `EntityPlayer#attackTargetEntityWithCurrentItem`: the attacker loses
    /// forty percent of horizontal velocity and sprinting is cancelled. This
    /// is intentionally separate from the server packet; vanilla performs the
    /// same mutation on the local EntityPlayer before the next movement tick.
    pub fn applyLocalAttackKnockbackSlowdown(&self) -> bool {
        let mut state = self
            .inner
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        let Some(player) = state.thePlayer.as_mut() else {
            return false;
        };
        player.entity.motionX *= 0.6_f64;
        player.entity.motionZ *= 0.6_f64;
        player.setSprinting(false);
        state.revision = state.revision.wrapping_add(1);
        true
    }

    pub fn startUsingHeldItemExact(&self, hand: EnumHand) -> bool {
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        let gameType = state.gameType;
        let Some(player) = state.thePlayer.as_mut() else { return false; };
        if !try_start_using_held_item(player, gameType, hand) {
            return false;
        }
        state.revision = state.revision.wrapping_add(1);
        true
    }

    pub fn startUsingHeldItem(&self, preferredHand: EnumHand) -> Option<EnumHand> {
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        let gameType = state.gameType;
        let player = state.thePlayer.as_mut()?;
        let hands = match preferredHand {
            EnumHand::MainHand => [EnumHand::MainHand, EnumHand::OffHand],
            EnumHand::OffHand => [EnumHand::OffHand, EnumHand::MainHand],
        };
        for hand in hands {
            if try_start_using_held_item(player, gameType, hand) {
                state.revision = state.revision.wrapping_add(1);
                return Some(hand);
            }
        }
        None
    }

    pub fn stopUsingItem(&self) -> bool {
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        let Some(player) = state.thePlayer.as_mut() else { return false; };
        if !player.isHandActive() { return false; }
        player.stopActiveHand();
        state.revision = state.revision.wrapping_add(1);
        true
    }

    /// Client-side `Container.slotClick` prediction followed by the exact
    /// protocol-340 `CPacketClickWindow`. The server remains authoritative and
    /// subsequent WindowItems/SetSlot packets replace this predicted state.
    pub fn clickPlayerInventorySlot(
        &self,
        slotId: i32,
        mouseButton: i32,
        clickType: ClickType,
    ) -> Result<Option<RawPacket>, crate::net::minecraft::network::PacketBuffer::CodecError> {
        use crate::net::minecraft::network::PacketBuffer::CodecError;

        if !valid_player_container_click_button(mouseButton, clickType) { return Ok(None); }

        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        let gameType = state.gameType;
        let Some(player) = state.thePlayer.as_mut() else { return Ok(None); };
        if clickType == ClickType::Clone && gameType != GameType::Creative { return Ok(None); }
        if slotId < -999 || slotId >= crate::net::minecraft::inventory::ContainerPlayer::ContainerPlayer::SLOT_COUNT as i32 {
            return Err(CodecError::InvalidData(format!("player-container slot {slotId} outside -999 or 0..45")));
        }

        // Only ordinary window clicks consume Container.transactionID. The
        // creative screen calls the same local ContainerPlayer#slotClick body
        // directly and reports changes with CPacketCreativeInventoryAction.
        let actionNumber = player.inventoryContainer.getNextTransactionID();
        let Some(clickedItem) = apply_player_container_click(
            player,
            gameType,
            slotId,
            mouseButton,
            clickType,
        )? else {
            return Ok(None);
        };
        state.revision = state.revision.wrapping_add(1);
        CPacketClickWindow::new(
            0,
            slotId,
            mouseButton,
            clickType,
            &clickedItem,
            actionNumber,
        ).writePacketData().map(Some)
    }

    /// Runs `EntityPlayer.inventoryContainer.slotClick` for
    /// `GuiContainerCreative` without creating a CPacketClickWindow and,
    /// critically, without advancing the normal container transaction ID.
    /// The caller reproduces CreativeCrafting/sendSlotPacket from the returned
    /// before/after snapshots using CPacketCreativeInventoryAction.
    pub fn clickCreativePlayerInventorySlot(
        &self,
        slotId: i32,
        mouseButton: i32,
        clickType: ClickType,
        inventoryTab: bool,
    ) -> Result<Option<CreativePlayerContainerClick>, crate::net::minecraft::network::PacketBuffer::CodecError> {
        use crate::net::minecraft::network::PacketBuffer::CodecError;

        if !valid_player_container_click_button(mouseButton, clickType) { return Ok(None); }

        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        if state.gameType != GameType::Creative { return Ok(None); }
        let gameType = state.gameType;
        let Some(player) = state.thePlayer.as_mut() else { return Ok(None); };
        if slotId < -999 || slotId >= crate::net::minecraft::inventory::ContainerPlayer::ContainerPlayer::SLOT_COUNT as i32 {
            return Err(CodecError::InvalidData(format!("creative player-container slot {slotId} outside -999 or 0..45")));
        }
        if !inventoryTab && slotId >= 0 && !(36..=44).contains(&slotId) {
            return Err(CodecError::InvalidData(format!(
                "ContainerCreative player slot {slotId} is outside hotbar 36..44"
            )));
        }

        let beforeCursor = player.inventory.getItemStack().clone();
        let beforeSlots = player.inventoryContainer.slots().to_vec();
        let originalSlotStack = if slotId >= 0 {
            beforeSlots.get(slotId as usize).cloned().unwrap_or(ItemStack::EMPTY)
        } else {
            ItemStack::EMPTY
        };
        let quickCraftFinished = clickType == ClickType::QuickCraft
            && Container::getDragEvent(mouseButton) == 2;
        let applied = if !inventoryTab && clickType == ClickType::QuickMove {
            if slotId < 0 {
                false
            } else {
                player.inventoryContainer.putStackInSlot(slotId, ItemStack::EMPTY)?;
                player.inventory.syncFromContainerPlayer(&player.inventoryContainer);
                true
            }
        } else if !inventoryTab && clickType == ClickType::PickupAll {
            if slotId < 0 {
                false
            } else {
                creative_hotbar_pickup_all(player, mouseButton == 1)
            }
        } else {
            apply_player_container_click(
                player,
                gameType,
                slotId,
                mouseButton,
                clickType,
            )?.is_some()
        };
        if !applied { return Ok(None); }
        let afterCursor = player.inventory.getItemStack().clone();
        let afterSlots = player.inventoryContainer.slots().to_vec();
        state.revision = state.revision.wrapping_add(1);
        Ok(Some(CreativePlayerContainerClick {
            beforeCursor,
            afterCursor,
            beforeSlots,
            afterSlots,
            originalSlotStack,
            quickCraftFinished,
        }))
    }

    /// Client-side `Container.slotClick` prediction for the active non-player
    /// container, followed by protocol-340 `CPacketClickWindow` using its real
    /// window ID. The server remains authoritative through SetSlot/WindowItems.
    pub fn clickOpenContainerSlot(
        &self,
        slotId: i32,
        mouseButton: i32,
        clickType: ClickType,
    ) -> Result<Option<RawPacket>, crate::net::minecraft::network::PacketBuffer::CodecError> {
        use crate::net::minecraft::network::PacketBuffer::CodecError;

        let validButton = match clickType {
            ClickType::Pickup | ClickType::QuickMove | ClickType::Throw | ClickType::PickupAll => {
                matches!(mouseButton, 0 | 1)
            }
            ClickType::Swap => (0..9).contains(&mouseButton),
            ClickType::Clone => mouseButton == 2,
            ClickType::QuickCraft => {
                Container::getDragEvent(mouseButton) <= 2
                    && Container::extractDragMode(mouseButton) <= 2
            }
        };
        if !validButton { return Ok(None); }

        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        let gameType = state.gameType;
        let Some(player) = state.thePlayer.as_mut() else { return Ok(None); };
        if clickType == ClickType::Clone && gameType != GameType::Creative { return Ok(None); }
        let crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP {
            inventory,
            openContainer,
            ..
        } = player;
        let Some(container) = openContainer.as_mut() else { return Ok(None); };
        if slotId < -999 || slotId >= container.slotCount() as i32 {
            return Err(CodecError::InvalidData(format!(
                "open-container slot {slotId} outside -999 or 0..{}",
                container.slotCount().saturating_sub(1)
            )));
        }
        let windowId = container.windowId();
        let actionNumber = container.getNextTransactionID();
        if clickType != ClickType::QuickCraft {
            container.resetQuickCraft();
        }

        let clickedItem = match clickType {
            ClickType::Pickup => {
                if slotId == -999 {
                    let original = ItemStack::EMPTY;
                    let mut cursor = inventory.getItemStack().clone();
                    if !cursor.isEmpty() {
                        if mouseButton == 0 { cursor = ItemStack::EMPTY; } else { cursor.shrink(1); }
                        inventory.setItemStack(cursor);
                    }
                    original
                } else {
                    let index = usize::try_from(slotId)
                        .map_err(|_| CodecError::InvalidData(format!("negative open-container slot {slotId}")))?;
                    let original = container.getSlot(index).cloned().ok_or_else(|| {
                        CodecError::InvalidData(format!("open-container slot {slotId} is absent"))
                    })?;
                    let mut slotStack = original.clone();
                    let mut cursor = inventory.getItemStack().clone();
                    let slotLimit = cursor.getMaxStackSize().max(1);
                    let cursorValidForSlot = container.isItemValidForSlot(slotId, &cursor);

                    if slotStack.isEmpty() {
                        if !cursor.isEmpty() && cursorValidForSlot {
                            let requested = if mouseButton == 0 { cursor.getCount() } else { 1 };
                            let moved = requested.min(slotLimit).min(cursor.getMaxStackSize());
                            slotStack = cursor.splitStack(moved);
                        }
                    } else if cursor.isEmpty() {
                        let removed = if mouseButton == 0 {
                            slotStack.getCount()
                        } else {
                            (slotStack.getCount() + 1) / 2
                        };
                        cursor = slotStack.splitStack(removed);
                    } else if slotStack.canStackWith(&cursor) && cursorValidForSlot {
                        let requested = if mouseButton == 0 { cursor.getCount() } else { 1 };
                        let capacity = slotStack.getMaxStackSize().saturating_sub(slotStack.getCount());
                        let moved = requested.min(capacity).max(0);
                        if moved > 0 {
                            cursor.shrink(moved);
                            slotStack.grow(moved);
                        }
                    } else if cursor.getCount() <= slotLimit && cursorValidForSlot {
                        std::mem::swap(&mut slotStack, &mut cursor);
                    }
                    container.putStackInSlot(slotId, slotStack)?;
                    inventory.setItemStack(cursor);
                    original
                }
            }
            ClickType::QuickMove => {
                if slotId < 0 { return Ok(None); }
                let index = slotId as usize;
                let mut result = ItemStack::EMPTY;
                loop {
                    let moved = container.transferStackInSlot(index);
                    if moved.isEmpty() { break; }
                    result = moved.clone();
                    if !container.getSlot(index).is_some_and(|remaining| ItemStack::areItemsEqual(remaining, &moved)) {
                        break;
                    }
                }
                result
            }
            ClickType::Swap => {
                if slotId < 0 { return Ok(None); }
                container.swapWithHotbar(slotId as usize, mouseButton as usize);
                ItemStack::EMPTY
            }
            ClickType::Throw => {
                if slotId >= 0 && inventory.getItemStack().isEmpty() {
                    container.throwFromSlot(slotId as usize, mouseButton == 1);
                }
                ItemStack::EMPTY
            }
            ClickType::PickupAll => {
                if slotId < 0 { return Ok(None); }
                let mut cursor = inventory.getItemStack().clone();
                if container.pickupAll(&mut cursor, mouseButton == 1) {
                    inventory.setItemStack(cursor);
                }
                ItemStack::EMPTY
            }
            ClickType::Clone => {
                if slotId >= 0 && inventory.getItemStack().isEmpty() {
                    if let Some(stack) = container.getSlot(slotId as usize) {
                        if !stack.isEmpty() {
                            let mut clone = stack.clone();
                            clone.setCount(clone.getMaxStackSize());
                            inventory.setItemStack(clone);
                        }
                    }
                }
                ItemStack::EMPTY
            }
            ClickType::QuickCraft => {
                let mut cursor = inventory.getItemStack().clone();
                if container.quickCraft(
                    slotId,
                    mouseButton,
                    &mut cursor,
                    gameType == GameType::Creative,
                ) {
                    inventory.setItemStack(cursor);
                }
                ItemStack::EMPTY
            }
        };
        container.syncToPlayerInventory(inventory);
        state.revision = state.revision.wrapping_add(1);
        CPacketClickWindow::new(
            windowId,
            slotId,
            mouseButton,
            clickType,
            &clickedItem,
            actionNumber,
        ).writePacketData().map(Some)
    }

    pub fn activeItemState(&self) -> Option<(bool, EnumHand, ItemStack, i32)> {
        let state = self.inner.read().unwrap_or_else(|poison| poison.into_inner());
        let player = state.thePlayer.as_ref()?;
        Some((player.isHandActive(), player.getActiveHand(), player.getActiveItemStack().clone(), player.getItemInUseCount()))
    }

    pub fn heldItemState(&self) -> Option<(ItemStack, ItemStack, f32)> {
        let state = self
            .inner
            .read()
            .unwrap_or_else(|poison| poison.into_inner());
        let player = state.thePlayer.as_ref()?;
        let main = player.inventory.getCurrentItem().clone();
        let off = player.inventory.offHandInventory
            .first()
            .cloned()
            .unwrap_or(ItemStack::EMPTY);
        Some((main, off, player.getCooledAttackStrength(1.0)))
    }

    pub fn localPlayerIsRowingBoat(&self) -> bool {
        self.inner
            .read()
            .unwrap_or_else(|poison| poison.into_inner())
            .thePlayer
            .as_ref()
            .is_some_and(EntityPlayerSP::isRowingBoat)
    }

    /// Runs the local `EntityPlayerSP` tick against the latest `WorldClient`
    /// and returns the exact ordered serverbound packets selected by MCP's
    /// `onUpdateWalkingPlayer` state machine.
    pub fn tickLocalPlayer(&self, keys: MovementKeyState) -> Vec<RawPacket> {
        let mut state = self
            .inner
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut packets = Vec::new();
        let mut ticked = false;

        {
            let PlayClientState {
                worldClient,
                thePlayer,
                playerPosition,
                gameType,
                doneLoadingTerrain,
                ..
            } = &mut *state;
            let emitServerboundMovement = *doneLoadingTerrain;

            if let Some(world) = worldClient.as_mut() {
                let xpTarget = thePlayer.as_ref().map(|player| [
                    player.entity.posX,
                    player.entity.posY + player.getEyeHeight() as f64 * 0.5,
                    player.entity.posZ,
                ]);
                let localPlayerId = thePlayer.as_ref().map(|player| player.entityId);
                let localPlayerState = thePlayer.as_ref().map(|player| (
                    [player.entity.posX, player.entity.posY, player.entity.posZ],
                    player.entity.height,
                ));
                world.tickEntitiesWithPlayerContext(xpTarget, localPlayerId, localPlayerState);

                // EntityBoat#onUpdate emits CPacketSteerBoat after controlBoat,
                // before EntityPlayerSP's riding packets in the passenger tick.
                if let Some(player) = thePlayer.as_ref() {
                    if let Some(vehicleId) = player.entity.ridingEntityId {
                        if world.localPlayerControlsVehicle(vehicleId, player.entityId) {
                            if let Some(vehicle) = world.getNonPlayerEntityByID(vehicleId) {
                                if matches!(
                                    &vehicle.kind,
                                    ClientEntityKind::Object { objectType: ObjectSpawnType::Boat, .. }
                                ) {
                                    if emitServerboundMovement {
                                        packets.push(CPacketSteerBoat::new(
                                            vehicle.boatPaddleState(0),
                                            vehicle.boatPaddleState(1),
                                        ).writePacketData());
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(player) = thePlayer.as_mut() {
                    // EntityPlayerSP belongs to the network state rather than
                    // WorldClient, but MCP still ticks it with the ordinary
                    // entity list before any ITickable tile entity. Its
                    // CPacketPlayer therefore describes the pre-shulker-sweep
                    // position; TileEntityShulkerBox moves both client and
                    // server players afterward during the same world tick.
                    let generatedPackets = player.onUpdate(world, keys, *gameType);
                    if emitServerboundMovement {
                        packets.extend(generatedPackets);
                    }
                    world.updateAttachedFireworksForLocalPlayer(player);
                    world.tickTileEntitiesAfterPlayers(None, Some(&mut player.entity));
                    *playerPosition = player_position_state(player);
                } else {
                    world.tickTileEntitiesAfterPlayers(None, None);
                }
                ticked = true;
            }
        }

        if ticked {
            state.cloudTickCounter = state.cloudTickCounter.wrapping_add(1);
            let cloudTick = state.cloudTickCounter;
            if cloudTick % 20 == 0 {
                state.damagedBlocks.retain(|_, progress| {
                    cloudTick.wrapping_sub(progress.getCreationCloudUpdateTick()) <= 400
                });
            }
            state.revision = state.revision.wrapping_add(1);
        }
        packets
    }

    /// Applies raw mouse deltas after the caller has performed the original
    /// sensitivity-cube and invert-mouse calculation.
    pub fn turnLocalPlayer(&self, yaw: f32, pitch: f32) -> bool {
        let mut state = self
            .inner
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        let PlayClientState {
            thePlayer,
            playerPosition,
            revision,
            ..
        } = &mut *state;
        let Some(player) = thePlayer.as_mut() else {
            return false;
        };
        player.turn(yaw, pitch);
        *playerPosition = player_position_state(player);
        *revision = (*revision).wrapping_add(1);
        true
    }

    pub fn takeTitlePackets(&self) -> Vec<SPacketTitle> {
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        std::mem::take(&mut state.pendingTitlePackets)
    }

    pub fn takeBossInfoPackets(&self) -> Vec<SPacketUpdateBossInfo> {
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        std::mem::take(&mut state.pendingBossInfoPackets)
    }

    pub fn takeTabCompleteMatches(&self) -> Vec<Vec<String>> {
        let mut state = self.inner.write().unwrap_or_else(|poison| poison.into_inner());
        std::mem::take(&mut state.tabCompleteMatches)
    }

    fn update(&self, operation: impl FnOnce(&mut PlayClientState)) {
        let mut state = self
            .inner
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        operation(&mut state);
        state.revision = state.revision.wrapping_add(1);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayHandlerEvent {
    None,
    JoinGame(SPacketJoinGame),
    Respawn { dimension: i32, dimensionChanged: bool },
    TerrainReady,
    PlayerDied { message: ITextComponent },
    Sound {
        sound: ResourceLocation,
        category: SoundCategory,
        x: f64,
        y: f64,
        z: f64,
        volume: f32,
        pitch: f32,
    },
    WorldEffect {
        effectType: i32,
        position: BlockPos,
        data: i32,
        serverWide: bool,
    },
    MapUpdated { mapId: i32, revision: u64 },
    ChunkLoaded { chunkX: i32, chunkZ: i32, loadedChunks: usize },
    ChunkUnloaded { chunkX: i32, chunkZ: i32, loadedChunks: usize },
    BlockChanged,
    TimeUpdated { totalWorldTime: i64, worldTime: i64 },
    SignEditorOpened { position: BlockPos },
    TileEntityUpdated { position: BlockPos, action: u8, applied: bool },
    /// `SPacketChangeGameState(4, 1)`: the end-credits `GuiWinGame` opens and
    /// sends `CPacketClientStatus(PERFORM_RESPAWN)` when it finishes.
    WinGame,
    /// `SPacketChangeGameState(4, 0)`: credits already seen, respawn directly.
    AutoRespawn,
    Disconnected(ITextComponent),
    IgnoredPacket(i32),
}

fn valid_player_container_click_button(mouseButton: i32, clickType: ClickType) -> bool {
    match clickType {
        ClickType::Pickup | ClickType::QuickMove | ClickType::Throw | ClickType::PickupAll => {
            matches!(mouseButton, 0 | 1)
        }
        ClickType::Swap => (0..9).contains(&mouseButton),
        ClickType::Clone => mouseButton == 2,
        ClickType::QuickCraft => {
            Container::getDragEvent(mouseButton) <= 2
                && Container::extractDragMode(mouseButton) <= 2
        }
    }
}

/// Shared local implementation of MCP `ContainerPlayer#slotClick` used by
/// both ordinary inventory windows and `GuiContainerCreative`. Transaction
/// IDs and outbound packet selection deliberately remain with the callers.
fn apply_player_container_click(
    player: &mut EntityPlayerSP,
    gameType: GameType,
    slotId: i32,
    mouseButton: i32,
    clickType: ClickType,
) -> Result<Option<ItemStack>, crate::net::minecraft::network::PacketBuffer::CodecError> {
    use crate::net::minecraft::network::PacketBuffer::CodecError;

    if clickType != ClickType::QuickCraft {
        player.inventoryContainer.resetQuickCraft();
    }
    let clickedItem = match clickType {
        ClickType::Pickup => {
            if slotId == -999 {
                let original = ItemStack::EMPTY;
                let mut cursor = player.inventory.getItemStack().clone();
                if !cursor.isEmpty() {
                    if mouseButton == 0 { cursor = ItemStack::EMPTY; } else { cursor.shrink(1); }
                    player.inventory.setItemStack(cursor);
                }
                original
            } else {
                let index = usize::try_from(slotId).map_err(|_| {
                    CodecError::InvalidData(format!("negative player-container slot {slotId}"))
                })?;
                let original = player.inventoryContainer.getSlot(index).cloned().ok_or_else(|| {
                    CodecError::InvalidData(format!("player-container slot {slotId} outside 0..45"))
                })?;
                let mut slotStack = original.clone();
                let mut cursor = player.inventory.getItemStack().clone();
                let validPlacement = player_container_slot_accepts(slotId, &cursor);
                let slotLimit = player_container_slot_limit(slotId, &cursor);

                if slotStack.isEmpty() {
                    if !cursor.isEmpty() && validPlacement {
                        let requested = if mouseButton == 0 { cursor.getCount() } else { 1 };
                        let moved = requested.min(slotLimit).min(cursor.getMaxStackSize());
                        slotStack = cursor.splitStack(moved);
                    }
                } else if cursor.isEmpty() {
                    let removed = if mouseButton == 0 {
                        slotStack.getCount()
                    } else {
                        (slotStack.getCount() + 1) / 2
                    };
                    cursor = slotStack.splitStack(removed);
                } else if validPlacement && slotStack.canStackWith(&cursor) {
                    let requested = if mouseButton == 0 { cursor.getCount() } else { 1 };
                    let capacity = slotLimit
                        .min(cursor.getMaxStackSize())
                        .saturating_sub(slotStack.getCount());
                    let moved = requested.min(capacity).max(0);
                    if moved > 0 {
                        cursor.shrink(moved);
                        slotStack.grow(moved);
                    }
                } else if validPlacement && cursor.getCount() <= slotLimit {
                    std::mem::swap(&mut slotStack, &mut cursor);
                } else if !validPlacement
                    && slotStack.canStackWith(&cursor)
                    && cursor.getMaxStackSize() > 1
                    && slotStack.getCount() + cursor.getCount() <= cursor.getMaxStackSize()
                {
                    let moved = slotStack.getCount();
                    cursor.grow(moved);
                    slotStack = ItemStack::EMPTY;
                }

                player.inventoryContainer.putStackInSlot(slotId, slotStack)?;
                player.inventory.setItemStack(cursor);
                player.inventory.syncFromContainerPlayer(&player.inventoryContainer);
                original
            }
        }
        ClickType::QuickMove => {
            if slotId < 0 { return Ok(None); }
            let index = slotId as usize;
            let mut result = ItemStack::EMPTY;
            loop {
                let moved = player.inventoryContainer.transferStackInSlot(index);
                if moved.isEmpty() {
                    break;
                }
                result = moved.clone();
                let sameItemRemains = player.inventoryContainer
                    .getSlot(index)
                    .is_some_and(|remaining| ItemStack::areItemsEqual(remaining, &moved));
                if !sameItemRemains {
                    break;
                }
            }
            if !result.isEmpty() {
                player.inventory.syncFromContainerPlayer(&player.inventoryContainer);
            }
            result
        }
        ClickType::Swap => {
            if slotId < 0 { return Ok(None); }
            if player.inventoryContainer.swapWithHotbar(slotId as usize, mouseButton as usize) {
                player.inventory.syncFromContainerPlayer(&player.inventoryContainer);
            }
            ItemStack::EMPTY
        }
        ClickType::Throw => {
            if slotId >= 0 && player.inventory.getItemStack().isEmpty() {
                if player.inventoryContainer.throwFromSlot(slotId as usize, mouseButton == 1) {
                    player.inventory.syncFromContainerPlayer(&player.inventoryContainer);
                }
            }
            ItemStack::EMPTY
        }
        ClickType::PickupAll => {
            if slotId < 0 { return Ok(None); }
            let mut cursor = player.inventory.getItemStack().clone();
            if player.inventoryContainer.pickupAll(&mut cursor, mouseButton == 1) {
                player.inventory.setItemStack(cursor);
                player.inventory.syncFromContainerPlayer(&player.inventoryContainer);
            }
            ItemStack::EMPTY
        }
        ClickType::Clone => {
            if gameType != GameType::Creative { return Ok(None); }
            if slotId >= 0 && player.inventory.getItemStack().isEmpty() {
                if let Some(stack) = player.inventoryContainer.getSlot(slotId as usize) {
                    if !stack.isEmpty() {
                        let mut clone = stack.clone();
                        clone.setCount(clone.getMaxStackSize());
                        player.inventory.setItemStack(clone);
                    }
                }
            }
            ItemStack::EMPTY
        }
        ClickType::QuickCraft => {
            let mut cursor = player.inventory.getItemStack().clone();
            if player.inventoryContainer.quickCraft(
                slotId,
                mouseButton,
                &mut cursor,
                gameType == GameType::Creative,
            ) {
                player.inventory.setItemStack(cursor);
                player.inventory.syncFromContainerPlayer(&player.inventoryContainer);
            }
            ItemStack::EMPTY
        }
    };
    Ok(Some(clickedItem))
}

/// `ContainerCreative#canMergeSlot` restricts PICKUP_ALL to the nine real
/// hotbar slots. Using ContainerPlayer#pickupAll here would incorrectly pull
/// matching stacks from the main inventory while a non-INVENTORY creative tab
/// is active.
fn creative_hotbar_pickup_all(player: &mut EntityPlayerSP, reverse: bool) -> bool {
    let mut cursor = player.inventory.getItemStack().clone();
    if cursor.isEmpty() || cursor.getCount() >= cursor.getMaxStackSize() { return false; }
    let indices: Vec<usize> = if reverse {
        (36..=44).rev().collect()
    } else {
        (36..=44).collect()
    };
    let mut changed = false;
    for pass in 0..2 {
        for &index in &indices {
            if cursor.getCount() >= cursor.getMaxStackSize() { break; }
            let stack = player.inventoryContainer.getSlot(index).cloned().unwrap_or(ItemStack::EMPTY);
            if stack.isEmpty() || !stack.canStackWith(&cursor) { continue; }
            if pass == 0 && stack.getCount() == stack.getMaxStackSize() { continue; }
            let moved = (cursor.getMaxStackSize() - cursor.getCount()).min(stack.getCount());
            if moved <= 0 { continue; }
            let mut remaining = stack;
            remaining.shrink(moved);
            cursor.grow(moved);
            if player.inventoryContainer.putStackInSlot(index as i32, remaining).is_err() {
                return changed;
            }
            changed = true;
        }
    }
    if changed {
        player.inventory.setItemStack(cursor);
        player.inventory.syncFromContainerPlayer(&player.inventoryContainer);
    }
    changed
}

fn player_container_slot_limit(slotId: i32, stack: &ItemStack) -> i32 {
    crate::net::minecraft::inventory::ContainerPlayer::playerContainerSlotLimit(slotId, stack)
}

fn player_container_slot_accepts(slotId: i32, stack: &ItemStack) -> bool {
    crate::net::minecraft::inventory::ContainerPlayer::playerContainerSlotAccepts(slotId, stack)
}

fn try_start_using_held_item(
    player: &mut EntityPlayerSP,
    gameType: GameType,
    hand: EnumHand,
) -> bool {
    let stack = player.getHeldItem(hand).clone();
    if stack.isEmpty()
        || stack.getItemUseAction()
            == crate::net::minecraft::item::EnumAction::EnumAction::None
    {
        return false;
    }
    // MCP `EntityPlayer#canEat(alwaysEdible)`: `(alwaysEdible || needFood())
    // && !capabilities.disableDamage`.
    if stack.isFood()
        && (player.capabilities.disableDamage
            || (player.getFoodStats().getFoodLevel() >= 20 && !stack.isAlwaysEdible()))
    {
        return false;
    }
    // ItemBow.findAmmo: offhand, main hand, then the complete main inventory.
    if stack.itemId == 261 && gameType != GameType::Creative {
        let hasArrow = player
            .inventory
            .offHandInventory
            .iter()
            .chain(std::iter::once(player.inventory.getCurrentItem()))
            .chain(player.inventory.mainInventory.iter())
            .any(|candidate| {
                matches!(candidate.itemId, 262 | 439 | 440) && !candidate.isEmpty()
            });
        if !hasArrow {
            return false;
        }
    }
    player.setActiveHand(hand)
}

#[derive(Debug, thiserror::Error)]
pub enum NetHandlerPlayClientError {
    #[error(transparent)]
    Network(#[from] NetworkManagerError),
    #[error("invalid play packet {packetId:#x}: {message}")]
    Packet { packetId: i32, message: String },
}

#[derive(Debug, Clone)]
pub struct NetHandlerPlayClient {
    profile: GameProfile,
    settings: ClientSettingsSnapshot,
    doneLoadingTerrain: bool,
    currentServerMaxPlayers: u8,
    playerEntityId: i32,
    playerPosition: PlayerPositionState,
    sharedState: SharedPlayClientState,
    playerInfoMap: HashMap<Uuid, NetworkPlayerInfo>,
    particleRandomizer: JavaRandom,
}

impl NetHandlerPlayClient {
    pub fn new(
        profileIn: GameProfile,
        settings: ClientSettingsSnapshot,
        sharedState: SharedPlayClientState,
    ) -> Self {
        let authenticatedProfile = profileIn.clone();
        sharedState.update(|state| {
            state.localGameProfile = Some(authenticatedProfile);
        });
        Self {
            profile: profileIn,
            settings,
            doneLoadingTerrain: false,
            currentServerMaxPlayers: 20,
            playerEntityId: 0,
            playerPosition: PlayerPositionState::default(),
            sharedState,
            playerInfoMap: HashMap::new(),
            particleRandomizer: JavaRandom::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as i64,
            ),
        }
    }

    pub fn processPacket(
        &mut self,
        networkManager: &mut NetworkManager,
        packet: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        match packet.id {
            0x00 => self.handleSpawnObject(packet),
            0x01 => self.handleSpawnExperienceOrb(packet),
            0x02 => self.handleSpawnGlobalEntity(packet),
            0x03 => self.handleSpawnMob(packet),
            0x04 => self.handleSpawnPainting(packet),
            0x05 => self.handleSpawnPlayer(packet),
            0x06 => self.handleAnimation(packet),
            0x08 => self.handleBlockBreakAnim(packet),
            0x09 => self.handleUpdateTileEntity(packet),
            0x0A => self.handleBlockAction(packet),
            0x0B => self.handleBlockChange(packet),
            0x0C => self.handleUpdateBossInfo(packet),
            0x0D => self.handleServerDifficulty(packet),
            0x0E => self.handleTabComplete(packet),
            0x0F => self.handleChat(packet),
            0x10 => self.handleMultiBlockChange(packet),
            0x11 => self.handleConfirmTransaction(networkManager, packet),
            0x12 => self.handleCloseWindow(packet),
            0x13 => self.handleOpenWindow(packet),
            0x14 => self.handleWindowItems(packet),
            0x15 => self.handleWindowProperty(packet),
            0x16 => self.handleSetSlot(packet),
            0x17 => self.handleCooldown(packet),
            0x18 => self.handleCustomPayload(packet),
            0x19 => self.handleCustomSound(packet),
            0x1A => self.handleDisconnect(packet),
            0x1B => self.handleEntityStatus(packet),
            0x1D => self.processChunkUnload(packet),
            0x1E => self.handleChangeGameState(packet),
            0x1F => self.handleKeepAlive(networkManager, packet),
            0x20 => self.handleChunkData(packet),
            0x21 => self.handleEffect(packet),
            0x22 => self.handleParticles(packet),
            0x23 => self.handleJoinGame(networkManager, packet),
            0x24 => self.handleMaps(packet),
            0x25 | 0x26 | 0x27 | 0x28 => self.handleEntityMovement(packet),
            0x29 => self.handleMoveVehicle(networkManager, packet),
            0x2A => self.handleSignEditorOpen(packet),
            0x2B => self.handlePlaceGhostRecipe(packet),
            0x2C => self.handlePlayerAbilities(packet),
            0x2D => self.handleCombatEvent(packet),
            0x2E => self.handlePlayerListItem(packet),
            0x2F => self.handlePlayerPosLook(networkManager, packet),
            0x30 => self.handleUseBed(packet),
            0x31 => self.handleRecipeBook(packet),
            0x32 => self.handleDestroyEntities(packet),
            0x33 => self.handleRemoveEntityEffect(packet),
            0x35 => self.handleRespawn(packet),
            0x36 => self.handleEntityHeadLook(packet),
            0x3A => self.handleHeldItemChange(packet),
            0x3B => self.handleDisplayObjective(packet),
            0x3C => self.handleEntityMetadata(packet),
            0x3D => self.handleEntityAttach(packet),
            0x3E => self.handleEntityVelocity(packet),
            0x3F => self.handleEntityEquipment(packet),
            0x40 => self.handleSetExperience(packet),
            0x41 => self.handleUpdateHealth(packet),
            0x42 => self.handleScoreboardObjective(packet),
            0x43 => self.handleSetPassengers(packet),
            0x44 => self.handleTeams(packet),
            0x45 => self.handleUpdateScore(packet),
            0x46 => self.handleSpawnPosition(packet),
            0x47 => self.handleTimeUpdate(packet),
            0x48 => self.handleTitle(packet),
            0x49 => self.handleSoundEffect(packet),
            0x4A => self.handlePlayerListHeaderFooter(packet),
            0x4C => self.handleEntityTeleport(packet),
            0x4E => self.handleEntityProperties(packet),
            0x4F => self.handleEntityEffect(packet),
            packetId => Ok(PlayHandlerEvent::IgnoredPacket(packetId)),
        }
    }

    pub fn handleJoinGame(
        &mut self,
        networkManager: &mut NetworkManager,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketJoinGame::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.currentServerMaxPlayers = packet.getMaxPlayers();
        self.playerEntityId = packet.getPlayerId();
        let dimension = packet.getDimension();
        let maxPlayers = self.currentServerMaxPlayers;
        let playerEntityId = self.playerEntityId;
        let gameType = packet.getGameType();
        self.sharedState.update(|state| {
            state.worldClient = Some(WorldClient::new(dimension));
            let mut player = EntityPlayerSP::new(playerEntityId);
            gameType.configurePlayerCapabilities(&mut player.capabilities);
            state.thePlayer = Some(player);
            state.playerPosition = PlayerPositionState::default();
            state.currentServerMaxPlayers = maxPlayers;
            state.gameType = gameType;
            state.hardcoreMode = packet.isHardcoreMode();
            state.doneLoadingTerrain = false;
            state.cloudTickCounter = 0;
            state.damagedBlocks.clear();
            state.pendingSignEditorPosition = None;
            state.mapData.clear();
        });

        let clientSettings = CPacketClientSettings::new(
            &self.settings.language,
            self.settings.renderDistanceChunks,
            self.settings.chatVisibility,
            self.settings.chatColours,
            self.settings.modelPartFlags,
            self.settings.mainHand,
        );
        networkManager.sendPacket(
            &clientSettings
                .writePacketData()
                .map_err(NetworkManagerError::Codec)?,
        )?;

        let mut brandData = Vec::new();
        write_string(
            crate::net::minecraft::client::ClientBrandRetriever::getClientModName(),
            32767,
            &mut brandData,
        )
        .map_err(NetworkManagerError::Codec)?;
        networkManager.sendPacket(
            &CPacketCustomPayload::new("MC|Brand", brandData)
                .and_then(|packet| packet.writePacketData())
                .map_err(NetworkManagerError::Codec)?,
        )?;
        Ok(PlayHandlerEvent::JoinGame(packet))
    }

    /// MCP `NetHandlerPlayClient#handleRespawn` plus the Rust-equivalent
    /// `Minecraft#setDimensionAndSpawnPlayer` hand-off.
    pub fn handleRespawn(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketRespawn::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let dimension = packet.getDimensionID();
        let gameType = packet.getGameType();
        let playerEntityId = self.playerEntityId;
        let mut dimensionChanged = false;

        self.sharedState.update(|state| {
            dimensionChanged = state
                .worldClient
                .as_ref()
                .map_or(true, |world| world.getDimension() != dimension);

            if dimensionChanged {
                state.worldClient = Some(WorldClient::new(dimension));
                state.doneLoadingTerrain = false;
                state.cloudTickCounter = 0;
                state.damagedBlocks.clear();
                state.pendingSignEditorPosition = None;
                state.pendingGhostRecipe = None;
                state.mapData.clear();
            } else if let Some(world) = state.worldClient.as_mut() {
                world.removeAllEntities();
            }

            // `Minecraft#setDimensionAndSpawnPlayer` creates a fresh
            // EntityPlayerSP but copies the recipe book and data-manager
            // entries from the old instance before restoring the entity ID.
            let oldPlayer = state.thePlayer.take();
            let mut player = EntityPlayerSP::new(playerEntityId);
            if let Some(oldPlayer) = oldPlayer {
                player.recipeBook = oldPlayer.recipeBook;
                player.dataManager = oldPlayer.dataManager;
            }
            gameType.configurePlayerCapabilities(&mut player.capabilities);
            state.thePlayer = Some(player);
            state.playerPosition = PlayerPositionState::default();
            state.gameType = gameType;
        });

        self.playerPosition = PlayerPositionState::default();
        if dimensionChanged {
            self.doneLoadingTerrain = false;
        }

        Ok(PlayHandlerEvent::Respawn {
            dimension,
            dimensionChanged,
        })
    }

    pub fn handleChunkData(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketChunkData::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let mut result = Ok(0_usize);
        self.sharedState.update(|state| {
            result = match state.worldClient.as_mut() {
                Some(world) => world
                    .applyChunkData(&packet)
                    .map(|_| world.loadedChunkCount()),
                None => Err(
                    crate::net::minecraft::network::PacketBuffer::CodecError::InvalidData(
                        "chunk data received before Join Game".to_owned(),
                    ),
                ),
            };
        });
        let loadedChunks = result.map_err(|error| packet_error(rawPacket.id, error))?;
        Ok(PlayHandlerEvent::ChunkLoaded {
            chunkX: packet.getChunkX(),
            chunkZ: packet.getChunkZ(),
            loadedChunks,
        })
    }

    pub fn processChunkUnload(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketUnloadChunk::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let mut loadedChunks = None;
        self.sharedState.update(|state| {
            if let Some(world) = state.worldClient.as_mut() {
                world.doPreChunk(packet.getX(), packet.getZ(), false);
                loadedChunks = Some(world.loadedChunkCount());
            }
        });
        let loadedChunks = loadedChunks
            .ok_or_else(|| packet_error(rawPacket.id, "unload chunk received before Join Game"))?;
        Ok(PlayHandlerEvent::ChunkUnloaded {
            chunkX: packet.getX(),
            chunkZ: packet.getZ(),
            loadedChunks,
        })
    }

    /// MCP `NetHandlerPlayClient#handleBlockBreakAnim` delegates to
    /// `WorldClient#sendBlockBreakProgress`, whose client renderer stores one
    /// progress record per breaker entity.
    pub fn handleBlockBreakAnim(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketBlockBreakAnim::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            if (0..10).contains(&packet.getProgress()) {
                let currentTick = state.cloudTickCounter;
                let replace = state.damagedBlocks
                    .get(&packet.getBreakerId())
                    .map_or(true, |progress| progress.getPosition() != packet.getPosition());
                if replace {
                    state.damagedBlocks.insert(
                        packet.getBreakerId(),
                        DestroyBlockProgress::new(packet.getBreakerId(), packet.getPosition()),
                    );
                }
                if let Some(progress) = state.damagedBlocks.get_mut(&packet.getBreakerId()) {
                    progress.setPartialBlockDamage(packet.getProgress());
                    progress.setCloudUpdateTick(currentTick);
                }
            } else {
                state.damagedBlocks.remove(&packet.getBreakerId());
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    /// MCP `NetHandlerPlayClient#handleUpdateTileEntity`. Action 4 is
    /// `TileEntitySkull`; other concrete TileEntity classes remain decoded but
    /// are not silently represented as skulls.
    pub fn handleUpdateTileEntity(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketUpdateTileEntity::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let mut applied = false;
        self.sharedState.update(|state| {
            if let Some(world) = state.worldClient.as_mut() {
                applied = world.handleUpdateTileEntity(&packet);
            }
        });
        Ok(PlayHandlerEvent::TileEntityUpdated {
            position: packet.getPos(),
            action: packet.getTileEntityType(),
            applied,
        })
    }

    pub fn handleBlockAction(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketBlockAction::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            if let Some(world) = state.worldClient.as_mut() {
                world.handleBlockAction(&packet);
            }
        });
        Ok(PlayHandlerEvent::BlockChanged)
    }

    pub fn handleBlockChange(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketBlockChange::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let mut result = Ok(());
        self.sharedState.update(|state| {
            result = state
                .worldClient
                .as_mut()
                .ok_or_else(|| "block change received before Join Game".to_owned())
                .and_then(|world| world.handleBlockChange(&packet));
        });
        result.map_err(|error| packet_error(rawPacket.id, error))?;
        Ok(PlayHandlerEvent::BlockChanged)
    }

    pub fn handleMultiBlockChange(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketMultiBlockChange::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let mut result = Ok(());
        self.sharedState.update(|state| {
            result = state
                .worldClient
                .as_mut()
                .ok_or_else(|| "multi block change received before Join Game".to_owned())
                .and_then(|world| world.handleMultiBlockChange(&packet));
        });
        result.map_err(|error| packet_error(rawPacket.id, error))?;
        Ok(PlayHandlerEvent::BlockChanged)
    }

    pub fn handleKeepAlive(
        &mut self,
        networkManager: &mut NetworkManager,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketKeepAlive::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        networkManager.sendPacket(&CPacketKeepAlive::new(packet.getId()).writePacketData())?;
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleSpawnPosition(&mut self, rawPacket:&RawPacket) -> Result<PlayHandlerEvent,NetHandlerPlayClientError> {
        let packet=SPacketSpawnPosition::readPacketData(rawPacket).map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.withWrite(|state| { if let Some(world)=state.worldClient.as_mut(){ world.setSpawnPoint(packet.getSpawnPos()); } });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleTimeUpdate(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketTimeUpdate::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            if let Some(world) = state.worldClient.as_mut() {
                world.setTotalWorldTime(packet.getTotalWorldTime());
                world.setWorldTime(packet.getWorldTime());
            }
        });
        Ok(PlayHandlerEvent::TimeUpdated {
            totalWorldTime: packet.getTotalWorldTime(),
            worldTime: packet.getWorldTime(),
        })
    }

    pub fn handleSignEditorOpen(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketSignEditorOpen::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let position = packet.getSignPosition();
        self.sharedState.update(|state| {
            state.pendingSignEditorPosition = Some(position);
        });
        Ok(PlayHandlerEvent::SignEditorOpened { position })
    }

    pub fn handlePlayerPosLook(
        &mut self,
        networkManager: &mut NetworkManager,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketPlayerPosLook::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let flags = packet.getFlags();
        let current = self
            .sharedState
            .withRead(|state| state.playerPosition);

        let posX = if flags.contains(EnumFlags::X) {
            current.posX + packet.getX()
        } else {
            packet.getX()
        };
        let posY = if flags.contains(EnumFlags::Y) {
            current.posY + packet.getY()
        } else {
            packet.getY()
        };
        let posZ = if flags.contains(EnumFlags::Z) {
            current.posZ + packet.getZ()
        } else {
            packet.getZ()
        };
        let rotationYaw = if flags.contains(EnumFlags::Y_ROT) {
            current.rotationYaw + packet.getYaw()
        } else {
            packet.getYaw()
        };
        let rotationPitch = if flags.contains(EnumFlags::X_ROT) {
            current.rotationPitch + packet.getPitch()
        } else {
            packet.getPitch()
        };

        let first = !self.doneLoadingTerrain;
        let playerEntityId = self.playerEntityId;
        let mut appliedPosition = PlayerPositionState::default();
        self.sharedState.update(|state| {
            if state.thePlayer.is_none() {
                let mut player = EntityPlayerSP::new(playerEntityId);
                state.gameType.configurePlayerCapabilities(&mut player.capabilities);
                state.thePlayer = Some(player);
            }
            let player = state.thePlayer.as_mut().expect("local player was initialized");

            if !flags.contains(EnumFlags::X) {
                player.entity.motionX = 0.0;
            }
            if !flags.contains(EnumFlags::Y) {
                player.entity.motionY = 0.0;
            }
            if !flags.contains(EnumFlags::Z) {
                player.entity.motionZ = 0.0;
            }

            player.setPositionAndRotation(posX, posY, posZ, rotationYaw, rotationPitch);
            if first {
                player.setPreviousPositionToCurrent();
            }
            appliedPosition = player_position_state(player);
            state.playerPosition = appliedPosition;
            state.doneLoadingTerrain = true;
        });

        self.playerPosition = appliedPosition;
        self.doneLoadingTerrain = true;
        networkManager.sendPacket(
            &CPacketConfirmTeleport::new(packet.getTeleportId()).writePacketData(),
        )?;
        networkManager.sendPacket(
            &PositionRotation::new(
                appliedPosition.posX,
                appliedPosition.posY,
                appliedPosition.posZ,
                appliedPosition.rotationYaw,
                appliedPosition.rotationPitch,
                false,
            )
            .writePacketData(),
        )?;

        Ok(if first {
            PlayHandlerEvent::TerrainReady
        } else {
            PlayHandlerEvent::None
        })
    }

    pub fn handleUpdateBossInfo(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketUpdateBossInfo::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| state.pendingBossInfoPackets.push(packet));
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleTitle(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketTitle::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| state.pendingTitlePackets.push(packet));
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleTabComplete(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketTabComplete::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            state.tabCompleteMatches.push(packet.getMatches().to_vec());
            if state.tabCompleteMatches.len() > 8 {
                let excess = state.tabCompleteMatches.len() - 8;
                state.tabCompleteMatches.drain(0..excess);
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleChat(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketChat::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let component = packet.getChatComponent().clone();
        let chatType = packet.func_192590_c();
        self.sharedState.update(|state| {
            let playerTicks = state.thePlayer.as_ref().map_or(0, |player| player.entity.ticksExisted);
            if chatType == ChatType::GameInfo {
                state.actionBarMessage = Some(component);
                state.actionBarUpdatedTick = playerTicks;
            } else {
                state.nextChatSerial = state.nextChatSerial.wrapping_add(1);
                let serial = state.nextChatSerial;
                state.chatMessages.push(ReceivedChatMessage { serial, component, chatType });
                if state.chatMessages.len() > 100 {
                    let excess = state.chatMessages.len() - 100;
                    state.chatMessages.drain(0..excess);
                }
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleScoreboardObjective(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketScoreboardObjective::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| match packet.getAction() {
            0 => state.scoreboard.addScoreObjective(packet.getObjectiveName(), packet.getObjectiveValue(), packet.getRenderType()),
            1 => state.scoreboard.removeObjective(packet.getObjectiveName()),
            2 => state.scoreboard.updateScoreObjective(packet.getObjectiveName(), packet.getObjectiveValue(), packet.getRenderType()),
            _ => {}
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleDisplayObjective(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketDisplayObjective::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| state.scoreboard.setObjectiveInDisplaySlot(packet.getPosition(), packet.getName()));
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleUpdateScore(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketUpdateScore::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| match packet.getScoreAction() {
            UpdateScoreAction::Change => state.scoreboard.setScore(packet.getPlayerName(), packet.getObjectiveName(), packet.getScoreValue()),
            UpdateScoreAction::Remove => state.scoreboard.removeScore(
                packet.getPlayerName(),
                (!packet.getObjectiveName().is_empty()).then_some(packet.getObjectiveName()),
            ),
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleTeams(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketTeams::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            match packet.getAction() {
                0 => {
                    {
                        let team = state.scoreboard.createTeam(packet.getName());
                        team.update(
                            packet.getDisplayName(), packet.getPrefix(), packet.getSuffix(),
                            packet.getFriendlyFlags(), packet.getNameTagVisibility(),
                            packet.getCollisionRule(), packet.getColor(),
                        );
                    }
                    for player in packet.getPlayers() { state.scoreboard.addPlayerToTeam(player.clone(), packet.getName()); }
                }
                1 => state.scoreboard.removeTeam(packet.getName()),
                2 => {
                    if let Some(team) = state.scoreboard.getTeamMut(packet.getName()) {
                        team.update(
                            packet.getDisplayName(), packet.getPrefix(), packet.getSuffix(),
                            packet.getFriendlyFlags(), packet.getNameTagVisibility(),
                            packet.getCollisionRule(), packet.getColor(),
                        );
                    }
                }
                3 => for player in packet.getPlayers() { state.scoreboard.addPlayerToTeam(player.clone(), packet.getName()); },
                4 => for player in packet.getPlayers() { state.scoreboard.removePlayerFromTeam(player, packet.getName()); },
                _ => {}
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handlePlayerListItem(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketPlayerListItem::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        match packet.getAction() {
            PlayerListAction::AddPlayer => for entry in packet.getEntries() {
                if let Some(id) = entry.profile.getId() {
                    self.playerInfoMap.insert(id, NetworkPlayerInfo::new(entry.profile.clone(), entry.gameMode, entry.ping, entry.displayName.clone()));
                }
            },
            PlayerListAction::UpdateGameMode => for entry in packet.getEntries() {
                if let Some(info) = entry.profile.getId().and_then(|id| self.playerInfoMap.get_mut(&id)) { info.setGameType(entry.gameMode); }
            },
            PlayerListAction::UpdateLatency => for entry in packet.getEntries() {
                if let Some(info) = entry.profile.getId().and_then(|id| self.playerInfoMap.get_mut(&id)) { info.setResponseTime(entry.ping); }
            },
            PlayerListAction::UpdateDisplayName => for entry in packet.getEntries() {
                if let Some(info) = entry.profile.getId().and_then(|id| self.playerInfoMap.get_mut(&id)) { info.setDisplayName(entry.displayName.clone()); }
            },
            PlayerListAction::RemovePlayer => for entry in packet.getEntries() {
                if let Some(id) = entry.profile.getId() { self.playerInfoMap.remove(&id); }
            },
        }
        let snapshot = self.playerInfoMap.clone();
        self.sharedState.update(|state| {
            if let Some(localId) = state
                .localGameProfile
                .as_ref()
                .and_then(GameProfile::getId)
            {
                if let Some(playerInfo) = snapshot.get(&localId) {
                    // `EntityPlayerSP` is also an AbstractClientPlayer. Keep the
                    // resolved object after a later REMOVE_PLAYER packet.
                    state.localPlayerInfo = Some(playerInfo.clone());
                }
            }
            if let Some(world) = state.worldClient.as_mut() {
                for (&uniqueId, playerInfo) in &snapshot {
                    let _ = world.cachePlayerInfo(uniqueId, playerInfo.clone());
                }
            }
            state.playerInfoMap = snapshot;
        });
        Ok(PlayHandlerEvent::None)
    }


    pub fn handlePlayerListHeaderFooter(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketPlayerListHeaderFooter::readPacketData(rawPacket)
            .map_err(|error| NetHandlerPlayClientError::Packet { packetId: rawPacket.id, message: error.to_string() })?;
        let header = packet.getHeader().clone();
        let footer = packet.getFooter().clone();
        self.sharedState.update(|state| {
            state.playerListHeader = (!header.getUnformattedText().is_empty()).then_some(header);
            state.playerListFooter = (!footer.getUnformattedText().is_empty()).then_some(footer);
        });
        Ok(PlayHandlerEvent::None)
    }

    /// MCP `NetHandlerPlayClient#handleParticles`: protocol count zero uses
    /// offset*speed directly; positive counts use six sequential samples from
    /// Java Random#nextGaussian for position spread and velocity.
    pub fn handleParticles(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketParticles::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let particleType = packet.getParticleType();
        let visibility = packet.isLongDistance();
        let parameters = packet.getParticleArgs();
        let origin = [
            packet.getXCoordinate(),
            packet.getYCoordinate(),
            packet.getZCoordinate(),
        ];
        let mut requests = Vec::new();
        if packet.getParticleCount() == 0 {
            requests.push(
                ParticleSpawnRequest::new(
                    particleType,
                    origin,
                    [
                        (packet.getParticleSpeed() * packet.getXOffset()) as f64,
                        (packet.getParticleSpeed() * packet.getYOffset()) as f64,
                        (packet.getParticleSpeed() * packet.getZOffset()) as f64,
                    ],
                    parameters,
                )
                .withVisibility(visibility, false),
            );
        } else {
            for _ in 0..packet.getParticleCount() {
                let offset = [
                    self.particleRandomizer.next_gaussian() * packet.getXOffset() as f64,
                    self.particleRandomizer.next_gaussian() * packet.getYOffset() as f64,
                    self.particleRandomizer.next_gaussian() * packet.getZOffset() as f64,
                ];
                let speed = [
                    self.particleRandomizer.next_gaussian() * packet.getParticleSpeed() as f64,
                    self.particleRandomizer.next_gaussian() * packet.getParticleSpeed() as f64,
                    self.particleRandomizer.next_gaussian() * packet.getParticleSpeed() as f64,
                ];
                requests.push(
                    ParticleSpawnRequest::new(
                        particleType,
                        [origin[0] + offset[0], origin[1] + offset[1], origin[2] + offset[2]],
                        speed,
                        parameters,
                    )
                    .withVisibility(visibility, false),
                );
            }
        }
        self.sharedState.update(|state| {
            if let Some(world) = state.worldClient.as_mut() {
                world.queueParticleSpawns(requests);
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    /// MCP `NetHandlerPlayClient#handleMaps`: update or create the exact
    /// `map_<id>` client MapData instance, then refresh its pixel/decorations
    /// from the server patch.
    pub fn handleMaps(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketMaps::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let mapId = packet.getMapId();
        let mut revision = 0;
        self.sharedState.update(|state| {
            let mapData = state
                .mapData
                .entry(mapId)
                .or_insert_with(|| MapData::new(mapId));
            packet.setMapdataTo(mapData);
            revision = mapData.revision;
        });
        Ok(PlayHandlerEvent::MapUpdated { mapId, revision })
    }

    pub fn handleSpawnObject(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketSpawnObject::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let objectType = ObjectSpawnType::fromPacketType(packet.getType());
        if matches!(objectType, ObjectSpawnType::Unknown(_)) {
            // Vanilla leaves `entity` null and skips unknown object types.
            return Ok(PlayHandlerEvent::None);
        }
        if objectType == ObjectSpawnType::FishHook {
            let anglerId = packet.getData();
            let anglerExists = self.sharedState.withRead(|state| {
                state.thePlayer.as_ref().is_some_and(|player| player.entityId == anglerId)
                    || state.worldClient.as_ref().is_some_and(|world| world.getEntityByID(anglerId).is_some())
            });
            // MCP creates no EntityFishHook when packet data does not resolve
            // to an EntityPlayer.
            if !anglerExists { return Ok(PlayHandlerEvent::None); }
        }
        let yaw = packet.getYaw() as f32 * 360.0 / 256.0;
        let pitch = packet.getPitch() as f32 * 360.0 / 256.0;
        let spawnVelocity = [
            packet.getSpeedX() as f64 / 8000.0,
            packet.getSpeedY() as f64 / 8000.0,
            packet.getSpeedZ() as f64 / 8000.0,
        ];
        let kind = ClientEntityKind::Object {
            objectType,
            data: packet.getData(),
            spawnVelocity,
        };
        let mut entity = EntityOtherClient::new(
            packet.getEntityID(),
            Some(packet.getUniqueId()),
            kind,
            packet.getX(),
            packet.getY(),
            packet.getZ(),
            yaw,
            pitch,
        );
        // MCP resets packet data to zero after consuming class-specific data
        // for item frames, leash knots, shulker bullets and all four fireball
        // constructors. Facing/anchor values and constructor acceleration must
        // therefore never fall through to the generic velocity assignment.
        let remainingData = if matches!(
            objectType,
            ObjectSpawnType::ItemFrame
                | ObjectSpawnType::LeashKnot
                | ObjectSpawnType::ShulkerBullet
                | ObjectSpawnType::FishHook
        ) || objectType.isFireball()
        {
            0
        } else {
            packet.getData()
        };
        if remainingData > 0 {
            entity.setVelocity(spawnVelocity[0], spawnVelocity[1], spawnVelocity[2]);
        }
        self.sharedState.update(|state| {
            if let Some(world) = state.worldClient.as_mut() {
                world.addNonPlayerEntityToWorld(packet.getEntityID(), entity);
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleSpawnExperienceOrb(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketSpawnExperienceOrb::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let entity = EntityOtherClient::new(
            packet.getEntityID(),
            None,
            ClientEntityKind::ExperienceOrb { xpValue: packet.getXPValue() },
            packet.getX(),
            packet.getY(),
            packet.getZ(),
            0.0,
            0.0,
        );
        self.sharedState.update(|state| {
            if let Some(world) = state.worldClient.as_mut() {
                world.addNonPlayerEntityToWorld(packet.getEntityID(), entity);
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    /// MCP `handleSpawnGlobalEntity`: vanilla protocol 340 only defines
    /// discriminator 1, which enters World.weatherEffects as lightning.
    pub fn handleSpawnGlobalEntity(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketSpawnGlobalEntity::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        if packet.getType() != 1 {
            return Ok(PlayHandlerEvent::None);
        }
        let effect = EntityLightningBolt::new(
            packet.getEntityId(),
            packet.getX(),
            packet.getY(),
            packet.getZ(),
            false,
        );
        self.sharedState.update(|state| {
            if let Some(world) = state.worldClient.as_mut() {
                world.addWeatherEffect(effect);
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleSpawnMob(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketSpawnMob::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let Some(entityType) = MobEntityType::fromId(packet.getEntityType()) else {
            // Mirrors EntityList.createEntityByID returning null for an unknown registry ID.
            return Ok(PlayHandlerEvent::None);
        };
        let yaw = packet.getYaw() as f32 * 360.0 / 256.0;
        let pitch = packet.getPitch() as f32 * 360.0 / 256.0;
        let headYaw = packet.getHeadPitch() as f32 * 360.0 / 256.0;
        let mut entity = EntityOtherClient::new(
            packet.getEntityID(),
            Some(packet.getUniqueId()),
            ClientEntityKind::Mob { entityType },
            packet.getX(),
            packet.getY(),
            packet.getZ(),
            yaw,
            pitch,
        );
        entity.renderYawOffset = headYaw;
        entity.rotationYawHead = headYaw;
        entity.setVelocity(
            packet.getVelocityX() as f64 / 8000.0,
            packet.getVelocityY() as f64 / 8000.0,
            packet.getVelocityZ() as f64 / 8000.0,
        );
        entity.applyMetadata(packet.getDataManagerEntries().iter().cloned());
        self.sharedState.update(|state| {
            if let Some(world) = state.worldClient.as_mut() {
                world.addNonPlayerEntityToWorld(packet.getEntityID(), entity);
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleSpawnPainting(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketSpawnPainting::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let position = packet.getPosition();
        let entity = EntityOtherClient::new(
            packet.getEntityID(),
            Some(packet.getUniqueId()),
            ClientEntityKind::Painting {
                title: packet.getTitle().to_owned(),
                hangingPosition: position,
                facing: packet.getFacing(),
            },
            position.x as f64,
            position.y as f64,
            position.z as f64,
            0.0,
            0.0,
        );
        self.sharedState.update(|state| {
            if let Some(world) = state.worldClient.as_mut() {
                world.addNonPlayerEntityToWorld(packet.getEntityID(), entity);
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleOpenWindow(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketOpenWindow::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let mut result = Ok(());
        self.sharedState.update(|state| {
            let guiId = packet.getGuiId();
            let horseSpec = if guiId == "EntityHorse" {
                state
                    .worldClient
                    .as_ref()
                    .and_then(|world| world.getNonPlayerEntityByID(packet.getEntityId()))
                    .and_then(|entity| {
                        let registryName = match &entity.kind {
                            ClientEntityKind::Mob { entityType } => entityType.registryName,
                            _ => return None,
                        };
                        let kind = HorseInventoryKind::fromRegistryName(registryName)?;
                        let chested = matches!(kind, HorseInventoryKind::Donkey | HorseInventoryKind::Mule | HorseInventoryKind::Llama)
                            && entity.horseChested();
                        let chestColumns = if kind == HorseInventoryKind::Llama {
                            entity.llamaStrength()
                        } else if chested {
                            5
                        } else {
                            0
                        };
                        Some(HorseInventorySpec {
                            entityId: packet.getEntityId(),
                            kind,
                            chested,
                            chestColumns,
                        })
                    })
            } else {
                None
            };
            let Some(player) = state.thePlayer.as_mut() else {
                result = Err(crate::net::minecraft::network::PacketBuffer::CodecError::InvalidData(
                    "open window received before Join Game".to_owned(),
                ));
                return;
            };
            let windowId = packet.getWindowId() as i32;
            let title = packet.getWindowTitle().clone();
            let slotCount = packet.getSlotCount() as usize;

            // MCP 1.12.2 `NetHandlerPlayClient#handleOpenWindow` delegates
            // zero-slot `IInteractionObject` IDs to `EntityPlayerSP#displayGui`
            // and slotted inventories to `displayGUIChest`. Preserve those
            // concrete branches rather than substituting a chest layout.
            let opened = match guiId {
                "minecraft:crafting_table" => ContainerWorkbench::new(
                    windowId,
                    title,
                    slotCount,
                    &player.inventory,
                )
                .map(OpenContainer::Workbench),
                "minecraft:furnace" => ContainerFurnace::new(
                    windowId,
                    title,
                    slotCount,
                    &player.inventory,
                )
                .map(OpenContainer::Furnace),
                "minecraft:anvil" => ContainerRepair::new(
                    windowId,
                    title,
                    slotCount,
                    &player.inventory,
                )
                .map(OpenContainer::Repair),
                "minecraft:enchanting_table" => ContainerEnchantment::new(
                    windowId,
                    title,
                    slotCount,
                    &player.inventory,
                )
                .map(OpenContainer::Enchantment),
                "minecraft:shulker_box" => ContainerShulkerBox::new(
                    windowId,
                    title,
                    slotCount,
                    &player.inventory,
                )
                .map(OpenContainer::ShulkerBox),
                "minecraft:hopper" => ContainerHopper::new(
                    windowId,
                    title,
                    slotCount,
                    &player.inventory,
                )
                .map(OpenContainer::Hopper),
                "minecraft:brewing_stand" => ContainerBrewingStand::new(
                    windowId,
                    title,
                    slotCount,
                    &player.inventory,
                )
                .map(OpenContainer::BrewingStand),
                "minecraft:dispenser" => ContainerDispenser::new(
                    windowId,
                    title,
                    slotCount,
                    &player.inventory,
                )
                .map(OpenContainer::Dispenser),
                "minecraft:dropper" => ContainerDispenser::newDropper(
                    windowId,
                    title,
                    slotCount,
                    &player.inventory,
                )
                .map(OpenContainer::Dropper),
                "minecraft:beacon" => ContainerBeacon::new(
                    windowId,
                    title,
                    slotCount,
                    &player.inventory,
                )
                .map(OpenContainer::Beacon),
                "minecraft:villager" => ContainerMerchant::new(
                    windowId,
                    title,
                    slotCount,
                    &player.inventory,
                )
                .map(OpenContainer::Merchant),
                "EntityHorse" => match horseSpec {
                    Some(spec) => ContainerHorseInventory::new(
                        windowId,
                        title,
                        slotCount,
                        &player.inventory,
                        spec,
                    )
                    .map(OpenContainer::Horse),
                    None => Err(crate::net::minecraft::network::PacketBuffer::CodecError::InvalidData(
                        format!("EntityHorse window references missing or non-horse entity {}", packet.getEntityId()),
                    )),
                },
                _ => {
                    let vanillaChestLike = guiId == "minecraft:container"
                        || guiId == "minecraft:chest"
                        || packet.hasSlots();
                    if vanillaChestLike {
                        ContainerChest::new(
                            windowId,
                            guiId,
                            title,
                            slotCount,
                            &player.inventory,
                        )
                        .map(OpenContainer::Chest)
                    } else {
                        log::warn!(
                            "SPacketOpenWindow gui={} window={} requires an unmigrated concrete GUI",
                            guiId,
                            packet.getWindowId(),
                        );
                        player.openContainer = None;
                        return;
                    }
                }
            };

            match opened {
                Ok(container) => {
                    player.openContainer = Some(container);
                    player.inventory.setItemStack(ItemStack::EMPTY);
                }
                Err(error) => result = Err(error),
            }
        });
        result.map_err(|error| packet_error(rawPacket.id, error))?;
        Ok(PlayHandlerEvent::None)
    }


    /// MCP `NetHandlerPlayClient#func_191980_a`.
    pub fn handleRecipeBook(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketRecipeBook::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            let Some(player) = state.thePlayer.as_mut() else { return; };
            player.recipeBook.setGuiOpen(packet.isGuiOpen());
            player.recipeBook.setFilteringCraftable(packet.isFilteringCraftable());
            match packet.getState() {
                RecipeBookState::Remove => {
                    for &recipeId in packet.getRecipes() { player.recipeBook.lockById(recipeId); }
                }
                RecipeBookState::Init => {
                    for &recipeId in packet.getRecipes() { player.recipeBook.unlockById(recipeId); }
                    for &recipeId in packet.getDisplayedRecipes() { player.recipeBook.markNewById(recipeId); }
                }
                RecipeBookState::Add => {
                    for &recipeId in packet.getRecipes() {
                        player.recipeBook.unlockById(recipeId);
                        player.recipeBook.markNewById(recipeId);
                    }
                }
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    /// MCP `NetHandlerPlayClient#func_194307_a`. The actual ghost slots are
    /// built by the active `GuiRecipeBook`, never by the network handler.
    pub fn handlePlaceGhostRecipe(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketPlaceGhostRecipe::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            let Some(player) = state.thePlayer.as_ref() else { return; };
            let activeWindowId = player.openContainer.as_ref().map_or(0, OpenContainer::windowId);
            if activeWindowId == packet.getWindowId() as i32 {
                state.pendingGhostRecipe = Some((packet.getWindowId(), packet.getRecipeId()));
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    /// MCP `NetHandlerPlayClient#handleCustomPayload` branch for `MC|TrList`.
    /// The server owns all completed trade state; this only installs the
    /// recipe list sent for the currently open merchant window.
    pub fn handleCustomPayload(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketCustomPayload::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        if packet.getChannelName() != "MC|TrList" {
            return Ok(PlayHandlerEvent::None);
        }
        let mut input = packet.getBufferData();
        let windowId = crate::net::minecraft::network::PacketBuffer::read_i32_be(&mut input)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let recipes = MerchantRecipeList::readFromBuf(&mut input)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        if !input.is_empty() {
            return Err(packet_error(rawPacket.id, crate::net::minecraft::network::PacketBuffer::CodecError::InvalidData(
                format!("{} unread MC|TrList bytes", input.len()),
            )));
        }
        self.sharedState.update(|state| {
            let Some(player) = state.thePlayer.as_mut() else { return; };
            let Some(container) = player.openContainer.as_mut() else { return; };
            if container.windowId() == windowId {
                container.setMerchantRecipes(recipes);
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleWindowProperty(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketWindowProperty::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let mut result = Ok(());
        self.sharedState.update(|state| {
            let Some(player) = state.thePlayer.as_mut() else {
                result = Err(crate::net::minecraft::network::PacketBuffer::CodecError::InvalidData(
                    "window property received before Join Game".to_owned(),
                ));
                return;
            };
            let Some(container) = player.openContainer.as_mut() else {
                return;
            };
            if container.windowId() == packet.getWindowId() as i32 {
                result = container.updateProgressBar(
                    packet.getProperty() as i32,
                    packet.getValue() as i32,
                );
            }
        });
        result.map_err(|error| packet_error(rawPacket.id, error))?;
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleCloseWindow(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketCloseWindow::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            let Some(player) = state.thePlayer.as_mut() else { return; };
            // MCP `handleCloseWindow` delegates to
            // `EntityPlayerSP.closeScreenAndDropStack()` and does not gate the
            // close on the packet window id. `None` is this Rust port's
            // equivalent of restoring `openContainer = inventoryContainer`.
            let _windowId = packet.getWindowId();
            player.inventory.setItemStack(ItemStack::EMPTY);
            player.openContainer = None;
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleConfirmTransaction(
        &mut self,
        networkManager: &mut NetworkManager,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketConfirmTransaction::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let matchesActiveWindow = self.sharedState.withRead(|state| {
            packet.getWindowId() == 0
                || state.thePlayer.as_ref().is_some_and(|player| {
                    player.openContainer.as_ref().is_some_and(|container| {
                        container.windowId() == packet.getWindowId()
                    })
                })
        });
        if matchesActiveWindow && !packet.wasAccepted() {
            networkManager.sendPacket(
                &CPacketConfirmTransaction::new(
                    packet.getWindowId(),
                    packet.getActionNumber(),
                    true,
                ).writePacketData(),
            )?;
        }
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleWindowItems(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketWindowItems::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let mut result = Ok(());
        self.sharedState.update(|state| {
            let Some(player) = state.thePlayer.as_mut() else {
                result = Err(crate::net::minecraft::network::PacketBuffer::CodecError::InvalidData(
                    "window items received before Join Game".to_owned(),
                ));
                return;
            };
            if packet.getWindowId() == 0 {
                result = player.inventoryContainer.setAll(packet.getItemStacks());
                if result.is_ok() {
                    player.inventory.syncFromContainerPlayer(&player.inventoryContainer);
                    if let Some(container) = player.openContainer.as_mut() {
                        container.syncFromPlayerInventory(&player.inventory);
                    }
                }
            } else {
                let crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP {
                    inventory,
                    openContainer,
                    ..
                } = player;
                if let Some(container) = openContainer.as_mut().filter(|container| {
                    container.windowId() == packet.getWindowId() as i32
                }) {
                    result = container.setAll(packet.getItemStacks());
                    if result.is_ok() {
                        container.syncToPlayerInventory(inventory);
                    }
                }
            }
        });
        result.map_err(|error| packet_error(rawPacket.id, error))?;
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleSetSlot(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketSetSlot::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let mut result = Ok(());
        self.sharedState.update(|state| {
            let Some(player) = state.thePlayer.as_mut() else {
                result = Err(crate::net::minecraft::network::PacketBuffer::CodecError::InvalidData(
                    "set slot received before Join Game".to_owned(),
                ));
                return;
            };
            let stack = packet.getStack().clone();
            match packet.getWindowId() {
                -1 => player.inventory.setItemStack(stack),
                -2 => {
                    let crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP {
                        inventory,
                        openContainer,
                        ..
                    } = player;
                    result = inventory.setInventorySlotContents(packet.getSlot() as i32, stack);
                    if result.is_ok() {
                        if let Some(container) = openContainer.as_mut() {
                            container.syncFromPlayerInventory(inventory);
                        }
                    }
                }
                0 => {
                    let slot = packet.getSlot() as i32;
                    let crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP {
                        inventory,
                        inventoryContainer,
                        openContainer,
                        ..
                    } = player;
                    result = inventoryContainer.putStackInSlot(slot, stack.clone());
                    if result.is_ok() {
                        result = inventory.applyContainerPlayerSlot(slot, stack);
                    }
                    if result.is_ok() {
                        if let Some(container) = openContainer.as_mut() {
                            container.syncFromPlayerInventory(inventory);
                        }
                    }
                    if result.is_ok() && (slot == 45 || (36..=44).contains(&slot)) {
                        log::debug!("SPacketSetSlot window=0 slot={slot} applied to hand-capable inventory state");
                    }
                }
                windowId => {
                    let crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP {
                        inventory,
                        openContainer,
                        ..
                    } = player;
                    if let Some(container) = openContainer.as_mut().filter(|container| {
                        container.windowId() == windowId as i32
                    }) {
                        result = container.putStackInSlot(packet.getSlot() as i32, stack);
                        if result.is_ok() {
                            container.syncToPlayerInventory(inventory);
                        }
                    }
                }
            }
        });
        result.map_err(|error| packet_error(rawPacket.id, error))?;
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleHeldItemChange(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketHeldItemChange::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let mut result = Ok(());
        self.sharedState.update(|state| {
            let Some(player) = state.thePlayer.as_mut() else {
                result = Err(crate::net::minecraft::network::PacketBuffer::CodecError::InvalidData(
                    "held item change received before Join Game".to_owned(),
                ));
                return;
            };
            result = player.inventory.setCurrentItem(packet.getHeldItemHotbarIndex());
        });
        result.map_err(|error| packet_error(rawPacket.id, error))?;
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleEntityEffect(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketEntityEffect::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        if packet.getEntityId() == self.playerEntityId {
            let effect = PotionEffect::new(
                packet.getEffectId(),
                packet.getDuration(),
                packet.getAmplifier(),
                packet.getIsAmbient(),
                packet.doesShowParticles(),
            );
            self.sharedState.update(|state| {
                if let Some(player) = state.thePlayer.as_mut() {
                    player.addPotionEffect(effect);
                }
            });
        }
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleRemoveEntityEffect(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketRemoveEntityEffect::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        if packet.getEntityId() == self.playerEntityId {
            self.sharedState.update(|state| {
                if let Some(player) = state.thePlayer.as_mut() {
                    player.removeActivePotionEffect(packet.getPotionId());
                }
            });
        }
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleEntityStatus(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketEntityStatus::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let localPlayerId = self.playerEntityId;
        let opcode = packet.getOpCode();
        let entityId = packet.getEntityId();
        let activationRandom = (opcode == 35 && entityId == localPlayerId).then(|| (
            self.particleRandomizer.next_f32() * 2.0 - 1.0,
            self.particleRandomizer.next_f32() * 2.0 - 1.0,
        ));
        self.sharedState.update(|state| {
            if entityId == localPlayerId {
                if let Some(player) = state.thePlayer.as_mut() {
                    if let Some((randomX, randomY)) = activationRandom {
                        // NetHandlerPlayClient has a dedicated opcode-35 branch
                        // in 1.12.2: activation overlay, particles and exactly
                        // one totem sound, without Entity#handleStatusUpdate.
                        player.activateTotem(randomX, randomY);
                        player.queueSoundAtPlayer(
                            "item.totem.use",
                            SoundCategory::Players,
                            1.0,
                            1.0,
                        );
                    } else {
                        player.handleStatusUpdate(opcode);
                    }
                }
            } else if let Some(world) = state.worldClient.as_mut() {
                if opcode == 35 {
                    world.addParticleEmitter(entityId, EnumParticleTypes::Totem, 30);
                    world.queueSoundAtEntity(entityId, "item.totem.use", 1.0, 1.0);
                } else {
                    world.handleEntityStatus(entityId, opcode);
                }
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleEntityAttach(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketEntityAttach::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            if let Some(world) = state.worldClient.as_mut() {
                world.attachEntity(packet.getEntityId(), packet.getVehicleEntityId());
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleEntityEquipment(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketEntityEquipment::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let slot = packet.getEquipmentSlot();
        let stack = packet.getItemStack().clone();
        let mut result = Ok(());
        let localPlayerId = self.playerEntityId;
        self.sharedState.update(|state| {
            if packet.getEntityID() == localPlayerId {
                if let Some(player) = state.thePlayer.as_mut() {
                    let inventoryIndex = match slot {
                        EntityEquipmentSlot::Mainhand => player.inventory.currentItem,
                        EntityEquipmentSlot::Offhand => 40,
                        EntityEquipmentSlot::Feet => 36,
                        EntityEquipmentSlot::Legs => 37,
                        EntityEquipmentSlot::Chest => 38,
                        EntityEquipmentSlot::Head => 39,
                    };
                    result = player.inventory.setInventorySlotContents(inventoryIndex, stack.clone());
                    if result.is_ok() {
                        let containerSlot = match slot {
                            EntityEquipmentSlot::Mainhand => 36 + player.inventory.currentItem,
                            EntityEquipmentSlot::Offhand => 45,
                            EntityEquipmentSlot::Feet => 8,
                            EntityEquipmentSlot::Legs => 7,
                            EntityEquipmentSlot::Chest => 6,
                            EntityEquipmentSlot::Head => 5,
                        };
                        result = player.inventoryContainer.putStackInSlot(containerSlot, stack.clone());
                    }
                }
            } else if let Some(world) = state.worldClient.as_mut() {
                world.setEntityEquipment(packet.getEntityID(), slot, stack.clone());
            }
        });
        result.map_err(|error| packet_error(rawPacket.id, error))?;
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleChangeGameState(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketChangeGameState::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        match packet.getGameState() {
            1 => self.sharedState.update(|state| {
                if let Some(world) = state.worldClient.as_mut() {
                    world.setRainStrength(0.0);
                }
            }),
            2 => self.sharedState.update(|state| {
                if let Some(world) = state.worldClient.as_mut() {
                    world.setRainStrength(1.0);
                }
            }),
            3 => {
                let gameType = GameType::getByID((packet.getValue() + 0.5).floor() as i32);
                self.sharedState.update(|state| {
                    state.gameType = gameType;
                    if let Some(player) = state.thePlayer.as_mut() {
                        gameType.configurePlayerCapabilities(&mut player.capabilities);
                    }
                });
            }
            4 => {
                // MCP `NetHandlerPlayClient#handleChangeGameState` game state 4:
                // `EntityPlayerMP` announces the end-credits on leaving the End.
                // Value 0 (credits already seen) respawns immediately, value 1
                // opens `GuiWinGame`, whose Runnable sends PERFORM_RESPAWN.
                return Ok(if (packet.getValue() + 0.5).floor() as i32 == 0 {
                    PlayHandlerEvent::AutoRespawn
                } else {
                    PlayHandlerEvent::WinGame
                });
            }
            7 => self.sharedState.update(|state| {
                if let Some(world) = state.worldClient.as_mut() {
                    world.setRainStrength(packet.getValue());
                }
            }),
            8 => self.sharedState.update(|state| {
                if let Some(world) = state.worldClient.as_mut() {
                    world.setThunderStrength(packet.getValue());
                }
            }),
            _ => {}
        }
        Ok(PlayHandlerEvent::None)
    }

    /// MCP `NetHandlerPlayClient#handleServerDifficulty`: stores the packet
    /// value on the client World's WorldInfo-equivalent difficulty field.
    pub fn handleServerDifficulty(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketServerDifficulty::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            if let Some(world) = state.worldClient.as_mut() {
                world.setDifficulty(packet.getDifficulty());
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    /// MCP `NetHandlerPlayClient#handleCooldown`.
    pub fn handleCooldown(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketCooldown::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            if let Some(player) = state.thePlayer.as_mut() {
                if packet.getTicks() == 0 {
                    player.getCooldownTrackerMut().removeCooldown(packet.getItemId());
                } else {
                    player.getCooldownTrackerMut().setCooldown(packet.getItemId(), packet.getTicks());
                }
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleCustomSound(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketCustomSound::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        Ok(PlayHandlerEvent::Sound {
            sound: ResourceLocation::parse(packet.getSoundName()),
            category: packet.getCategory(),
            x: packet.getX(),
            y: packet.getY(),
            z: packet.getZ(),
            volume: packet.getVolume(),
            pitch: packet.getPitch(),
        })
    }

    pub fn handleSoundEffect(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketSoundEffect::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        Ok(PlayHandlerEvent::Sound {
            sound: packet.getSound().clone(),
            category: packet.getCategory(),
            x: packet.getX(),
            y: packet.getY(),
            z: packet.getZ(),
            volume: packet.getVolume(),
            pitch: packet.getPitch(),
        })
    }

    pub fn handleEffect(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketEffect::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        Ok(PlayHandlerEvent::WorldEffect {
            effectType: packet.getSoundType(),
            position: packet.getSoundPos(),
            data: packet.getSoundData(),
            serverWide: packet.isSoundServerwide(),
        })
    }

    pub fn handleCombatEvent(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketCombatEvent::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        if packet.getEventType() == CombatEvent::EntityDied
            && packet.getPlayerId() == self.playerEntityId
        {
            let message = packet.getDeathMessage().cloned()
                .unwrap_or_else(|| ITextComponent::fromPlainText(""));
            return Ok(PlayHandlerEvent::PlayerDied { message });
        }
        Ok(PlayHandlerEvent::None)
    }

    pub fn handlePlayerAbilities(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketPlayerAbilities::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            if let Some(player) = state.thePlayer.as_mut() {
                player.capabilities.disableDamage = packet.isInvulnerable();
                player.capabilities.isFlying = packet.isFlying();
                player.capabilities.allowFlying = packet.isAllowFlying();
                player.capabilities.isCreativeMode = packet.isCreativeMode();
                player.capabilities.setFlySpeed(packet.getFlySpeed());
                player.capabilities.setPlayerWalkSpeed(packet.getWalkSpeed());
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleSetExperience(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketSetExperience::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            if let Some(player) = state.thePlayer.as_mut() {
                player.setXPStats(
                    packet.getExperienceBar(),
                    packet.getTotalExperience(),
                    packet.getLevel()
                );
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleUpdateHealth(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketUpdateHealth::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            if let Some(player) = state.thePlayer.as_mut() {
                player.setPlayerSPHealth(packet.getHealth());
                player.foodStats.setFoodLevel(packet.getFoodLevel());
                player.foodStats.setFoodSaturationLevel(packet.getSaturationLevel());
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleSetPassengers(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketSetPassengers::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let localPlayerId = self.playerEntityId;
        self.sharedState.update(|state| {
            let newlyMounting = state.thePlayer.as_ref().is_some_and(|player| {
                player.entity.ridingEntityId != Some(packet.getEntityId())
                    && packet.getPassengerIds().contains(&localPlayerId)
            });
            let boatYaw = state.worldClient.as_ref().and_then(|world| {
                let vehicle = world.getNonPlayerEntityByID(packet.getEntityId())?;
                matches!(
                    &vehicle.kind,
                    ClientEntityKind::Object { objectType: ObjectSpawnType::Boat, .. }
                ).then_some(vehicle.entity.rotationYaw)
            });

            if let Some(world) = state.worldClient.as_mut() {
                world.setPassengers(packet.getEntityId(), packet.getPassengerIds());
            }
            if let Some(player) = state.thePlayer.as_mut() {
                if packet.getPassengerIds().contains(&localPlayerId) {
                    player.entity.ridingEntityId = Some(packet.getEntityId());
                    if newlyMounting {
                        if let Some(yaw) = boatYaw {
                            // EntityPlayerSP#startRiding boat orientation reset.
                            player.entity.prevRotationYaw = yaw;
                            player.entity.rotationYaw = yaw;
                            player.rotationYawHead = yaw;
                        }
                    }
                } else if player.entity.ridingEntityId == Some(packet.getEntityId()) {
                    player.entity.ridingEntityId = None;
                    player.rowingBoat = false;
                }
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleMoveVehicle(
        &mut self,
        networkManager: &mut NetworkManager,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketMoveVehicle::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let localPlayerId = self.playerEntityId;
        let reply = self.sharedState.withWrite(|state| {
            let directVehicleId = state.thePlayer.as_ref()?.entity.ridingEntityId?;
            let reply = {
                let world = state.worldClient.as_mut()?;
                let lowest = world.lowestRidingEntityId(localPlayerId, Some(directVehicleId));
                if lowest == localPlayerId
                    || !world.localPlayerControlsVehicle(lowest, localPlayerId)
                {
                    return None;
                }
                if !world.setEntityPositionAndRotation(
                    lowest,
                    packet.getX(),
                    packet.getY(),
                    packet.getZ(),
                    packet.getYaw(),
                    packet.getPitch(),
                ) {
                    return None;
                }
                let vehicle = world.getBaseEntityByID(lowest)?;
                CPacketVehicleMove::fromEntity(vehicle).writePacketData()
            };
            state.revision = state.revision.wrapping_add(1);
            Some(reply)
        });
        if let Some(reply) = reply {
            networkManager.sendPacket(&reply)?;
        }
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleSpawnPlayer(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketSpawnPlayer::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let profile = self.playerInfoMap.get(&packet.getUniqueId())
            .map(|info| info.getGameProfile().clone())
            .unwrap_or_else(|| GameProfile::new(Some(packet.getUniqueId()), ""));
        let mut player = EntityOtherPlayerMP::new(packet.getEntityID(), packet.getUniqueId(), profile);
        player.setPlayerInfo(self.playerInfoMap.get(&packet.getUniqueId()).cloned());
        let yaw = packet.getYaw() as f32 * 360.0 / 256.0;
        let pitch = packet.getPitch() as f32 * 360.0 / 256.0;
        player.entity.setPositionAndRotation(packet.getX(), packet.getY(), packet.getZ(), yaw, pitch);
        player.setServerPosition(packet.getX(), packet.getY(), packet.getZ());
        player.applyMetadata(packet.getDataManagerEntries().iter().cloned());
        self.sharedState.update(|state| if let Some(world) = state.worldClient.as_mut() { world.addEntityToWorld(packet.getEntityID(), player); });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleEntityMovement(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketEntity::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            let Some(world) = state.worldClient.as_mut() else { return; };
            if let Some(entity) = world.getEntityByIDMut(packet.getEntityId()) {
                entity.serverPosX += packet.getX() as i64;
                entity.serverPosY += packet.getY() as i64;
                entity.serverPosZ += packet.getZ() as i64;
                let x = entity.serverPosX as f64 / 4096.0;
                let y = entity.serverPosY as f64 / 4096.0;
                let z = entity.serverPosZ as f64 / 4096.0;
                let yaw = if packet.isRotating() {
                    packet.getYaw() as f32 * 360.0 / 256.0
                } else {
                    entity.entity.rotationYaw
                };
                let pitch = if packet.isRotating() {
                    packet.getPitch() as f32 * 360.0 / 256.0
                } else {
                    entity.entity.rotationPitch
                };
                entity.setPositionAndRotationDirect(x, y, z, yaw, pitch, 3, false);
                entity.entity.onGround = packet.getOnGround();
                return;
            }
            if let Some(entity) = world.getNonPlayerEntityByIDMut(packet.getEntityId()) {
                entity.serverPosX += packet.getX() as i64;
                entity.serverPosY += packet.getY() as i64;
                entity.serverPosZ += packet.getZ() as i64;
                let x = entity.serverPosX as f64 / 4096.0;
                let y = entity.serverPosY as f64 / 4096.0;
                let z = entity.serverPosZ as f64 / 4096.0;
                let yaw = if packet.isRotating() {
                    packet.getYaw() as f32 * 360.0 / 256.0
                } else {
                    entity.entity.rotationYaw
                };
                let pitch = if packet.isRotating() {
                    packet.getPitch() as f32 * 360.0 / 256.0
                } else {
                    entity.entity.rotationPitch
                };
                entity.setPositionAndRotationDirect(x, y, z, yaw, pitch, 3, false);
                entity.entity.onGround = packet.getOnGround();
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleEntityTeleport(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketEntityTeleport::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            let Some(world) = state.worldClient.as_mut() else { return; };
            let yaw = packet.getYaw() as f32 * 360.0 / 256.0;
            let pitch = packet.getPitch() as f32 * 360.0 / 256.0;
            if let Some(entity) = world.getEntityByIDMut(packet.getEntityId()) {
                entity.setServerPosition(packet.getX(), packet.getY(), packet.getZ());
                let close = (entity.entity.posX - packet.getX()).abs() < 0.03125
                    && (entity.entity.posY - packet.getY()).abs() < 0.015625
                    && (entity.entity.posZ - packet.getZ()).abs() < 0.03125;
                entity.setPositionAndRotationDirect(
                    if close { entity.entity.posX } else { packet.getX() },
                    if close { entity.entity.posY } else { packet.getY() },
                    if close { entity.entity.posZ } else { packet.getZ() },
                    yaw,
                    pitch,
                    if close { 0 } else { 3 },
                    true,
                );
                entity.entity.onGround = packet.getOnGround();
                return;
            }
            if let Some(entity) = world.getNonPlayerEntityByIDMut(packet.getEntityId()) {
                entity.setServerPosition(packet.getX(), packet.getY(), packet.getZ());
                entity.setPositionAndRotationDirect(
                    packet.getX(),
                    packet.getY(),
                    packet.getZ(),
                    yaw,
                    pitch,
                    3,
                    true,
                );
                entity.entity.onGround = packet.getOnGround();
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleEntityHeadLook(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketEntityHeadLook::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            if let Some(world) = state.worldClient.as_mut() {
                world.setEntityRotationYawHead(
                    packet.getEntityId(),
                    packet.getYaw() as f32 * 360.0 / 256.0,
                );
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleEntityProperties(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketEntityProperties::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            let entityId = packet.getEntityId();
            if let Some(player) = state.thePlayer.as_mut().filter(|player| player.entityId == entityId) {
                for snapshot in packet.getSnapshots() {
                    player.attributeMap.setSnapshot(
                        snapshot.getName(),
                        snapshot.getBaseValue(),
                        snapshot.getModifiers(),
                    );
                }
                return;
            }
            let Some(world) = state.worldClient.as_mut() else { return; };
            if let Some(player) = world.getEntityByIDMut(entityId) {
                for snapshot in packet.getSnapshots() {
                    player.attributeMap.setSnapshot(
                        snapshot.getName(),
                        snapshot.getBaseValue(),
                        snapshot.getModifiers(),
                    );
                }
                return;
            }
            if let Some(entity) = world.getNonPlayerEntityByIDMut(entityId) {
                if entity.isLivingBase() {
                    for snapshot in packet.getSnapshots() {
                        entity.attributeMap.setSnapshot(
                            snapshot.getName(),
                            snapshot.getBaseValue(),
                            snapshot.getModifiers(),
                        );
                    }
                }
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleEntityVelocity(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketEntityVelocity::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let localPlayerId = self.playerEntityId;
        self.sharedState.update(|state| {
            if packet.getEntityID() == localPlayerId {
                if let Some(player) = state.thePlayer.as_mut() {
                    player.entity.setVelocity(
                        packet.getMotionX() as f64 / 8000.0,
                        packet.getMotionY() as f64 / 8000.0,
                        packet.getMotionZ() as f64 / 8000.0,
                    );
                }
            } else if let Some(world) = state.worldClient.as_mut() {
                world.setEntityVelocity(
                    packet.getEntityID(),
                    packet.getMotionX() as f64 / 8000.0,
                    packet.getMotionY() as f64 / 8000.0,
                    packet.getMotionZ() as f64 / 8000.0,
                );
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleEntityMetadata(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketEntityMetadata::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        self.sharedState.update(|state| {
            let entries = packet.getDataManagerEntries();
            if let Some(player) = state.thePlayer.as_mut() {
                if player.entityId == packet.getEntityId() {
                    player.applyMetadata(entries.iter().cloned());
                }
            }
            if let Some(world) = state.worldClient.as_mut() {
                world.applyEntityMetadata(packet.getEntityId(), entries.iter().cloned());
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleAnimation(&mut self, rawPacket:&RawPacket)->Result<PlayHandlerEvent,NetHandlerPlayClientError>{
        let packet=SPacketAnimation::readPacketData(rawPacket).map_err(|error|packet_error(rawPacket.id,error))?;
        let animation = packet.getAnimationType();
        let hand = match animation {
            0 => Some(crate::net::minecraft::util::EnumHand::EnumHand::MainHand),
            3 => Some(crate::net::minecraft::util::EnumHand::EnumHand::OffHand),
            _ => None,
        };
        self.sharedState.update(|state| {
            if animation == 2 {
                let playerId = packet.getEntityID();
                if state.thePlayer.as_ref().is_some_and(|player| player.entityId == playerId) {
                    let safeExit = state.thePlayer.as_ref()
                        .and_then(|player| player.bedLocation)
                        .and_then(|bed| state.worldClient.as_ref()
                            .and_then(|world| BlockBed::getSafeExitLocation(world, bed, 0)));
                    if let Some(player) = state.thePlayer.as_mut() {
                        player.wakeUpPlayerClient(safeExit, false);
                        state.playerPosition = player_position_state(player);
                    }
                } else if let Some(world) = state.worldClient.as_mut() {
                    let bed = world.getEntityByID(playerId).and_then(|player| player.bedLocation);
                    let safeExit = bed.and_then(|bed| BlockBed::getSafeExitLocation(world, bed, 0));
                    if let Some(player) = world.getEntityByIDMut(playerId) {
                        player.wakeUpPlayerClient(safeExit, false);
                    }
                }
            } else if let Some(hand) = hand {
                if let Some(world) = state.worldClient.as_mut() {
                    if let Some(entity) = world.getEntityByIDMut(packet.getEntityID()) {
                        entity.swingArm(hand);
                    } else if let Some(entity) = world.getNonPlayerEntityByIDMut(packet.getEntityID()) {
                        entity.swingArm(hand);
                    }
                }
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleUseBed(&mut self, rawPacket: &RawPacket) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketUseBed::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        let playerId = packet.getPlayerId();
        let bedPos = packet.getBedPosition();
        self.sharedState.update(|state| {
            let bedState = state.worldClient
                .as_ref()
                .map(|world| world.getBlockState(bedPos));
            let Some(bedState) = bedState else { return; };
            if state.thePlayer.as_ref().is_some_and(|player| player.entityId == playerId) {
                if let Some(player) = state.thePlayer.as_mut() {
                    player.trySleepClient(bedState, bedPos);
                    state.playerPosition = player_position_state(player);
                }
            } else if let Some(world) = state.worldClient.as_mut() {
                if let Some(player) = world.getEntityByIDMut(playerId) {
                    player.trySleepClient(bedState, bedPos);
                }
            }
        });
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleDestroyEntities(&mut self, rawPacket:&RawPacket)->Result<PlayHandlerEvent,NetHandlerPlayClientError>{
        let packet=SPacketDestroyEntities::readPacketData(rawPacket).map_err(|error|packet_error(rawPacket.id,error))?;
        self.sharedState.update(|state|if let Some(world)=state.worldClient.as_mut(){for entityId in packet.getEntityIDs(){world.removeEntityFromWorld(*entityId);}});
        Ok(PlayHandlerEvent::None)
    }

    pub fn handleDisconnect(
        &mut self,
        rawPacket: &RawPacket,
    ) -> Result<PlayHandlerEvent, NetHandlerPlayClientError> {
        let packet = SPacketDisconnect::readPacketData(rawPacket)
            .map_err(|error| packet_error(rawPacket.id, error))?;
        Ok(PlayHandlerEvent::Disconnected(packet.getReason().clone()))
    }

    pub fn getGameProfile(&self) -> &GameProfile { &self.profile }
    pub const fn getCurrentServerMaxPlayers(&self) -> u8 { self.currentServerMaxPlayers }
    pub const fn isDoneLoadingTerrain(&self) -> bool { self.doneLoadingTerrain }
    pub const fn getPlayerPosition(&self) -> PlayerPositionState { self.playerPosition }
    pub fn getSharedState(&self) -> SharedPlayClientState { self.sharedState.clone() }
}

fn player_position_state(player: &EntityPlayerSP) -> PlayerPositionState {
    PlayerPositionState {
        posX: player.entity.posX,
        posY: player.entity.posY,
        posZ: player.entity.posZ,
        rotationYaw: player.entity.rotationYaw,
        rotationPitch: player.entity.rotationPitch,
        eyeHeight: player.getEyeHeight(),
    }
}

fn packet_error(
    packetId: i32,
    error: impl std::fmt::Display,
) -> NetHandlerPlayClientError {
    NetHandlerPlayClientError::Packet {
        packetId,
        message: error.to_string(),
    }
}


#[cfg(test)]
mod placement_prediction_tests {
    use super::*;
    use crate::net::minecraft::block::state::IBlockState::IBlockState;
    use crate::net::minecraft::block::SoundType::SoundType;

    fn placement(item_id: i16, pos: BlockPos, state_id: i32) -> ItemBlockPlacement {
        ItemBlockPlacement {
            pos,
            state: IBlockState::fromGlobalStateId(state_id),
            sourceItemId: item_id,
            sourceItemDamage: 0,
        }
    }

    #[test]
    fn source_backed_itemblock_prediction_mutates_world_and_survival_stack() {
        let shared = SharedPlayClientState::new();
        let target = BlockPos::new(0, 65, 0);
        shared.withWrite(|state| {
            state.gameType = GameType::Survival;
            state.worldClient = Some(WorldClient::new(0));
            let mut player = EntityPlayerSP::new(1);
            player.inventory.mainInventory[0] = ItemStack {
                itemId: 50,
                count: 2,
                itemDamage: 0,
                tagCompound: None,
            };
            player.inventoryContainer.putStackInSlot(
                36,
                player.inventory.mainInventory[0].clone(),
            ).unwrap();
            state.thePlayer = Some(player);
        });

        assert!(shared.applyPredictedItemBlockPlacement(
            placement(50, target, (50 << 4) | 5),
            EnumHand::MainHand,
        ));
        shared.withRead(|state| {
            assert_eq!(state.worldClient.as_ref().unwrap().getBlockState(target).getBlockId(), 50);
            let player = state.thePlayer.as_ref().unwrap();
            assert_eq!(player.inventory.mainInventory[0].count, 1);
            assert_eq!(player.inventoryContainer.getSlot(36).unwrap().count, 1);
        });
        let sounds = shared.takeLocalPlayerSoundEvents();
        assert_eq!(sounds.len(), 1);
        assert_eq!(sounds[0].sound, SoundType::forBlockId(50).getPlaceSound());
        assert_eq!(sounds[0].category, SoundCategory::Blocks);
    }

    #[test]
    fn creative_prediction_preserves_stack_and_stale_identity_is_rejected() {
        let shared = SharedPlayClientState::new();
        let target = BlockPos::new(0, 65, 0);
        shared.withWrite(|state| {
            state.gameType = GameType::Creative;
            state.worldClient = Some(WorldClient::new(0));
            let mut player = EntityPlayerSP::new(1);
            player.inventory.mainInventory[0] = ItemStack {
                itemId: 65,
                count: 4,
                itemDamage: 0,
                tagCompound: None,
            };
            state.thePlayer = Some(player);
        });
        assert!(!shared.applyPredictedItemBlockPlacement(
            placement(50, target, (50 << 4) | 5),
            EnumHand::MainHand,
        ));
        assert!(shared.applyPredictedItemBlockPlacement(
            placement(65, target, (65 << 4) | 2),
            EnumHand::MainHand,
        ));
        shared.withRead(|state| {
            assert_eq!(state.thePlayer.as_ref().unwrap().inventory.mainInventory[0].count, 4);
        });
    }

    #[test]
    fn completed_destroy_prediction_sets_air_and_rejects_stale_state() {
        let shared = SharedPlayClientState::new();
        let target = BlockPos::new(3, 70, -2);
        let stone = IBlockState::fromGlobalStateId(1 << 4);
        let dirt = IBlockState::fromGlobalStateId(3 << 4);
        shared.withWrite(|state| {
            let mut world = WorldClient::new(0);
            world.invalidateRegionAndSetBlock(target, stone).unwrap();
            state.worldClient = Some(world);
        });

        assert!(!shared.applyPredictedBlockDestruction(target, dirt));
        assert!(shared.applyPredictedBlockDestruction(target, stone));
        shared.withRead(|state| {
            assert!(state.worldClient.as_ref().unwrap().getBlockState(target).isAir());
        });
        assert!(!shared.applyPredictedBlockDestruction(target, stone));
    }

    #[test]
    fn activation_prediction_applies_only_to_matching_snapshot() {
        let shared = SharedPlayClientState::new();
        let target = BlockPos::new(3, 70, -2);
        let closed = IBlockState::fromGlobalStateId(96 << 4);
        let open = IBlockState::fromGlobalStateId((96 << 4) | 4);
        let stale = IBlockState::fromGlobalStateId((96 << 4) | 8);
        shared.withWrite(|state| {
            let mut world = WorldClient::new(0);
            world.invalidateRegionAndSetBlock(target, closed).unwrap();
            state.worldClient = Some(world);
        });

        assert!(!shared.applyPredictedBlockState(target, stale, open));
        assert!(shared.applyPredictedBlockState(target, closed, open));
        shared.withRead(|state| {
            assert_eq!(state.worldClient.as_ref().unwrap().getBlockState(target), open);
        });
    }

    #[test]
    fn network_handler_publishes_login_success_profile_to_render_state() {
        let shared = SharedPlayClientState::new();
        let id = Uuid::parse_str("12345678-1234-5678-9abc-def012345678").unwrap();
        let profile = GameProfile::new(Some(id), "AuthenticatedName");
        let settings = ClientSettingsSnapshot {
            language: "en_us".to_owned(),
            renderDistanceChunks: 8,
            chatVisibility: EnumChatVisibility::Full,
            chatColours: true,
            modelPartFlags: 0x7f,
            mainHand: EnumHandSide::Right,
        };
        let handler = NetHandlerPlayClient::new(profile.clone(), settings, shared.clone());

        assert_eq!(handler.getGameProfile(), &profile);
        shared.withRead(|state| {
            assert_eq!(state.localGameProfile.as_ref(), Some(&profile));
        });
    }
}
