use crate::compat::Java::JavaRandom;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockBed::BlockBed;
use crate::net::minecraft::block::BlockLiquid;
use crate::net::minecraft::block::BlockLiquid::LiquidMaterial;
use crate::net::minecraft::client::audio::LocalSoundEvent::LocalSoundEvent;
use crate::net::minecraft::client::entity::EntityOtherClient::{ClientEntityKind, ObjectSpawnType};
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::client::particle::ParticleEmitter::ParticleEmitter;
use crate::net::minecraft::entity::ai::attributes::AbstractAttributeMap::AbstractAttributeMap;
use crate::net::minecraft::entity::ai::attributes::AttributeModifier::AttributeModifier;
use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::entity::player::PlayerCapabilities::PlayerCapabilities;
use crate::net::minecraft::entity::Entity::Entity;
use crate::net::minecraft::entity::EntityLivingBase;
use crate::net::minecraft::entity::IJumpingMount::IJumpingMount;
use crate::net::minecraft::inventory::ContainerPlayer::ContainerPlayer;
use crate::net::minecraft::inventory::EntityEquipmentSlot::EntityEquipmentSlot;
use crate::net::minecraft::inventory::OpenContainer::OpenContainer;
use crate::net::minecraft::item::ItemElytra::ItemElytra;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::datasync::DataSerializers::DataValue;
use crate::net::minecraft::network::datasync::EntityDataManager::EntityDataManager;
use crate::net::minecraft::network::play::client::CPacketEntityAction::{
    Action, CPacketEntityAction,
};
use crate::net::minecraft::network::play::client::CPacketInput::CPacketInput;
use crate::net::minecraft::network::play::client::CPacketPlayer::{
    CPacketPlayer, Position, PositionRotation, Rotation,
};
use crate::net::minecraft::network::play::client::CPacketPlayerAbilities::CPacketPlayerAbilities;
use crate::net::minecraft::network::play::client::CPacketVehicleMove::CPacketVehicleMove;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::potion::PotionEffect::PotionEffect;
use crate::net::minecraft::stats::RecipeBook::RecipeBook;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::math::MathHelper::{
    cos as minecraft_cos, sin as minecraft_sin, wrap_degrees_f32,
};
use crate::net::minecraft::util::math::Vec3d::Vec3d;
use crate::net::minecraft::util::CooldownTracker::CooldownTracker;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::EnumHand::EnumHand;
use crate::net::minecraft::util::EnumParticleTypes::EnumParticleTypes;
use crate::net::minecraft::util::FoodStats::FoodStats;
use crate::net::minecraft::util::MovementInputFromOptions::{
    MovementInputFromOptions, MovementKeyState,
};
use crate::net::minecraft::util::SoundCategory::SoundCategory;
use crate::net::minecraft::world::EnumDifficulty::EnumDifficulty;
use crate::net::minecraft::world::GameType::GameType;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Gameplay-bearing subset of MCP `net.minecraft.client.entity.EntityPlayerSP`.
///
/// This class owns local movement input, player physics, riding/passenger state
/// and the exact walking/riding packet-selection paths. Potion and elytra
/// subsystems remain separate ports; water/lava and climbable travel follow
/// MCP 1.12.2.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityPlayerSP {
    pub entity: Entity,
    pub movementInput: MovementInputFromOptions,
    pub entityId: i32,
    pub attributeMap: AbstractAttributeMap,
    /// Inherited MCP `Entity.dataManager`; index 0 stores entity flags.
    pub dataManager: EntityDataManager,
    pub isSprinting: bool,
    pub jumpMovementFactor: f32,
    pub aiMoveSpeed: f32,
    pub capabilities: PlayerCapabilities,
    pub experience: f32,
    pub experienceLevel: i32,
    pub experienceTotal: i32,
    pub score: i32,
    pub inventory: InventoryPlayer,
    pub inventoryContainer: ContainerPlayer,
    /// MCP `EntityPlayer.openContainer`; `None` means the player container is active.
    pub openContainer: Option<OpenContainer>,
    /// MCP `EntityPlayer.recipeBook`, synchronized by `SPacketRecipeBook`.
    pub recipeBook: RecipeBook,
    pub health: f32,
    /// MCP `EntityLivingBase.absorptionAmount`.
    pub absorptionAmount: f32,
    /// MCP `EntityLivingBase.activePotionsMap`, keyed by numeric potion ID.
    pub activePotionEffects: HashMap<u8, PotionEffect>,
    /// MCP `EntityRenderer.itemActivationItem` mirrored on the local player so
    /// the render capture can remain an immutable snapshot.
    pub itemActivationItem: ItemStack,
    pub itemActivationTicks: i32,
    pub itemActivationRandomX: f32,
    pub itemActivationRandomY: f32,
    /// Local-player equivalent of the 30-tick `ParticleEmitter` created for
    /// entity status opcode 35.
    totemParticleEmitter: Option<ParticleEmitter>,
    pub foodStats: FoodStats,
    /// MCP `EntityPlayer.cooldownTracker`.
    pub cooldownTracker: CooldownTracker,
    pub hurtTime: i32,
    pub maxHurtTime: i32,
    pub deathTime: i32,
    pub attackedAtYaw: f32,
    pub hurtResistantTime: i32,
    pub maxHurtResistantTime: i32,
    pub lastStatusOpcode: Option<i8>,
    /// MCP `EntityPlayerSP.permissionLevel`, synchronized by status 24..28.
    pub permissionLevel: i32,
    /// MCP `EntityPlayer.hasReducedDebug`, synchronized by status 22/23.
    pub hasReducedDebug: bool,
    pub swingProgress: f32,
    pub prevSwingProgress: f32,
    /// MCP `EntityLivingBase.limbSwing`: accumulated walk-cycle phase.
    pub limbSwing: f32,
    /// Previous and current smoothed walk amplitude used by ModelBiped.
    pub prevLimbSwingAmount: f32,
    pub limbSwingAmount: f32,
    pub renderYawOffset: f32,
    pub prevRenderYawOffset: f32,
    pub rotationYawHead: f32,
    pub prevRotationYawHead: f32,
    pub onGroundSpeedFactor: f32,
    pub prevOnGroundSpeedFactor: f32,
    pub movedDistance: f32,
    pub prevMovedDistance: f32,
    pub chasingPosX: f64,
    pub chasingPosY: f64,
    pub chasingPosZ: f64,
    pub prevChasingPosX: f64,
    pub prevChasingPosY: f64,
    pub prevChasingPosZ: f64,
    pub cameraYaw: f32,
    pub prevCameraYaw: f32,
    pub cameraPitch: f32,
    /// MCP `EntityLivingBase.ticksElytraFlying`.
    pub ticksElytraFlying: i32,
    pub swingingHand: EnumHand,
    pub activeItemStack: ItemStack,
    pub activeItemStackUseCount: i32,
    pub activeHand: EnumHand,
    pub handActive: bool,
    /// Client-originated sounds from methods that execute directly in
    /// `WorldClient`, rather than arriving as SPacketSoundEffect.
    pendingSoundEvents: Vec<LocalSoundEvent>,
    /// Inherited `Entity.rand` stream used by 1.12.2 sound-pitch formulas.
    soundRandomizer: JavaRandom,
    pub renderArmYaw: f32,
    pub renderArmPitch: f32,
    pub prevRenderArmYaw: f32,
    pub prevRenderArmPitch: f32,
    /// MCP `EntityPlayer.sleeping` state populated by `SPacketUseBed`.
    pub sleeping: bool,
    pub sleepTimer: i32,
    pub bedLocation: Option<BlockPos>,
    pub renderOffsetX: f32,
    pub renderOffsetY: f32,
    pub renderOffsetZ: f32,
    /// MCP `EntityPlayerSP.rowingBoat`; consumed by first-person ItemRenderer.
    pub rowingBoat: bool,
    /// MCP `EntityPlayerSP.horseJumpPowerCounter` and `horseJumpPower`.
    pub horseJumpPowerCounter: i32,
    pub horseJumpPower: f32,
    swingProgressInt: i32,
    isSwingInProgress: bool,
    ticksSinceLastSwing: i32,
    hasValidHealth: bool,
    lastDamage: f32,
    lastReportedPosX: f64,
    lastReportedPosY: f64,
    lastReportedPosZ: f64,
    lastReportedYaw: f32,
    lastReportedPitch: f32,
    positionUpdateTicks: i32,
    prevOnGround: bool,
    serverSneakState: bool,
    serverSprintState: bool,
    jumpTicks: i32,
    sprintToggleTimer: i32,
    flyToggleTimer: i32,
}

const SPRINTING_SPEED_BOOST_ID: Uuid = Uuid::from_u128(0x662A6B8D_DA3E_4C1C_8813_96EA6097278D);
const SPRINTING_SPEED_BOOST_AMOUNT: f64 = 0.30000001192092896_f64;
const SPEED_POTION_MODIFIER_ID: Uuid = Uuid::from_u128(0x91AEAA56_376B_4498_935B_2F7F68070635);
const SLOWNESS_POTION_MODIFIER_ID: Uuid = Uuid::from_u128(0x7107DE5E_7CE8_4030_940E_514C1F160890);
const SPEED_POTION_AMOUNT: f64 = 0.20000000298023224_f64;
const SLOWNESS_POTION_AMOUNT: f64 = -0.15000000596046448_f64;
const PLAYER_SPEED_IN_AIR: f32 = 0.02_f32;
const JUMP_BOOST_POTION_ID: u8 = 8;
const BLINDNESS_POTION_ID: u8 = 15;
const LEVITATION_POTION_ID: u8 = 25;

fn player_attribute_map() -> AbstractAttributeMap {
    let mut map = AbstractAttributeMap::default();
    map.registerAttribute("generic.maxHealth", 20.0);
    map.registerAttribute("generic.movementSpeed", 0.10000000149011612);
    map.registerAttribute("generic.attackDamage", 1.0);
    map.registerAttribute("generic.attackSpeed", 4.0);
    map.registerAttribute("generic.luck", 0.0);
    map
}

impl EntityPlayerSP {
    pub fn new(entityId: i32) -> Self {
        Self {
            entity: Entity::default(),
            movementInput: MovementInputFromOptions::new(),
            entityId,
            attributeMap: player_attribute_map(),
            dataManager: EntityDataManager::default(),
            isSprinting: false,
            jumpMovementFactor: 0.02,
            // EntityLivingBase.landMovementFactor is zero-initialized; the
            // first loaded living update copies the movement-speed attribute
            // into it at EntityPlayer#onLivingUpdate's post-travel stage.
            aiMoveSpeed: 0.0_f32,
            capabilities: PlayerCapabilities::default(),
            experience: 0.0,
            experienceLevel: 0,
            experienceTotal: 0,
            score: 0,
            inventory: InventoryPlayer::default(),
            inventoryContainer: ContainerPlayer::default(),
            openContainer: None,
            recipeBook: RecipeBook::default(),
            health: 20.0,
            absorptionAmount: 0.0,
            activePotionEffects: HashMap::new(),
            itemActivationItem: ItemStack::EMPTY,
            itemActivationTicks: 0,
            itemActivationRandomX: 0.0,
            itemActivationRandomY: 0.0,
            totemParticleEmitter: None,
            foodStats: FoodStats::default(),
            cooldownTracker: CooldownTracker::default(),
            hurtTime: 0,
            maxHurtTime: 0,
            deathTime: 0,
            attackedAtYaw: 0.0,
            hurtResistantTime: 0,
            maxHurtResistantTime: 20,
            lastStatusOpcode: None,
            permissionLevel: 0,
            hasReducedDebug: false,
            swingProgress: 0.0,
            prevSwingProgress: 0.0,
            limbSwing: 0.0,
            prevLimbSwingAmount: 0.0,
            limbSwingAmount: 0.0,
            renderYawOffset: 0.0,
            prevRenderYawOffset: 0.0,
            rotationYawHead: 0.0,
            prevRotationYawHead: 0.0,
            onGroundSpeedFactor: 0.0,
            prevOnGroundSpeedFactor: 0.0,
            movedDistance: 0.0,
            prevMovedDistance: 0.0,
            chasingPosX: 0.0,
            chasingPosY: 0.0,
            chasingPosZ: 0.0,
            prevChasingPosX: 0.0,
            prevChasingPosY: 0.0,
            prevChasingPosZ: 0.0,
            cameraYaw: 0.0,
            prevCameraYaw: 0.0,
            cameraPitch: 0.0,
            ticksElytraFlying: 0,
            swingingHand: EnumHand::MainHand,
            activeItemStack: ItemStack::EMPTY,
            activeItemStackUseCount: 0,
            activeHand: EnumHand::MainHand,
            handActive: false,
            pendingSoundEvents: Vec::new(),
            soundRandomizer: JavaRandom::new(
                (SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as i64)
                    ^ i64::from(entityId),
            ),
            renderArmYaw: 0.0,
            renderArmPitch: 0.0,
            prevRenderArmYaw: 0.0,
            prevRenderArmPitch: 0.0,
            sleeping: false,
            sleepTimer: 0,
            bedLocation: None,
            renderOffsetX: 0.0,
            renderOffsetY: 0.0,
            renderOffsetZ: 0.0,
            rowingBoat: false,
            horseJumpPowerCounter: 0,
            horseJumpPower: 0.0,
            swingProgressInt: 0,
            isSwingInProgress: false,
            ticksSinceLastSwing: 0,
            hasValidHealth: false,
            lastDamage: 0.0,
            lastReportedPosX: 0.0,
            lastReportedPosY: 0.0,
            lastReportedPosZ: 0.0,
            lastReportedYaw: 0.0,
            lastReportedPitch: 0.0,
            positionUpdateTicks: 0,
            prevOnGround: false,
            serverSneakState: false,
            serverSprintState: false,
            jumpTicks: 0,
            sprintToggleTimer: 0,
            flyToggleTimer: 0,
        }
    }

    /// Apply server-owned entity metadata to the local player. This mirrors
    /// `Entity#notifyDataManagerChange` for the inherited flags and
    /// `EntityLivingBase.HAND_STATES` fields used by the 1.12.2 client.
    pub fn applyMetadata(&mut self, entries: impl IntoIterator<Item = (u8, DataValue)>) {
        let previousHandStates = self.dataManager.byte(6, 0);
        self.dataManager.setEntryValues(entries);
        let entityFlags = self.dataManager.byte(0, 0) as u8;
        self.entity.sneaking = (entityFlags & 0x02) != 0;
        let sprinting = (entityFlags & 0x08) != 0;
        if self.isSprinting != sprinting {
            // Server metadata is authoritative for sprint cancellation (for
            // example after a successful attack). Keep the operation-2
            // movement-speed modifier synchronized with the inherited flag.
            self.setSprinting(sprinting);
        }
        // EntityPlayer.PLAYER_SCORE is the VarInt data parameter registered
        // after EntityLivingBase's four entries: protocol metadata id 11.
        self.score = self.dataManager.varInt(11, self.score);

        let handStates = self.dataManager.byte(6, 0);
        if handStates != previousHandStates {
            self.handActive = (handStates & 0x01) != 0;
            self.activeHand = if (handStates & 0x02) != 0 {
                EnumHand::OffHand
            } else {
                EnumHand::MainHand
            };

            if self.handActive {
                let stack = self.getHeldItem(self.activeHand).clone();
                if stack.isEmpty() {
                    self.resetActiveHand();
                } else {
                    self.activeItemStackUseCount = stack.getMaxItemUseDuration();
                    self.activeItemStack = stack;
                }
            } else {
                self.resetActiveHand();
            }
        }
    }

    /// MCP `Entity#isInvisible`, inherited by `EntityPlayerSP`.
    pub fn isInvisible(&self) -> bool {
        (self.dataManager.byte(0, 0) & 0x20) != 0
    }

    /// MCP `Entity#isBurning`, exposed for the common Render fire pass.
    pub fn isBurning(&self) -> bool {
        self.entity.fire > 0 || (self.dataManager.byte(0, 0) & 0x01) != 0
    }

    /// MCP `EntityLivingBase#isElytraFlying` reads entity flag 7.
    pub fn isElytraFlying(&self) -> bool {
        (self.dataManager.byte(0, 0) & 0x80_u8 as i8) != 0
    }

    /// Exact MCP `Entity#setFlag` mutation for inherited metadata index 0.
    fn setEntityFlag(&mut self, flag: u8, enabled: bool) {
        let mask = 1_u8 << flag;
        let mut flags = self.dataManager.byte(0, 0) as u8;
        if enabled {
            flags |= mask;
        } else {
            flags &= !mask;
        }
        self.dataManager.setByte(0, flags as i8);
    }

    /// MCP `EntityPlayer#updateSize`. The candidate AABB is checked before
    /// changing height, so standing up beneath a low ceiling remains blocked.
    fn updateSize(&mut self, world: &WorldClient) {
        let (width, height) = if self.isElytraFlying() {
            (0.6_f32, 0.6_f32)
        } else if self.isPlayerSleeping() {
            (0.2_f32, 0.2_f32)
        } else if self.entity.sneaking {
            (0.6_f32, 1.65_f32)
        } else {
            (0.6_f32, 1.8_f32)
        };

        if self.entity.width == width && self.entity.height == height {
            return;
        }
        let current = self.entity.boundingBox;
        let candidate = AxisAlignedBB::new(
            current.min_x,
            current.min_y,
            current.min_z,
            current.min_x + width as f64,
            current.min_y + height as f64,
            current.min_z + width as f64,
        );
        if world.getCollisionBoxes(candidate).is_empty() {
            self.entity.setSize(width, height);
        }
    }

    /// Client half of MCP `EntityPlayer#trySleep`, invoked only after the
    /// authoritative `SPacketUseBed`. Server-only validation is deliberately
    /// not repeated here.
    pub fn trySleepClient(&mut self, bedState: IBlockState, bedLocation: BlockPos) {
        self.entity.ridingEntityId = None;
        self.rowingBoat = false;
        self.entity.setSize(0.2, 0.2);
        let facing = if BlockBed::isBlockBed(bedState) {
            BlockBed::getFacing(bedState)
        } else {
            crate::net::minecraft::util::EnumFacing::EnumFacing::South
        };
        let (offsetX, _, offsetZ) = facing.offsets();
        self.renderOffsetX = -1.8 * offsetX as f32;
        self.renderOffsetY = 0.0;
        self.renderOffsetZ = -1.8 * offsetZ as f32;
        self.entity.setPosition(
            bedLocation.x as f64 + 0.5 + offsetX as f64 * 0.4,
            bedLocation.y as f64 + 0.6875,
            bedLocation.z as f64 + 0.5 + offsetZ as f64 * 0.4,
        );
        self.entity.motionX = 0.0;
        self.entity.motionY = 0.0;
        self.entity.motionZ = 0.0;
        self.sleeping = true;
        self.sleepTimer = 0;
        self.bedLocation = Some(bedLocation);
    }

    /// Client half of MCP `EntityPlayer#wakeUpPlayer`. The server remains
    /// authoritative for the final safe-exit teleport.
    pub fn wakeUpPlayerClient(&mut self, safeExit: Option<BlockPos>, immediately: bool) {
        self.entity.setSize(0.6, 1.8);
        if let Some(exit) = safeExit.or_else(|| self.bedLocation.map(|bed| bed.up(1))) {
            self.entity.setPosition(
                exit.x as f64 + 0.5,
                exit.y as f64 + 0.1,
                exit.z as f64 + 0.5,
            );
        }
        self.sleeping = false;
        self.sleepTimer = if immediately { 0 } else { 100 };
        self.renderOffsetX = 0.0;
        self.renderOffsetY = 0.0;
        self.renderOffsetZ = 0.0;
    }

    pub fn isPlayerSleeping(&self) -> bool {
        self.sleeping
    }

    pub fn getBedOrientationInDegrees(&self, world: &WorldClient) -> f32 {
        self.bedLocation
            .map(|bed| world.getBlockState(bed))
            .filter(|state| BlockBed::isBlockBed(*state))
            .map(BlockBed::orientationDegrees)
            .unwrap_or(0.0)
    }

    fn updateSleepState(&mut self, world: &WorldClient) {
        if self.sleeping {
            self.sleepTimer = (self.sleepTimer + 1).min(100);
            if self
                .bedLocation
                .is_some_and(|bed| !BlockBed::isBlockBed(world.getBlockState(bed)))
            {
                self.wakeUpPlayerClient(
                    self.bedLocation
                        .and_then(|bed| BlockBed::getSafeExitLocation(world, bed, 0)),
                    true,
                );
            }
        } else if self.sleepTimer > 0 {
            self.sleepTimer += 1;
            if self.sleepTimer >= 110 {
                self.sleepTimer = 0;
            }
        }
    }

    /// Port of MCP `EntityPlayerSP#onUpdate`, including the riding branch
    /// selected by `World#updateEntityWithOptionalForce`.
    pub fn onUpdate(
        &mut self,
        world: &mut WorldClient,
        keys: MovementKeyState,
        gameType: GameType,
    ) -> Vec<RawPacket> {
        // EntityPlayerSP#onUpdate does not enter any inherited entity update
        // until the column containing the player is loaded.
        let loaded_pos = BlockPos::new(
            self.entity.posX.floor() as i32,
            0,
            self.entity.posZ.floor() as i32,
        );
        if !world.isBlockLoaded(loaded_pos) {
            return Vec::new();
        }

        // MCP EntityPlayer#onUpdate applies spectator noclip before the
        // inherited EntityLivingBase update.
        self.entity.noClip = gameType == GameType::Spectator;
        if gameType == GameType::Spectator {
            self.entity.onGround = false;
        }

        // Advanced by the inherited entity update only after the loaded-column
        // guard above, matching EntityPlayerSP#onUpdate.
        self.entity.ticksExisted = self.entity.ticksExisted.wrapping_add(1);
        self.ticksSinceLastSwing = self.ticksSinceLastSwing.saturating_add(1);
        self.tickItemActivation();
        self.tickTotemParticleEmitter(world);

        // MCP Entity#updateRidden first drops a stale/dead vehicle relation.
        // Perform that check before selecting the riding packet branch so a
        // removed vehicle cannot cause one extra CPacketInput/VehicleMove tick.
        if let Some(vehicleId) = self.entity.ridingEntityId {
            let invalidRiding = match world.getBaseEntityByID(vehicleId) {
                Some(vehicle) => vehicle.isDead || !vehicle.passengerIds.contains(&self.entityId),
                None => true,
            };
            if invalidRiding {
                self.entity.ridingEntityId = None;
                self.rowingBoat = false;
            }
        }

        let ridingAtTickStart = self.entity.ridingEntityId;
        if ridingAtTickStart.is_some() {
            // Entity#updateRidden clears all passenger velocity before invoking
            // the virtual player onUpdate method.
            self.entity.motionX = 0.0;
            self.entity.motionY = 0.0;
            self.entity.motionZ = 0.0;
        }

        if self.hurtTime > 0 {
            self.hurtTime -= 1;
        }
        if self.hurtResistantTime > 0 {
            self.hurtResistantTime -= 1;
        }
        self.tickPotionEffects();
        if self.health <= 0.0 {
            self.deathTime = self.deathTime.saturating_add(1);
        } else {
            self.deathTime = 0;
        }

        self.updateActiveHand();

        self.entity.prevPosX = self.entity.posX;
        self.entity.prevPosY = self.entity.posY;
        self.entity.prevPosZ = self.entity.posZ;
        self.entity.prevRotationYaw = self.entity.rotationYaw;
        self.entity.prevRotationPitch = self.entity.rotationPitch;
        self.prevRenderYawOffset = self.renderYawOffset;
        self.prevRotationYawHead = self.rotationYawHead;

        self.prevSwingProgress = self.swingProgress;
        self.updateSleepState(world);
        if self.sleeping {
            self.entity.motionX = 0.0;
            self.entity.motionY = 0.0;
            self.entity.motionZ = 0.0;
            self.movementInput
                .updatePlayerMoveState(MovementKeyState::default());
            // MCP EntityPlayer#onUpdate ticks CooldownTracker even while the
            // player is sleeping; the sleeping rendering/input branch must not
            // freeze item cooldowns.
            self.cooldownTracker.tick();
            self.updateSize(world);
            self.entity.firstUpdate = false;
            return self.onUpdateWalkingPlayer();
        }
        self.updateArmSwingProgress();
        self.entity.handleWaterMovement(world);
        let mut packets = self.onLivingUpdate(world, keys, gameType);
        self.updateCape();
        // MCP EntityPlayer#onUpdate ticks the item CooldownTracker after the
        // inherited living update and cape update, once per loaded client tick.
        self.cooldownTracker.tick();
        self.updateSize(world);

        if let Some(directVehicleId) = self.entity.ridingEntityId {
            // EntityPlayerSP#onUpdate riding packets occur before the vehicle
            // repositions its passenger in Entity#updateRidden.
            packets.push(
                Rotation::new(
                    self.entity.rotationYaw,
                    self.entity.rotationPitch,
                    self.entity.onGround,
                )
                .writePacketData(),
            );
            let input = self.movementInput.movementInput;
            packets.push(
                CPacketInput::new(
                    input.moveStrafe,
                    input.field_192832_b,
                    input.jump,
                    input.sneak,
                )
                .writePacketData(),
            );

            let lowest = world.lowestRidingEntityId(self.entityId, Some(directVehicleId));
            if lowest != self.entityId && world.localPlayerControlsVehicle(lowest, self.entityId) {
                if let Some(vehicle) = world.getBaseEntityByID(lowest) {
                    packets.push(CPacketVehicleMove::fromEntity(vehicle).writePacketData());
                }
            }

            // Finish the source `Entity#updateRidden` and then the
            // EntityPlayerSP override that supplies boat input for the next
            // vehicle tick.
            self.finishUpdateRidden(world, keys);
        } else {
            self.rowingBoat = false;
            packets.extend(self.onUpdateWalkingPlayer());
        }

        self.entity.firstUpdate = false;
        packets
    }

    fn finishUpdateRidden(&mut self, world: &mut WorldClient, keys: MovementKeyState) {
        let Some(vehicleId) = self.entity.ridingEntityId else {
            self.rowingBoat = false;
            return;
        };
        let Some(vehicle) = world.getNonPlayerEntityByID(vehicleId).cloned() else {
            self.entity.ridingEntityId = None;
            self.rowingBoat = false;
            return;
        };
        if vehicle.entity.isDead || !vehicle.entity.passengerIds.contains(&self.entityId) {
            self.entity.ridingEntityId = None;
            self.rowingBoat = false;
            return;
        }

        let mut x = vehicle.entity.posX;
        let mut y = vehicle.entity.posY + self.vehicleMountedYOffset(&vehicle);
        let mut z = vehicle.entity.posZ;
        let isBoat = matches!(
            &vehicle.kind,
            ClientEntityKind::Object {
                objectType: ObjectSpawnType::Boat,
                ..
            }
        );

        if isBoat {
            let mut lateral = 0.0_f32;
            if vehicle.entity.passengerIds.len() > 1 {
                lateral = if vehicle.entity.passengerIds.first().copied() == Some(self.entityId) {
                    0.2
                } else {
                    -0.6
                };
            }
            let offset = Vec3d::new(lateral as f64, 0.0, 0.0).rotate_yaw(
                -vehicle.entity.rotationYaw * 0.017453292 - std::f32::consts::FRAC_PI_2,
            );
            x += offset.x;
            z += offset.z;

            self.entity.rotationYaw += vehicle.boatDeltaRotation;
            self.rotationYawHead += vehicle.boatDeltaRotation;
            self.renderYawOffset = vehicle.entity.rotationYaw;
            let relative = wrap_degrees_f32(self.entity.rotationYaw - vehicle.entity.rotationYaw);
            let clamped = relative.clamp(-105.0, 105.0);
            let correction = clamped - relative;
            self.entity.prevRotationYaw += correction;
            self.entity.rotationYaw += correction;
            self.rotationYawHead = self.entity.rotationYaw;
        } else if let ClientEntityKind::Mob { entityType } = &vehicle.kind {
            match entityType.registryName {
                // EntityLlama#updatePassenger offsets its passenger 0.3 blocks
                // forward from the body yaw rather than using base Entity.
                "llama" => {
                    let yaw = vehicle.renderYawOffset * 0.017453292;
                    x += (0.3_f32 * yaw.sin()) as f64;
                    z -= (0.3_f32 * yaw.cos()) as f64;
                }
                // AbstractHorse#updatePassenger applies its rearing offset
                // after the ordinary mounted-Y position.
                "horse" | "donkey" | "mule" | "skeleton_horse" | "zombie_horse"
                    if vehicle.horsePrevRearingAmount > 0.0 =>
                {
                    let yaw = vehicle.renderYawOffset * 0.017453292;
                    let forward = 0.7 * vehicle.horsePrevRearingAmount;
                    let lift = 0.15 * vehicle.horsePrevRearingAmount;
                    x += (forward * yaw.sin()) as f64;
                    y += lift as f64;
                    z -= (forward * yaw.cos()) as f64;
                    self.renderYawOffset = vehicle.renderYawOffset;
                }
                _ => {}
            }
        }

        self.entity.setPosition(x, y, z);
        self.entity.motionX = 0.0;
        self.entity.motionY = 0.0;
        self.entity.motionZ = 0.0;

        self.rowingBoat = isBoat && (keys.left || keys.right || keys.forward || keys.back);
        if isBoat {
            world.setBoatInputs(vehicleId, keys.left, keys.right, keys.forward, keys.back);
        } else if vehicle.isHorseFamily() {
            let input = self.movementInput.movementInput;
            world.setHorseInputs(
                vehicleId,
                self.entity.rotationYaw,
                self.entity.rotationPitch,
                input.moveStrafe,
                input.field_192832_b,
            );
        }
    }

    fn vehicleMountedYOffset(
        &self,
        vehicle: &crate::net::minecraft::client::entity::EntityOtherClient::EntityOtherClient,
    ) -> f64 {
        match &vehicle.kind {
            ClientEntityKind::Object {
                objectType: ObjectSpawnType::Boat,
                ..
            } => -0.1,
            ClientEntityKind::Object {
                objectType: ObjectSpawnType::Minecart,
                ..
            } => 0.0,
            ClientEntityKind::Mob { entityType } => match entityType.registryName {
                "llama" => vehicle.entity.height as f64 * 0.67,
                "donkey" | "mule" => vehicle.entity.height as f64 * 0.75 - 0.25,
                "skeleton_horse" => vehicle.entity.height as f64 * 0.75 - 0.1875,
                "spider" | "cave_spider" => vehicle.entity.height as f64 * 0.5,
                _ => vehicle.entity.height as f64 * 0.75,
            },
            _ => vehicle.entity.height as f64 * 0.75,
        }
    }

    pub const fn isRowingBoat(&self) -> bool {
        self.rowingBoat
    }

    pub const fn getHorseJumpPower(&self) -> f32 {
        self.horseJumpPower
    }

    /// Minimal land branch of `EntityPlayerSP.onLivingUpdate` plus
    /// `EntityLivingBase.onLivingUpdate/func_191986_a`.
    fn onLivingUpdate(
        &mut self,
        world: &mut WorldClient,
        keys: MovementKeyState,
        gameType: GameType,
    ) -> Vec<RawPacket> {
        // MCP captures the previous tick's input before
        // `MovementInputFromOptions.updatePlayerMoveState`; the double-tap
        // sprint state machine depends on that ordering.
        let previous_input = self.movementInput.movementInput;
        let was_jumping = previous_input.jump;
        let was_sneaking = previous_input.sneak;
        let was_moving_forward = previous_input.field_192832_b >= 0.8;
        let mut packets = Vec::new();
        self.movementInput.updatePlayerMoveState(keys);
        let mut input = self.movementInput.movementInput;
        self.entity.sneaking = input.sneak;

        // EntityPlayerSP.onLivingUpdate: using an item while not riding cuts
        // strafe/forward input to 20 percent and cancels the sprint-toggle
        // window. The movementInput object itself carries the reduced values.
        if self.isHandActive() && !self.entity.isRiding() {
            input.moveStrafe *= 0.2;
            input.field_192832_b *= 0.2;
            self.movementInput.movementInput = input;
            self.sprintToggleTimer = 0;
        }

        // EntityPlayerSP.updateEntityActionState: the first-person arm follows
        // camera yaw/pitch by half the remaining delta every client tick.
        self.prevRenderArmYaw = self.renderArmYaw;
        self.prevRenderArmPitch = self.renderArmPitch;
        self.renderArmPitch += (self.entity.rotationPitch - self.renderArmPitch) * 0.5;
        self.renderArmYaw += (self.entity.rotationYaw - self.renderArmYaw) * 0.5;

        if self.sprintToggleTimer > 0 {
            self.sprintToggleTimer -= 1;
        }
        // Land-player subset of `EntityPlayerSP.onLivingUpdate`. Potion blindness,
        // auto-jump remains pending; elytra initiation follows the source path below;
        // food, collision, sprint and creative/spectator flight ordering follow
        // the original 1.12.2 conditions.
        let can_sprint = self.foodStats.getFoodLevel() > 6 || self.capabilities.allowFlying;
        if self.entity.onGround
            && !was_sneaking
            && !was_moving_forward
            && input.field_192832_b >= 0.8
            && !self.isSprinting
            && can_sprint
            && !self.isHandActive()
            && !self.isPotionActive(BLINDNESS_POTION_ID)
        {
            if self.sprintToggleTimer <= 0 && !keys.sprint {
                self.sprintToggleTimer = 7;
            } else {
                self.setSprinting(true);
            }
        }
        if !self.isSprinting
            && input.field_192832_b >= 0.8
            && can_sprint
            && !self.isHandActive()
            && !self.isPotionActive(BLINDNESS_POTION_ID)
            && keys.sprint
        {
            self.setSprinting(true);
        }
        if self.isSprinting
            && (input.field_192832_b < 0.8 || self.entity.isCollidedHorizontally || !can_sprint)
        {
            self.setSprinting(false);
        }
        // EntityPlayerSP creative/spectator double-tap flight state machine.
        if self.capabilities.allowFlying {
            if gameType == GameType::Spectator {
                if !self.capabilities.isFlying {
                    self.capabilities.isFlying = true;
                    packets.push(CPacketPlayerAbilities::new(&self.capabilities).writePacketData());
                }
            } else if !was_jumping && input.jump {
                if self.flyToggleTimer == 0 {
                    self.flyToggleTimer = 7;
                } else {
                    self.capabilities.isFlying = !self.capabilities.isFlying;
                    packets.push(CPacketPlayerAbilities::new(&self.capabilities).writePacketData());
                    self.flyToggleTimer = 0;
                }
            }
        }

        // Exact MCP `EntityPlayerSP#onLivingUpdate` fall-flying request.
        // The server remains authoritative and sets entity flag 7 through
        // metadata after validating the same chest-slot/durability conditions.
        if input.jump
            && !was_jumping
            && !self.entity.onGround
            && self.entity.motionY < 0.0
            && !self.isElytraFlying()
            && !self.capabilities.isFlying
        {
            if self
                .inventory
                .armorInventory
                .get(2)
                .is_some_and(ItemElytra::isBroken)
            {
                packets.push(
                    CPacketEntityAction::new(self.entityId, Action::StartFallFlying)
                        .writePacketData(),
                );
            }
        }

        if self.capabilities.isFlying {
            if input.sneak {
                input.moveStrafe = (input.moveStrafe as f64 / 0.3) as f32;
                input.field_192832_b = (input.field_192832_b as f64 / 0.3) as f32;
                self.entity.motionY -= (self.capabilities.getFlySpeed() * 3.0) as f64;
            }
            if input.jump {
                self.entity.motionY += (self.capabilities.getFlySpeed() * 3.0) as f64;
            }
            self.movementInput.movementInput = input;
        }

        let ridingHorseId = self.entity.ridingEntityId.filter(|vehicleId| {
            world
                .getNonPlayerEntityByID(*vehicleId)
                .is_some_and(|vehicle| vehicle.isHorseFamily() && IJumpingMount::canJump(vehicle))
        });
        if let Some(vehicleId) = ridingHorseId {
            if self.horseJumpPowerCounter < 0 {
                self.horseJumpPowerCounter += 1;
                if self.horseJumpPowerCounter == 0 {
                    self.horseJumpPower = 0.0;
                }
            }

            if was_jumping && !input.jump {
                self.horseJumpPowerCounter = -10;
                let jumpPower = (self.horseJumpPower * 100.0).floor() as i32;
                world.setHorseJumpPower(vehicleId, jumpPower);
                packets.push(
                    CPacketEntityAction::withAuxData(
                        self.entityId,
                        Action::StartRidingJump,
                        jumpPower,
                    )
                    .writePacketData(),
                );
            } else if !was_jumping && input.jump {
                self.horseJumpPowerCounter = 0;
                self.horseJumpPower = 0.0;
            } else if was_jumping {
                self.horseJumpPowerCounter += 1;
                if self.horseJumpPowerCounter < 10 {
                    self.horseJumpPower = self.horseJumpPowerCounter as f32 * 0.1;
                } else {
                    self.horseJumpPower = 0.8 + 2.0 / (self.horseJumpPowerCounter - 9) as f32 * 0.1;
                }
            }
        } else {
            self.horseJumpPower = 0.0;
        }

        // EntityPlayer#onLivingUpdate enters its inherited chain only after
        // the local sprint/flying/horse state machines above. EntityPlayer
        // decrements flyToggleTimer first, then EntityLivingBase decrements
        // jumpTicks before evaluating this tick's jump input.
        if self.flyToggleTimer > 0 {
            self.flyToggleTimer -= 1;
        }

        // MCP `EntityPlayer#onLivingUpdate` peaceful regeneration, including
        // the World#getGameRules naturalRegeneration gate. Multiplayer clients
        // retain the vanilla default rules unless an integrated-world path
        // later replaces them from authoritative world data.
        if world.getDifficulty() == EnumDifficulty::Peaceful
            && world.getGameRules().getBoolean("naturalRegeneration")
        {
            if self.health < self.getMaxHealth() && self.entity.ticksExisted % 20 == 0 {
                self.heal(1.0);
            }
            if self.foodStats.needFood() && self.entity.ticksExisted % 10 == 0 {
                self.foodStats
                    .setFoodLevel(self.foodStats.getFoodLevel() + 1);
            }
        }
        if self.jumpTicks > 0 {
            self.jumpTicks -= 1;
        }

        if !self.entity.isRiding() {
            if self.entity.motionX.abs() < 0.003 {
                self.entity.motionX = 0.0;
            }
            if self.entity.motionY.abs() < 0.003 {
                self.entity.motionY = 0.0;
            }
            if self.entity.motionZ.abs() < 0.003 {
                self.entity.motionZ = 0.0;
            }

            if input.jump {
                if self.entity.isInWater() || self.entity.isInLava(world) {
                    self.entity.motionY += EntityLivingBase::LIQUID_JUMP_MOTION;
                } else if self.entity.onGround && self.jumpTicks == 0 {
                    self.jump();
                    self.jumpTicks = 10;
                }
            } else {
                self.jumpTicks = 0;
            }

            let move_strafe = input.moveStrafe * 0.98;
            let move_forward = input.field_192832_b * 0.98;
            if self.capabilities.isFlying {
                let previousMotionY = self.entity.motionY;
                let previousJumpMovementFactor = self.jumpMovementFactor;
                self.jumpMovementFactor =
                    self.capabilities.getFlySpeed() * if self.isSprinting { 2.0 } else { 1.0 };
                self.travelLand(world, move_strafe, 0.0, move_forward, gameType);
                self.entity.motionY = previousMotionY * 0.6;
                self.jumpMovementFactor = previousJumpMovementFactor;
                self.entity.fallDistance = 0.0;
                self.setEntityFlag(7, false);
            } else if self.entity.isInWater() {
                self.travelWater(world, move_strafe, 0.0, move_forward);
            } else if self.entity.isInLava(world) {
                self.travelLava(world, move_strafe, 0.0, move_forward);
            } else if self.isElytraFlying() {
                self.travelElytra(world);
            } else {
                self.travelLand(world, move_strafe, 0.0, move_forward, gameType);
            }
        }

        // MCP EntityPlayer#onLivingUpdate updates these inherited movement
        // factors only after EntityLivingBase#onLivingUpdate has integrated
        // this tick. This one-tick ordering is observable when sprint starts or
        // stops and is required by protocol-340 movement simulators.
        self.jumpMovementFactor = PLAYER_SPEED_IN_AIR;
        if self.isSprinting {
            self.jumpMovementFactor += PLAYER_SPEED_IN_AIR * 0.3;
        }
        self.aiMoveSpeed = self.attributeMap.getAttributeValue(
            "generic.movementSpeed",
            self.capabilities.getWalkSpeed() as f64,
        ) as f32;

        if self.entity.onGround && self.capabilities.isFlying && gameType != GameType::Spectator {
            self.capabilities.isFlying = false;
            packets.push(CPacketPlayerAbilities::new(&self.capabilities).writePacketData());
        }

        // EntityLivingBase.func_191986_a updates the model walk cycle after
        // movement and friction. The local player must retain the same fields
        // as remote players because GuiInventory renders the live entity pose.
        self.prevLimbSwingAmount = self.limbSwingAmount;
        let deltaX = self.entity.posX - self.entity.prevPosX;
        let deltaZ = self.entity.posZ - self.entity.prevPosZ;
        let mut amount = ((deltaX * deltaX + deltaZ * deltaZ).sqrt() as f32) * 4.0;
        if amount > 1.0 {
            amount = 1.0;
        }
        self.limbSwingAmount += (amount - self.limbSwingAmount) * 0.4;
        self.limbSwing += self.limbSwingAmount;

        // EntityLivingBase#onUpdate body/head turn state. RenderPlayer consumes
        // these fields in third person and GuiInventory, so they must follow
        // the same movement-facing and head-clamp rules as remote players.
        let squared = (deltaX * deltaX + deltaZ * deltaZ) as f32;
        let mut targetBodyYaw = self.renderYawOffset;
        let mut moved = 0.0_f32;
        self.prevOnGroundSpeedFactor = self.onGroundSpeedFactor;
        let mut groundFactor = 0.0_f32;
        if squared > 0.0025000002_f32 {
            groundFactor = 1.0;
            moved = squared.sqrt() * 3.0;
            let movementYaw = deltaZ.atan2(deltaX).to_degrees() as f32 - 90.0;
            let difference = (wrap_degrees_f32(self.entity.rotationYaw) - movementYaw).abs();
            targetBodyYaw = if 95.0 < difference && difference < 265.0 {
                movementYaw - 180.0
            } else {
                movementYaw
            };
        }
        if self.swingProgress > 0.0 {
            targetBodyYaw = self.entity.rotationYaw;
        }
        if !self.entity.onGround {
            groundFactor = 0.0;
        }
        self.onGroundSpeedFactor += (groundFactor - self.onGroundSpeedFactor) * 0.3;
        moved = self.updateDistance(targetBodyYaw, moved);
        self.prevMovedDistance = self.movedDistance;
        self.prevCameraYaw = self.cameraYaw;
        let mut horizontalMotion = (self.entity.motionX * self.entity.motionX
            + self.entity.motionZ * self.entity.motionZ)
            .sqrt() as f32;
        let mut verticalCamera =
            (-self.entity.motionY * 0.20000000298023224_f64).atan() as f32 * 15.0;
        if horizontalMotion > 0.1 {
            horizontalMotion = 0.1;
        }
        if !self.entity.onGround || self.health <= 0.0 {
            horizontalMotion = 0.0;
        }
        if self.entity.onGround || self.health <= 0.0 {
            verticalCamera = 0.0;
        }
        self.cameraYaw += (horizontalMotion - self.cameraYaw) * 0.4;
        self.cameraPitch += (verticalCamera - self.cameraPitch) * 0.8;
        self.rotationYawHead = self.entity.rotationYaw;
        normalize_previous_angle(self.entity.rotationYaw, &mut self.entity.prevRotationYaw);
        normalize_previous_angle(self.renderYawOffset, &mut self.prevRenderYawOffset);
        normalize_previous_angle(
            self.entity.rotationPitch,
            &mut self.entity.prevRotationPitch,
        );
        normalize_previous_angle(self.rotationYawHead, &mut self.prevRotationYawHead);
        self.movedDistance += moved;
        // EntityLivingBase#onUpdate advances the render timer after the full
        // living/body update. On a remote client it follows synchronized flag
        // 7 directly; equipment validation and flag clearing are server-owned.
        self.ticksElytraFlying = if self.isElytraFlying() {
            self.ticksElytraFlying.saturating_add(1)
        } else {
            0
        };
        packets
    }

    /// MCP `EntityPlayer#updateCape`: a critically damped trailing point used
    /// by `LayerCape` rather than an approximation based on instantaneous
    /// velocity.
    fn updateCape(&mut self) {
        self.prevChasingPosX = self.chasingPosX;
        self.prevChasingPosY = self.chasingPosY;
        self.prevChasingPosZ = self.chasingPosZ;
        let deltaX = self.entity.posX - self.chasingPosX;
        let deltaY = self.entity.posY - self.chasingPosY;
        let deltaZ = self.entity.posZ - self.chasingPosZ;
        if deltaX.abs() > 10.0 {
            self.chasingPosX = self.entity.posX;
            self.prevChasingPosX = self.chasingPosX;
        }
        if deltaY.abs() > 10.0 {
            self.chasingPosY = self.entity.posY;
            self.prevChasingPosY = self.chasingPosY;
        }
        if deltaZ.abs() > 10.0 {
            self.chasingPosZ = self.entity.posZ;
            self.prevChasingPosZ = self.chasingPosZ;
        }
        self.chasingPosX += deltaX * 0.25;
        self.chasingPosY += deltaY * 0.25;
        self.chasingPosZ += deltaZ * 0.25;
    }

    fn updateDistance(&mut self, targetBodyYaw: f32, mut distance: f32) -> f32 {
        let delta = wrap_degrees_f32(targetBodyYaw - self.renderYawOffset);
        self.renderYawOffset += delta * 0.3;
        let mut relative = wrap_degrees_f32(self.entity.rotationYaw - self.renderYawOffset);
        let reversed = relative < -90.0 || relative >= 90.0;
        relative = relative.clamp(-75.0, 75.0);
        self.renderYawOffset = self.entity.rotationYaw - relative;
        if relative * relative > 2500.0 {
            self.renderYawOffset += relative * 0.2;
        }
        if reversed {
            distance *= -1.0;
        }
        distance
    }

    /// Port of `EntityPlayer.getDigSpeed` for the state currently represented
    /// by this client. Base item speed, Efficiency, Aqua Affinity, liquid and
    /// airborne penalties are exact; potion modifiers await the potion system.
    pub fn getDigSpeed(&self, world: &WorldClient, state: IBlockState) -> f32 {
        let mut speed = self.inventory.getStrVsBlock(state);
        if speed > 1.0 {
            let held = self.inventory.getCurrentItem();
            let efficiency = held.getEnchantmentLevel(32);
            if efficiency > 0 && !held.isEmpty() {
                speed += (efficiency * efficiency + 1) as f32;
            }
        }
        if self.isInsideWater(world) && self.inventory.armorInventory[3].getEnchantmentLevel(6) == 0
        {
            speed /= 5.0;
        }
        if !self.entity.onGround {
            speed /= 5.0;
        }
        speed
    }

    pub fn canHarvestBlock(&self, state: IBlockState) -> bool {
        self.inventory.canHarvestBlock(state)
    }

    /// Exact MCP EntityLivingBase#setSprinting attribute mutation. The local
    /// client applies the same unsaved operation-2 modifier immediately; the
    /// next EntityPlayer#onLivingUpdate copies the resulting attribute value
    /// into landMovementFactor after this tick's travel step.
    pub fn setSprinting(&mut self, sprinting: bool) {
        self.isSprinting = sprinting;
        self.setEntityFlag(3, sprinting);
        if let Some(movementSpeed) = self
            .attributeMap
            .getAttributeInstanceByNameMut("generic.movementSpeed")
        {
            movementSpeed.removeModifier(SPRINTING_SPEED_BOOST_ID);
            if sprinting {
                movementSpeed.applyModifier(AttributeModifier::new(
                    SPRINTING_SPEED_BOOST_ID,
                    SPRINTING_SPEED_BOOST_AMOUNT,
                    2,
                ));
            }
        }
    }

    fn jump(&mut self) {
        self.entity.motionY = 0.41999998688697815_f64;
        if let Some(effect) = self.activePotionEffects.get(&JUMP_BOOST_POTION_ID) {
            self.entity.motionY += (f32::from(effect.getAmplifier()) + 1.0) as f64 * 0.1;
        }
        if self.isSprinting {
            let yaw = self.entity.rotationYaw * 0.017453292_f32;
            self.entity.motionX -= (minecraft_sin(yaw) * 0.2) as f64;
            self.entity.motionZ += (minecraft_cos(yaw) * 0.2) as f64;
        }
    }

    /// Client-world branch of MCP 1.12.2 EntityLivingBase#func_191986_a while
    /// entity flag 7 is set. Wall damage and clearing the flag are server-only
    /// in the source and therefore deliberately absent here.
    fn travelElytra(&mut self, world: &WorldClient) {
        if self.entity.motionY > -0.5 {
            self.entity.fallDistance = 1.0;
        }

        let look = self.getLook(1.0);
        let pitchRadians = self.entity.rotationPitch * 0.017453292_f32;
        let lookHorizontal = (look.x * look.x + look.z * look.z).sqrt();
        let horizontalSpeed = (self.entity.motionX * self.entity.motionX
            + self.entity.motionZ * self.entity.motionZ)
            .sqrt();
        let lookLength = look.length();
        let mut lift = minecraft_cos(pitchRadians);
        lift = (lift as f64 * lift as f64 * 1.0_f64.min(lookLength / 0.4_f64)) as f32;
        self.entity.motionY += -0.08 + lift as f64 * 0.06;

        if self.entity.motionY < 0.0 && lookHorizontal > 0.0 {
            let descentTransfer = self.entity.motionY * -0.1 * lift as f64;
            self.entity.motionY += descentTransfer;
            self.entity.motionX += look.x * descentTransfer / lookHorizontal;
            self.entity.motionZ += look.z * descentTransfer / lookHorizontal;
        }

        if pitchRadians < 0.0 {
            let climbTransfer = horizontalSpeed * (-minecraft_sin(pitchRadians)) as f64 * 0.04;
            self.entity.motionY += climbTransfer * 3.2;
            self.entity.motionX -= look.x * climbTransfer / lookHorizontal;
            self.entity.motionZ -= look.z * climbTransfer / lookHorizontal;
        }

        if lookHorizontal > 0.0 {
            self.entity.motionX +=
                (look.x / lookHorizontal * horizontalSpeed - self.entity.motionX) * 0.1;
            self.entity.motionZ +=
                (look.z / lookHorizontal * horizontalSpeed - self.entity.motionZ) * 0.1;
        }

        self.entity.motionX *= 0.9900000095367432_f64;
        self.entity.motionY *= 0.9800000190734863_f64;
        self.entity.motionZ *= 0.9900000095367432_f64;
        let (motionX, motionY, motionZ) = (
            self.entity.motionX,
            self.entity.motionY,
            self.entity.motionZ,
        );
        self.entity
            .moveEntityLivingWithContext(world, self.entityId, motionX, motionY, motionZ);
    }

    /// Land branch of MCP `EntityLivingBase#travel`, including ladder, vine
    /// and aligned-open-trapdoor traversal.
    fn travelLand(
        &mut self,
        world: &WorldClient,
        strafe: f32,
        vertical: f32,
        forward: f32,
        gameType: GameType,
    ) {
        let below = BlockPos::new(
            self.entity.posX.floor() as i32,
            (self.entity.boundingBox.min_y - 1.0).floor() as i32,
            self.entity.posZ.floor() as i32,
        );

        let mut friction = 0.91_f32;
        if self.entity.onGround {
            friction = world.getSlipperiness(below) * 0.91;
        }

        let acceleration_factor = 0.16277136_f32 / (friction * friction * friction);
        let acceleration = if self.entity.onGround {
            self.aiMoveSpeed * acceleration_factor
        } else {
            self.jumpMovementFactor
        };

        self.entity
            .func_191958_b(strafe, vertical, forward, acceleration);

        friction = 0.91;
        if self.entity.onGround {
            let below_after_acceleration = BlockPos::new(
                self.entity.posX.floor() as i32,
                (self.entity.boundingBox.min_y - 1.0).floor() as i32,
                self.entity.posZ.floor() as i32,
            );
            friction = world.getSlipperiness(below_after_acceleration) * 0.91;
        }

        if EntityLivingBase::isOnLadder(world, &self.entity, gameType == GameType::Spectator) {
            const LIMIT: f64 = EntityLivingBase::LADDER_HORIZONTAL_LIMIT;
            self.entity.motionX = self.entity.motionX.clamp(-LIMIT, LIMIT);
            self.entity.motionZ = self.entity.motionZ.clamp(-LIMIT, LIMIT);
            self.entity.fallDistance = 0.0;
            if self.entity.motionY < -LIMIT {
                self.entity.motionY = -LIMIT;
            }
            if self.entity.sneaking && self.entity.motionY < 0.0 {
                self.entity.motionY = 0.0;
            }
        }

        let (motionX, motionY, motionZ) = (
            self.entity.motionX,
            self.entity.motionY,
            self.entity.motionZ,
        );
        self.entity
            .moveEntityLivingWithContext(world, self.entityId, motionX, motionY, motionZ);

        if self.entity.isCollidedHorizontally
            && EntityLivingBase::isOnLadder(world, &self.entity, gameType == GameType::Spectator)
        {
            self.entity.motionY = 0.2;
        }

        if let Some(effect) = self.activePotionEffects.get(&LEVITATION_POTION_ID) {
            let target = 0.05 * (f64::from(effect.getAmplifier()) + 1.0);
            self.entity.motionY += (target - self.entity.motionY) * 0.2;
        } else {
            self.entity.motionY -= 0.08;
        }
        self.entity.motionY *= 0.9800000190734863;
        self.entity.motionX *= friction as f64;
        self.entity.motionZ *= friction as f64;
    }

    /// Water branch of MCP `EntityLivingBase#travel`. Depth Strider uses boots
    /// slot 0, enchantment ID 8, and the same airborne halving rule as 1.12.2.
    fn travelWater(&mut self, world: &WorldClient, strafe: f32, vertical: f32, forward: f32) {
        let start_y = self.entity.posY;
        let mut slowdown = 0.8_f32;
        let mut acceleration = 0.02_f32;
        let mut depth_strider = self.inventory.armorInventory[0].getEnchantmentLevel(8) as f32;
        if depth_strider > 3.0 {
            depth_strider = 3.0;
        }
        if !self.entity.onGround {
            depth_strider *= 0.5;
        }
        if depth_strider > 0.0 {
            slowdown += (0.54600006_f32 - slowdown) * depth_strider / 3.0;
            acceleration += (self.aiMoveSpeed - acceleration) * depth_strider / 3.0;
        }
        self.entity
            .func_191958_b(strafe, vertical, forward, acceleration);
        let (motionX, motionY, motionZ) = (
            self.entity.motionX,
            self.entity.motionY,
            self.entity.motionZ,
        );
        self.entity
            .moveEntityLivingWithContext(world, self.entityId, motionX, motionY, motionZ);
        self.entity.motionX *= slowdown as f64;
        self.entity.motionY *= 0.800000011920929_f64;
        self.entity.motionZ *= slowdown as f64;
        self.entity.motionY -= 0.02;
        if self.entity.isCollidedHorizontally
            && self.entity.isOffsetPositionInLiquid(
                world,
                self.entity.motionX,
                self.entity.motionY + 0.6000000238418579_f64 - self.entity.posY + start_y,
                self.entity.motionZ,
            )
        {
            self.entity.motionY = EntityLivingBase::LIQUID_WALL_EXIT_MOTION;
        }
    }

    /// Lava branch of MCP `EntityLivingBase#travel`.
    fn travelLava(&mut self, world: &WorldClient, strafe: f32, vertical: f32, forward: f32) {
        let start_y = self.entity.posY;
        self.entity.func_191958_b(strafe, vertical, forward, 0.02);
        let (motionX, motionY, motionZ) = (
            self.entity.motionX,
            self.entity.motionY,
            self.entity.motionZ,
        );
        self.entity
            .moveEntityLivingWithContext(world, self.entityId, motionX, motionY, motionZ);
        self.entity.motionX *= 0.5;
        self.entity.motionY *= 0.5;
        self.entity.motionZ *= 0.5;
        self.entity.motionY -= 0.02;
        if self.entity.isCollidedHorizontally
            && self.entity.isOffsetPositionInLiquid(
                world,
                self.entity.motionX,
                self.entity.motionY + 0.6000000238418579_f64 - self.entity.posY + start_y,
                self.entity.motionZ,
            )
        {
            self.entity.motionY = EntityLivingBase::LIQUID_WALL_EXIT_MOTION;
        }
    }

    /// `Entity#isInsideOfMaterial(Material.WATER)` specialized for the player
    /// eye position. It is used by the mining-speed penalty and underwater
    /// overlays once that renderer is connected.
    pub fn isInsideWater(&self, world: &WorldClient) -> bool {
        // MCP `Entity#isInsideOfMaterial` returns false while riding a boat.
        if let Some(vehicleId) = self.entity.ridingEntityId {
            if world
                .getNonPlayerEntityByID(vehicleId)
                .is_some_and(|vehicle| {
                    matches!(
                        &vehicle.kind,
                        ClientEntityKind::Object {
                            objectType: ObjectSpawnType::Boat,
                            ..
                        }
                    )
                })
            {
                return false;
            }
        }
        let eye_y = self.entity.posY + self.getEyeHeight() as f64;
        let pos = BlockPos::new(
            self.entity.posX.floor() as i32,
            eye_y.floor() as i32,
            self.entity.posZ.floor() as i32,
        );
        let state = world.getBlockState(pos);
        if LiquidMaterial::fromState(state) != Some(LiquidMaterial::Water) {
            return false;
        }
        let submerged_offset =
            BlockLiquid::getLiquidHeightPercent(state.getMetadata()) - 0.11111111_f32;
        let surface = (pos.y + 1) as f64 - submerged_offset as f64;
        eye_y < surface
    }

    /// Port of `EntityPlayerSP.onUpdateWalkingPlayer`. The returned packets are
    /// sent in-order by the network thread during the same 20 TPS client tick.
    pub fn onUpdateWalkingPlayer(&mut self) -> Vec<RawPacket> {
        let mut packets = Vec::with_capacity(3);

        if self.isSprinting != self.serverSprintState {
            let action = if self.isSprinting {
                Action::StartSprinting
            } else {
                Action::StopSprinting
            };
            packets.push(CPacketEntityAction::new(self.entityId, action).writePacketData());
            self.serverSprintState = self.isSprinting;
        }

        if self.entity.sneaking != self.serverSneakState {
            let action = if self.entity.sneaking {
                Action::StartSneaking
            } else {
                Action::StopSneaking
            };
            packets.push(CPacketEntityAction::new(self.entityId, action).writePacketData());
            self.serverSneakState = self.entity.sneaking;
        }

        let d0 = self.entity.posX - self.lastReportedPosX;
        let d1 = self.entity.boundingBox.min_y - self.lastReportedPosY;
        let d2 = self.entity.posZ - self.lastReportedPosZ;
        let d3 = (self.entity.rotationYaw - self.lastReportedYaw) as f64;
        let d4 = (self.entity.rotationPitch - self.lastReportedPitch) as f64;
        self.positionUpdateTicks += 1;
        let moving = d0 * d0 + d1 * d1 + d2 * d2 > 9.0e-4 || self.positionUpdateTicks >= 20;
        let rotating = d3 != 0.0 || d4 != 0.0;

        if moving && rotating {
            packets.push(
                PositionRotation::new(
                    self.entity.posX,
                    self.entity.boundingBox.min_y,
                    self.entity.posZ,
                    self.entity.rotationYaw,
                    self.entity.rotationPitch,
                    self.entity.onGround,
                )
                .writePacketData(),
            );
        } else if moving {
            packets.push(
                Position::new(
                    self.entity.posX,
                    self.entity.boundingBox.min_y,
                    self.entity.posZ,
                    self.entity.onGround,
                )
                .writePacketData(),
            );
        } else if rotating {
            packets.push(
                Rotation::new(
                    self.entity.rotationYaw,
                    self.entity.rotationPitch,
                    self.entity.onGround,
                )
                .writePacketData(),
            );
        } else if self.prevOnGround != self.entity.onGround {
            packets.push(CPacketPlayer::new(self.entity.onGround).writePacketData());
        }

        if moving {
            self.lastReportedPosX = self.entity.posX;
            self.lastReportedPosY = self.entity.boundingBox.min_y;
            self.lastReportedPosZ = self.entity.posZ;
            self.positionUpdateTicks = 0;
        }

        if rotating {
            self.lastReportedYaw = self.entity.rotationYaw;
            self.lastReportedPitch = self.entity.rotationPitch;
        }

        self.prevOnGround = self.entity.onGround;
        packets
    }

    pub const fn getAbsorptionAmount(&self) -> f32 {
        self.absorptionAmount
    }

    pub fn setAbsorptionAmount(&mut self, amount: f32) {
        self.absorptionAmount = amount.max(0.0);
    }

    pub fn isPotionActive(&self, potionId: u8) -> bool {
        self.activePotionEffects.contains_key(&potionId)
    }

    /// MCP `EntityLivingBase#addPotionEffect` for synchronized client effects.
    /// The packet contains the server's current combined effect; replacing the
    /// prior entry is therefore the authoritative client operation.
    pub fn addPotionEffect(&mut self, effect: PotionEffect) {
        let potionId = effect.getPotionId();
        if let Some(previous) = self.activePotionEffects.remove(&potionId) {
            self.removePotionAttributeModifier(previous);
        }
        self.applyPotionAttributeModifier(effect);
        self.activePotionEffects.insert(potionId, effect);
    }

    pub fn removeActivePotionEffect(&mut self, potionId: u8) -> Option<PotionEffect> {
        let removed = self.activePotionEffects.remove(&potionId);
        if let Some(effect) = removed {
            self.removePotionAttributeModifier(effect);
        }
        removed
    }

    fn applyPotionAttributeModifier(&mut self, effect: PotionEffect) {
        // Potion#applyAttributesModifiersToEntity scales the registered base
        // amount by amplifier + 1 while preserving UUID and operation.
        let amplifier = f64::from(effect.getAmplifier()) + 1.0;
        let movementModifier = match effect.getPotionId() {
            1 => Some((SPEED_POTION_MODIFIER_ID, SPEED_POTION_AMOUNT * amplifier)),
            2 => Some((
                SLOWNESS_POTION_MODIFIER_ID,
                SLOWNESS_POTION_AMOUNT * amplifier,
            )),
            _ => None,
        };
        if let Some((id, amount)) = movementModifier {
            if let Some(movementSpeed) = self
                .attributeMap
                .getAttributeInstanceByNameMut("generic.movementSpeed")
            {
                movementSpeed.removeModifier(id);
                movementSpeed.applyModifier(AttributeModifier::new(id, amount, 2));
            }
        }

        // PotionAbsorption#applyAttributesModifiersToEntity.
        if effect.getPotionId() == 22 {
            self.setAbsorptionAmount(
                self.getAbsorptionAmount() + 4.0 * (f32::from(effect.getAmplifier()) + 1.0),
            );
        }
    }

    fn removePotionAttributeModifier(&mut self, effect: PotionEffect) {
        let movementModifierId = match effect.getPotionId() {
            1 => Some(SPEED_POTION_MODIFIER_ID),
            2 => Some(SLOWNESS_POTION_MODIFIER_ID),
            _ => None,
        };
        if let Some(id) = movementModifierId {
            if let Some(movementSpeed) = self
                .attributeMap
                .getAttributeInstanceByNameMut("generic.movementSpeed")
            {
                movementSpeed.removeModifier(id);
            }
        }

        // PotionAbsorption#removeAttributesModifiersFromEntity.
        if effect.getPotionId() == 22 {
            self.setAbsorptionAmount(
                self.getAbsorptionAmount() - 4.0 * (f32::from(effect.getAmplifier()) + 1.0),
            );
        }
    }

    fn tickPotionEffects(&mut self) {
        let expired = self
            .activePotionEffects
            .iter_mut()
            .filter_map(|(&potionId, effect)| (!effect.tickDuration()).then_some(potionId))
            .collect::<Vec<_>>();
        for potionId in expired {
            self.removeActivePotionEffect(potionId);
        }
    }

    /// MCP `EntityRenderer#func_190565_a` plus the opcode-35 particle
    /// emitter setup from `NetHandlerPlayClient#handleEntityStatus`.
    pub fn activateTotem(&mut self, random_x: f32, random_y: f32) {
        self.itemActivationItem = ItemStack {
            itemId: 449,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        };
        self.itemActivationTicks = 40;
        self.itemActivationRandomX = random_x;
        self.itemActivationRandomY = random_y;
        self.totemParticleEmitter = Some(ParticleEmitter::new(
            self.entityId,
            EnumParticleTypes::Totem,
            30,
        ));
        // Opcode 35 sound ownership remains in NetHandlerPlayClient, matching
        // the dedicated 1.12.2 branch that bypasses handleStatusUpdate.
    }

    fn tickItemActivation(&mut self) {
        if self.itemActivationTicks > 0 {
            self.itemActivationTicks -= 1;
            if self.itemActivationTicks == 0 {
                self.itemActivationItem = ItemStack::EMPTY;
            }
        }
    }

    fn tickTotemParticleEmitter(&mut self, world: &mut WorldClient) {
        let Some(emitter) = self.totemParticleEmitter.as_mut() else {
            return;
        };
        world.queueParticleSpawns(emitter.onUpdate(&self.entity));
        if emitter.isExpired() {
            self.totemParticleEmitter = None;
        }
    }

    /// MCP `EntityLivingBase#heal`.
    pub fn heal(&mut self, healAmount: f32) {
        if self.health > 0.0 {
            self.setHealth(self.health + healAmount);
        }
    }

    /// MCP `EntityLivingBase#setHealth`, clamped to max health.
    pub fn setHealth(&mut self, health: f32) {
        self.health = health.clamp(0.0, self.getMaxHealth());
    }

    /// MCP `EntityLivingBase#getMaxHealth` through the MAX_HEALTH attribute.
    pub fn getMaxHealth(&self) -> f32 {
        self.attributeMap
            .getAttributeValue("generic.maxHealth", 20.0) as f32
    }

    /// MCP `EntityLivingBase#shouldHeal`.
    pub fn shouldHeal(&self) -> bool {
        self.health > 0.0 && self.health < self.getMaxHealth()
    }

    /// MCP `EntityPlayer#getTotalArmorValue`.
    pub fn getTotalArmorValue(&self) -> i32 {
        self.inventory
            .armorInventory
            .iter()
            .enumerate()
            .fold(0, |total, (index, stack)| {
                let Some(definition) =
                    crate::net::minecraft::item::ItemArmor::ItemArmor::definition(stack.itemId)
                else {
                    return total;
                };
                let slot = match index {
                    0 => EntityEquipmentSlot::Feet,
                    1 => EntityEquipmentSlot::Legs,
                    2 => EntityEquipmentSlot::Chest,
                    _ => EntityEquipmentSlot::Head,
                };
                total + definition.material.damageReduction(slot)
            })
    }

    /// MCP `Entity#getAir`: data-manager AIR value, 300 when full.
    pub fn getAir(&self) -> i32 {
        self.dataManager.varInt(1, 300)
    }

    /// MCP `EntityPlayer#canPlayerEdit`. Adventure-mode edit permission is
    /// granted by `CanPlaceOn` or an item that can edit blocks; survival and
    /// creative normally have `capabilities.allowEdit=true`.
    pub fn canPlayerEdit(
        &self,
        world: &WorldClient,
        pos: BlockPos,
        facing: EnumFacing,
        stack: &ItemStack,
    ) -> bool {
        if self.capabilities.allowEdit {
            return true;
        }
        if stack.isEmpty() {
            return false;
        }
        let support = pos.offset(facing.opposite(), 1);
        let block = world.getBlockState(support).getBlock();
        stack.canPlaceOn(block) || stack.canEditBlocks()
    }

    pub fn handleStatusUpdate(&mut self, opcode: i8) {
        self.lastStatusOpcode = Some(opcode);
        if opcode == 9 {
            self.onItemUseFinishClient();
            return;
        }
        match opcode {
            22 => self.hasReducedDebug = true,
            23 => self.hasReducedDebug = false,
            24..=28 => self.permissionLevel = (opcode - 24) as i32,
            2 | 33 | 36 | 37 => {
                self.hurtResistantTime = self.maxHurtResistantTime;
                self.maxHurtTime = 10;
                self.hurtTime = self.maxHurtTime;
                self.attackedAtYaw = 0.0;
                if opcode == 33 {
                    let pitch = (self.soundRandomizer.next_f32() - self.soundRandomizer.next_f32())
                        * 0.2
                        + 1.0;
                    self.queueSoundAtPlayer(
                        "enchant.thorns.hit",
                        SoundCategory::Players,
                        1.0,
                        pitch,
                    );
                }
                let sound = match opcode {
                    36 => "entity.player.hurt_drown",
                    37 => "entity.player.hurt_on_fire",
                    _ => "entity.player.hurt",
                };
                let pitch =
                    (self.soundRandomizer.next_f32() - self.soundRandomizer.next_f32()) * 0.2 + 1.0;
                self.queueSoundAtPlayer(sound, SoundCategory::Players, 1.0, pitch);
            }
            3 => {
                let pitch =
                    (self.soundRandomizer.next_f32() - self.soundRandomizer.next_f32()) * 0.2 + 1.0;
                self.queueSoundAtPlayer("entity.player.death", SoundCategory::Players, 1.0, pitch);
                self.health = 0.0;
            }
            29 => {
                let pitch = 0.8 + self.soundRandomizer.next_f32() * 0.4;
                self.queueSoundAtPlayer("item.shield.block", SoundCategory::Players, 1.0, pitch);
            }
            30 => {
                let pitch = 0.8 + self.soundRandomizer.next_f32() * 0.4;
                self.queueSoundAtPlayer("item.shield.break", SoundCategory::Players, 0.8, pitch);
            }
            _ => {}
        }
    }

    /// Port of `EntityPlayerSP.setPlayerSPHealth` state transitions. Sound and
    /// camera feedback remain renderer/audio responsibilities.
    pub fn setPlayerSPHealth(&mut self, health: f32) {
        if self.hasValidHealth {
            let damage = self.health - health;
            if damage <= 0.0 {
                self.health = health;
                if damage < 0.0 {
                    self.hurtResistantTime = self.maxHurtResistantTime / 2;
                }
            } else {
                self.lastDamage = damage;
                self.hurtResistantTime = self.maxHurtResistantTime;
                self.health = health;
                self.maxHurtTime = 10;
                self.hurtTime = self.maxHurtTime;
            }
        } else {
            self.health = health;
            self.hasValidHealth = true;
        }
    }

    /// MCP `EntityLivingBase.swingArm` with the 1.12.2 six-tick base animation.
    /// Potion-adjusted arm timing is added when concrete effect state exists.
    pub fn swingArm(&mut self, hand: EnumHand) {
        if !self.isSwingInProgress || self.swingProgressInt >= 3 {
            self.swingProgressInt = -1;
            self.isSwingInProgress = true;
            self.swingingHand = hand;
        }
    }

    fn updateArmSwingProgress(&mut self) {
        if self.isSwingInProgress {
            self.swingProgressInt += 1;
            if self.swingProgressInt >= 6 {
                self.swingProgressInt = 0;
                self.isSwingInProgress = false;
            }
            self.swingProgress = self.swingProgressInt as f32 / 6.0;
        } else {
            self.swingProgress = 0.0;
        }
    }

    pub fn getSwingProgress(&self, partialTicks: f32) -> f32 {
        let mut delta = self.swingProgress - self.prevSwingProgress;
        if delta < 0.0 {
            delta += 1.0;
        }
        self.prevSwingProgress + delta * partialTicks.clamp(0.0, 1.0)
    }

    /// Base 1.12.2 player attack-speed attribute is 4.0, producing a five-tick
    /// cooldown. Item attribute modifiers are applied when the attribute map is
    /// ported; this keeps the original timing source rather than a render-only
    /// approximation.
    pub fn getCooledAttackStrength(&self, adjustTicks: f32) -> f32 {
        ((self.ticksSinceLastSwing as f32 + adjustTicks) / 5.0).clamp(0.0, 1.0)
    }

    pub fn resetCooldown(&mut self) {
        self.ticksSinceLastSwing = 0;
    }

    pub fn getHeldItem(&self, hand: EnumHand) -> &ItemStack {
        match hand {
            EnumHand::MainHand => self.inventory.getCurrentItem(),
            EnumHand::OffHand => self
                .inventory
                .offHandInventory
                .first()
                .unwrap_or(&ItemStack::EMPTY),
        }
    }

    /// Client-side half of `ItemBlock#onItemUse` stack consumption. The held
    /// `ItemStack`, `inventoryContainer`, and any open container's mirrored
    /// player slots must remain one logical inventory, as they are in vanilla.
    /// Returns false if a concurrent packet or input event changed the held
    /// item after the interaction snapshot was taken.
    pub fn consumeHeldItemForPlacement(
        &mut self,
        hand: EnumHand,
        expectedItemId: i16,
        expectedItemDamage: i16,
    ) -> bool {
        let (containerSlot, updated) = match hand {
            EnumHand::MainHand => {
                let index = self.inventory.currentItem;
                if !(0..9).contains(&index) {
                    return false;
                }
                let stack = &mut self.inventory.mainInventory[index as usize];
                if stack.isEmpty()
                    || stack.itemId != expectedItemId
                    || stack.itemDamage != expectedItemDamage
                {
                    return false;
                }
                stack.shrink(1);
                (36 + index, stack.clone())
            }
            EnumHand::OffHand => {
                let Some(stack) = self.inventory.offHandInventory.first_mut() else {
                    return false;
                };
                if stack.isEmpty()
                    || stack.itemId != expectedItemId
                    || stack.itemDamage != expectedItemDamage
                {
                    return false;
                }
                stack.shrink(1);
                (45, stack.clone())
            }
        };
        let _ = self
            .inventoryContainer
            .putStackInSlot(containerSlot, updated);
        if let Some(container) = self.openContainer.as_mut() {
            container.syncFromPlayerInventory(&self.inventory);
        }
        true
    }

    pub fn setActiveHand(&mut self, hand: EnumHand) -> bool {
        let stack = self.getHeldItem(hand).clone();
        if stack.isEmpty() || self.isHandActive() || stack.getMaxItemUseDuration() <= 0 {
            return false;
        }
        self.activeItemStack = stack;
        self.activeItemStackUseCount = self.activeItemStack.getMaxItemUseDuration();
        self.activeHand = hand;
        self.handActive = true;
        true
    }

    fn updateActiveHand(&mut self) {
        if !self.handActive {
            return;
        }
        let held = self.getHeldItem(self.activeHand).clone();
        if held.itemId == self.activeItemStack.itemId
            && held.itemDamage == self.activeItemStack.itemDamage
            && ItemStack::areItemStackTagsEqual(&held, &self.activeItemStack)
        {
            // Exact remote-world branch of EntityLivingBase#updateActiveHand.
            // The server sends status 9 at completion; the client owns the
            // cadence feedback while inventory mutation remains authoritative.
            if self.activeItemStackUseCount <= 25 && self.activeItemStackUseCount % 4 == 0 {
                self.updateItemUseSound();
            }
            self.activeItemStackUseCount -= 1;
        } else {
            self.resetActiveHand();
        }
    }

    fn updateItemUseSound(&mut self) {
        use crate::net::minecraft::item::EnumAction::EnumAction;
        if self.activeItemStack.isEmpty() || !self.handActive {
            return;
        }
        match self.activeItemStack.getItemUseAction() {
            EnumAction::Drink => {
                let pitch = self.soundRandomizer.next_f32() * 0.1 + 0.9;
                self.queueSoundAtPlayer("entity.generic.drink", SoundCategory::Players, 0.5, pitch);
            }
            EnumAction::Eat => {
                let volume = 0.5 + 0.5 * self.soundRandomizer.next_i32_bound(2) as f32;
                let pitch =
                    (self.soundRandomizer.next_f32() - self.soundRandomizer.next_f32()) * 0.2 + 1.0;
                self.queueSoundAtPlayer(
                    "entity.generic.eat",
                    SoundCategory::Players,
                    volume,
                    pitch,
                );
            }
            _ => {}
        }
    }

    /// Client response to EntityPlayerMP status opcode 9. It reproduces the
    /// final `EntityLivingBase#updateItemUse` feedback while
    /// leaving inventory contents to SetSlot/WindowItems.
    fn onItemUseFinishClient(&mut self) {
        if self.activeItemStack.isEmpty() || !self.handActive {
            return;
        }
        self.updateItemUseSound();
        // ItemFood's burp originates from the authoritative server world and
        // is delivered as SPacketSoundEffect. Replaying it here would duplicate
        // the sound on a normal 1.12.2 server.
        self.resetActiveHand();
    }

    pub fn queueSoundEvent(&mut self, event: LocalSoundEvent) {
        self.pendingSoundEvents.push(event);
    }

    pub fn queueSoundAt(
        &mut self,
        sound: impl AsRef<str>,
        category: SoundCategory,
        position: [f32; 3],
        volume: f32,
        pitch: f32,
    ) {
        self.pendingSoundEvents.push(LocalSoundEvent::positioned(
            sound, category, position, volume, pitch,
        ));
    }

    pub fn queueSoundAtPlayer(
        &mut self,
        sound: impl AsRef<str>,
        category: SoundCategory,
        volume: f32,
        pitch: f32,
    ) {
        self.queueSoundAt(
            sound,
            category,
            [
                self.entity.posX as f32,
                self.entity.posY as f32,
                self.entity.posZ as f32,
            ],
            volume,
            pitch,
        );
    }

    pub fn takeSoundEvents(&mut self) -> Vec<LocalSoundEvent> {
        std::mem::take(&mut self.pendingSoundEvents)
    }

    pub fn stopActiveHand(&mut self) {
        self.resetActiveHand();
    }
    pub fn resetActiveHand(&mut self) {
        self.handActive = false;
        self.activeItemStack = ItemStack::EMPTY;
        self.activeItemStackUseCount = 0;
    }
    pub const fn isHandActive(&self) -> bool {
        self.handActive
    }
    pub const fn getActiveHand(&self) -> EnumHand {
        self.activeHand
    }
    pub fn getActiveItemStack(&self) -> &ItemStack {
        &self.activeItemStack
    }
    pub const fn getItemInUseCount(&self) -> i32 {
        self.activeItemStackUseCount
    }

    pub const fn getScore(&self) -> i32 {
        self.score
    }

    pub fn addScore(&mut self, scoreIn: i32) {
        self.score = self.score.saturating_add(scoreIn);
    }

    /// MCP `EntityPlayerSP.setXPStats`.
    pub fn setXPStats(&mut self, experienceIn: f32, totalExperienceIn: i32, levelIn: i32) {
        self.experience = experienceIn;
        self.experienceTotal = totalExperienceIn;
        self.experienceLevel = levelIn;
    }

    /// MCP `EntityPlayer.xpBarCap`.
    pub const fn xpBarCap(&self) -> i32 {
        if self.experienceLevel >= 30 {
            112_i32.wrapping_add(self.experienceLevel.wrapping_sub(30).wrapping_mul(9))
        } else if self.experienceLevel >= 15 {
            37_i32.wrapping_add(self.experienceLevel.wrapping_sub(15).wrapping_mul(5))
        } else {
            7_i32.wrapping_add(self.experienceLevel.wrapping_mul(2))
        }
    }

    pub const fn getHealth(&self) -> f32 {
        self.health
    }
    pub const fn getLastDamage(&self) -> f32 {
        self.lastDamage
    }
    pub fn getFoodStats(&self) -> &FoodStats {
        &self.foodStats
    }
    pub fn getFoodStatsMut(&mut self) -> &mut FoodStats {
        &mut self.foodStats
    }
    pub fn getCooldownTracker(&self) -> &CooldownTracker {
        &self.cooldownTracker
    }
    pub fn getCooldownTrackerMut(&mut self) -> &mut CooldownTracker {
        &mut self.cooldownTracker
    }

    pub fn setPositionAndRotation(&mut self, x: f64, y: f64, z: f64, yaw: f32, pitch: f32) {
        self.entity.setPositionAndRotation(x, y, z, yaw, pitch);
    }

    pub fn setPreviousPositionToCurrent(&mut self) {
        self.entity.prevPosX = self.entity.posX;
        self.entity.prevPosY = self.entity.posY;
        self.entity.prevPosZ = self.entity.posZ;
    }

    pub fn turn(&mut self, yaw: f32, pitch: f32) {
        self.entity.turn(yaw, pitch);
    }

    /// MCP `Entity.getPositionEyes`.
    pub fn getPositionEyes(&self, partialTicks: f32) -> Vec3d {
        if partialTicks == 1.0 {
            Vec3d::new(
                self.entity.posX,
                self.entity.posY + self.getEyeHeight() as f64,
                self.entity.posZ,
            )
        } else {
            Vec3d::new(
                self.entity.prevPosX
                    + (self.entity.posX - self.entity.prevPosX) * partialTicks as f64,
                self.entity.prevPosY
                    + (self.entity.posY - self.entity.prevPosY) * partialTicks as f64
                    + self.getEyeHeight() as f64,
                self.entity.prevPosZ
                    + (self.entity.posZ - self.entity.prevPosZ) * partialTicks as f64,
            )
        }
    }

    /// MCP `Entity.getLook` / `getVectorForRotation`.
    pub fn getLook(&self, partialTicks: f32) -> Vec3d {
        let (pitch, yaw) = if partialTicks == 1.0 {
            (self.entity.rotationPitch, self.entity.rotationYaw)
        } else {
            (
                self.entity.prevRotationPitch
                    + (self.entity.rotationPitch - self.entity.prevRotationPitch) * partialTicks,
                self.entity.prevRotationYaw
                    + (self.entity.rotationYaw - self.entity.prevRotationYaw) * partialTicks,
            )
        };
        let f = minecraft_cos(-yaw * 0.017453292 - core::f32::consts::PI);
        let f1 = minecraft_sin(-yaw * 0.017453292 - core::f32::consts::PI);
        let f2 = -minecraft_cos(-pitch * 0.017453292);
        let f3 = minecraft_sin(-pitch * 0.017453292);
        Vec3d::new((f1 * f2) as f64, f3 as f64, (f * f2) as f64)
    }

    pub fn rayTrace(
        &self,
        world: &WorldClient,
        blockReachDistance: f64,
        partialTicks: f32,
    ) -> Option<crate::net::minecraft::util::math::RayTraceResult::RayTraceResult> {
        let eyes = self.getPositionEyes(partialTicks);
        let look = self.getLook(partialTicks);
        world.rayTraceBlocks(
            eyes,
            eyes.add_vector(
                look.x * blockReachDistance,
                look.y * blockReachDistance,
                look.z * blockReachDistance,
            ),
            false,
            false,
            true,
        )
    }

    pub fn getEyeHeight(&self) -> f32 {
        if self.sleeping {
            0.2
        } else if self.entity.sneaking || self.entity.height == 1.65 {
            1.54
        } else if self.isElytraFlying() || self.entity.height == 0.6 {
            0.4
        } else {
            1.62
        }
    }
}

fn normalize_previous_angle(current: f32, previous: &mut f32) {
    while current - *previous < -180.0 {
        *previous -= 360.0;
    }
    while current - *previous >= 180.0 {
        *previous += 360.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_entity_status_queues_vanilla_player_feedback_sounds() {
        let mut player = EntityPlayerSP::new(7);
        player.soundRandomizer = JavaRandom::new(1);
        player.handleStatusUpdate(36);
        let sounds = player.takeSoundEvents();
        assert_eq!(sounds.len(), 1);
        assert_eq!(
            sounds[0].sound.to_string(),
            "minecraft:entity.player.hurt_drown"
        );
        assert_eq!(sounds[0].category, SoundCategory::Players);

        player.activateTotem(0.0, 0.0);
        assert!(player.takeSoundEvents().is_empty());
    }

    #[test]
    fn client_item_use_cadence_and_status_nine_emit_final_eat_without_server_burp_duplicate() {
        let mut player = EntityPlayerSP::new(7);
        player.soundRandomizer = JavaRandom::new(2);
        player.inventory.currentItem = 0;
        player.inventory.mainInventory[0] = ItemStack {
            itemId: 260,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        };
        assert!(player.setActiveHand(EnumHand::MainHand));
        player.activeItemStackUseCount = 24;
        player.updateActiveHand();
        let cadence = player.takeSoundEvents();
        assert_eq!(cadence.len(), 1);
        assert_eq!(cadence[0].sound.to_string(), "minecraft:entity.generic.eat");

        player.handleStatusUpdate(9);
        let finish = player.takeSoundEvents();
        assert_eq!(finish.len(), 1);
        assert_eq!(finish[0].sound.to_string(), "minecraft:entity.generic.eat");
        assert!(!player.isHandActive());
    }

    #[test]
    fn placement_consumption_keeps_inventory_and_container_in_sync() {
        let mut player = EntityPlayerSP::new(7);
        player.inventory.currentItem = 2;
        player.inventory.mainInventory[2] = ItemStack {
            itemId: 50,
            count: 2,
            itemDamage: 0,
            tagCompound: None,
        };
        player
            .inventoryContainer
            .putStackInSlot(38, player.inventory.mainInventory[2].clone())
            .unwrap();

        assert!(player.consumeHeldItemForPlacement(EnumHand::MainHand, 50, 0));
        assert_eq!(player.inventory.mainInventory[2].count, 1);
        assert_eq!(player.inventoryContainer.getSlot(38).unwrap().count, 1);
        assert!(!player.consumeHeldItemForPlacement(EnumHand::MainHand, 65, 0));
        assert_eq!(player.inventory.mainInventory[2].count, 1);
    }

    #[test]
    fn xp_bar_cap_matches_vanilla_piecewise_formula() {
        let mut player = EntityPlayerSP::new(7);
        player.experienceLevel = 0;
        assert_eq!(player.xpBarCap(), 7);
        player.experienceLevel = 14;
        assert_eq!(player.xpBarCap(), 35);
        player.experienceLevel = 15;
        assert_eq!(player.xpBarCap(), 37);
        player.experienceLevel = 30;
        assert_eq!(player.xpBarCap(), 112);
        player.experienceLevel = 31;
        assert_eq!(player.xpBarCap(), 121);
    }

    #[test]
    fn creative_double_tap_jump_toggles_flight_and_sends_abilities() {
        let mut world = WorldClient::new(0);
        let mut player = EntityPlayerSP::new(7);
        GameType::Creative.configurePlayerCapabilities(&mut player.capabilities);

        let first = player.onLivingUpdate(
            &mut world,
            MovementKeyState {
                jump: true,
                ..MovementKeyState::default()
            },
            GameType::Creative,
        );
        assert!(first.iter().all(|packet| packet.id != 0x13));
        assert!(!player.capabilities.isFlying);

        let _ = player.onLivingUpdate(&mut world, MovementKeyState::default(), GameType::Creative);
        let second = player.onLivingUpdate(
            &mut world,
            MovementKeyState {
                jump: true,
                ..MovementKeyState::default()
            },
            GameType::Creative,
        );
        assert!(player.capabilities.isFlying);
        assert!(second.iter().any(|packet| packet.id == 0x13));
    }

    #[test]
    fn survival_jump_transition_requests_fall_flying_with_usable_chest_elytra() {
        let mut world = WorldClient::new(0);
        let mut player = EntityPlayerSP::new(7);
        player.entity.onGround = false;
        player.entity.motionY = -0.2;
        player.inventory.armorInventory[2] = ItemStack {
            itemId: 443,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        };

        let packets = player.onLivingUpdate(
            &mut world,
            MovementKeyState {
                jump: true,
                ..MovementKeyState::default()
            },
            GameType::Survival,
        );
        let fall_flying = packets
            .iter()
            .find(|packet| packet.id == 0x15)
            .expect("START_FALL_FLYING packet");
        assert_eq!(fall_flying.payload, vec![7, 8, 0]);

        player.inventory.armorInventory[2].itemDamage = 431;
        let _ = player.onLivingUpdate(&mut world, MovementKeyState::default(), GameType::Survival);
        player.entity.motionY = -0.2;
        let broken = player.onLivingUpdate(
            &mut world,
            MovementKeyState {
                jump: true,
                ..MovementKeyState::default()
            },
            GameType::Survival,
        );
        assert!(broken.iter().all(|packet| packet.id != 0x15));
    }

    #[test]
    fn horse_jump_charge_releases_vanilla_action_and_local_mount_power() {
        use crate::net::minecraft::client::entity::EntityOtherClient::{
            ClientEntityKind, EntityOtherClient, MobEntityType,
        };
        use crate::net::minecraft::network::datasync::DataSerializers::DataValue;

        let mut world = WorldClient::new(0);
        let mut horse = EntityOtherClient::new(
            100,
            None,
            ClientEntityKind::Mob {
                entityType: MobEntityType::fromId(100).unwrap(),
            },
            0.0,
            64.0,
            0.0,
            0.0,
            0.0,
        );
        horse.applyMetadata([(13, DataValue::Byte(4))]);
        horse.entity.setPassengers(vec![7]);
        world.addNonPlayerEntityToWorld(100, horse);

        let mut player = EntityPlayerSP::new(7);
        player.entity.ridingEntityId = Some(100);
        let jump = MovementKeyState {
            jump: true,
            ..MovementKeyState::default()
        };
        let _ = player.onLivingUpdate(&mut world, jump, GameType::Survival);
        for _ in 0..5 {
            let _ = player.onLivingUpdate(&mut world, jump, GameType::Survival);
        }
        assert!((player.getHorseJumpPower() - 0.5).abs() < 1.0e-6);

        let packets =
            player.onLivingUpdate(&mut world, MovementKeyState::default(), GameType::Survival);
        let jump_packet = packets.iter().find(|packet| packet.id == 0x15).unwrap();
        assert_eq!(jump_packet.payload, vec![7, 5, 50]);
        assert_eq!(player.horseJumpPowerCounter, -10);
        let horse = world.getNonPlayerEntityByID(100).unwrap();
        assert!((horse.horseJumpPower - (0.4 + 0.4 * 50.0 / 90.0)).abs() < 1.0e-6);
    }

    #[test]
    fn local_metadata_updates_inherited_entity_flags_and_hand_state() {
        let mut player = EntityPlayerSP::new(7);
        player.inventory.currentItem = 0;
        player.inventory.mainInventory[0] = ItemStack {
            itemId: 261,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        };

        player.applyMetadata([(0, DataValue::Byte(0x22)), (6, DataValue::Byte(0x01))]);
        assert!(player.entity.sneaking);
        assert!(player.isInvisible());
        assert!(player.isHandActive());
        assert_eq!(player.getActiveHand(), EnumHand::MainHand);
        assert_eq!(player.getActiveItemStack().itemId, 261);

        player.applyMetadata([(0, DataValue::Byte(0)), (6, DataValue::Byte(0))]);
        assert!(!player.entity.sneaking);
        assert!(!player.isInvisible());
        assert!(!player.isHandActive());
    }

    #[test]
    fn sprinting_uses_vanilla_attribute_modifier_and_entity_flag() {
        let mut player = EntityPlayerSP::new(7);
        player.setSprinting(true);
        let speed = player
            .attributeMap
            .getAttributeValue("generic.movementSpeed", 0.0);
        assert!((speed - 0.1300000031292439).abs() < 1.0e-12);
        assert_ne!(player.dataManager.byte(0, 0) & 0x08, 0);

        player.setSprinting(false);
        let speed = player
            .attributeMap
            .getAttributeValue("generic.movementSpeed", 0.0);
        assert!((speed - 0.10000000149011612).abs() < 1.0e-12);
        assert_eq!(player.dataManager.byte(0, 0) & 0x08, 0);
    }

    #[test]
    fn speed_and_slowness_use_vanilla_attribute_uuids_and_amplifier_scaling() {
        let mut player = EntityPlayerSP::new(7);
        player.addPotionEffect(PotionEffect::new(1, 200, 1, false, true));
        let speed = player
            .attributeMap
            .getAttributeValue("generic.movementSpeed", 0.0);
        assert!((speed - 0.1400000026822091).abs() < 1.0e-12);

        player.addPotionEffect(PotionEffect::new(2, 200, 0, false, true));
        let combined = player
            .attributeMap
            .getAttributeValue("generic.movementSpeed", 0.0);
        assert!((combined - 0.11900000144541263).abs() < 1.0e-12);

        player.removeActivePotionEffect(1);
        let slowness_only = player
            .attributeMap
            .getAttributeValue("generic.movementSpeed", 0.0);
        assert!((slowness_only - 0.08500000067055224).abs() < 1.0e-12);
    }

    #[test]
    fn elytra_state_uses_vanilla_collision_height_and_eye_height() {
        let world = WorldClient::new(0);
        let mut player = EntityPlayerSP::new(7);
        player.dataManager.setByte(0, 0x80_u8 as i8);
        player.updateSize(&world);
        assert_eq!(player.entity.width, 0.6);
        assert_eq!(player.entity.height, 0.6);
        assert_eq!(player.getEyeHeight(), 0.4);
    }

    #[test]
    fn elytra_travel_matches_mcp_air_dynamics_for_one_tick() {
        let world = WorldClient::new(0);
        let mut player = EntityPlayerSP::new(7);
        player.entity.setPosition(0.0, 80.0, 0.0);
        player.entity.rotationYaw = 25.0;
        player.entity.rotationPitch = -15.0;
        player.rotationYawHead = 25.0;
        player.entity.motionX = 0.2;
        player.entity.motionY = -0.1;
        player.entity.motionZ = 0.05;

        let look = player.getLook(1.0);
        let pitch = player.entity.rotationPitch * 0.017453292_f32;
        let horizontal_look = (look.x * look.x + look.z * look.z).sqrt();
        let horizontal_speed = (player.entity.motionX * player.entity.motionX
            + player.entity.motionZ * player.entity.motionZ)
            .sqrt();
        let look_length = look.length();
        let mut lift = minecraft_cos(pitch);
        lift = (lift as f64 * lift as f64 * 1.0_f64.min(look_length / 0.4_f64)) as f32;
        let mut expected_x = player.entity.motionX;
        let mut expected_y = player.entity.motionY + -0.08 + lift as f64 * 0.06;
        let mut expected_z = player.entity.motionZ;
        if expected_y < 0.0 && horizontal_look > 0.0 {
            let transfer = expected_y * -0.1 * lift as f64;
            expected_y += transfer;
            expected_x += look.x * transfer / horizontal_look;
            expected_z += look.z * transfer / horizontal_look;
        }
        if pitch < 0.0 {
            let transfer = horizontal_speed * (-minecraft_sin(pitch)) as f64 * 0.04;
            expected_y += transfer * 3.2;
            expected_x -= look.x * transfer / horizontal_look;
            expected_z -= look.z * transfer / horizontal_look;
        }
        if horizontal_look > 0.0 {
            expected_x += (look.x / horizontal_look * horizontal_speed - expected_x) * 0.1;
            expected_z += (look.z / horizontal_look * horizontal_speed - expected_z) * 0.1;
        }
        expected_x *= 0.9900000095367432_f64;
        expected_y *= 0.9800000190734863_f64;
        expected_z *= 0.9900000095367432_f64;

        player.travelElytra(&world);
        assert!((player.entity.motionX - expected_x).abs() < 1.0e-12);
        assert!((player.entity.motionY - expected_y).abs() < 1.0e-12);
        assert!((player.entity.motionZ - expected_z).abs() < 1.0e-12);
        assert!((player.entity.posX - expected_x).abs() < 1.0e-12);
        assert!((player.entity.posY - (80.0 + expected_y)).abs() < 1.0e-12);
        assert!((player.entity.posZ - expected_z).abs() < 1.0e-12);
    }

    #[test]
    fn stationary_player_sends_periodic_position_at_twenty_ticks() {
        let mut player = EntityPlayerSP::new(7);
        for _ in 0..19 {
            assert!(player.onUpdateWalkingPlayer().is_empty());
        }
        let packets = player.onUpdateWalkingPlayer();
        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].id, 0x0D);
    }

    #[test]
    fn sneak_transition_precedes_movement_packet() {
        let mut player = EntityPlayerSP::new(7);
        player.entity.sneaking = true;
        player.entity.setPosition(1.0, 64.0, 1.0);
        let packets = player.onUpdateWalkingPlayer();
        assert_eq!(packets[0].id, 0x15);
        assert_eq!(packets[1].id, 0x0D);
        assert_eq!(player.getEyeHeight(), 1.54);
    }
}
