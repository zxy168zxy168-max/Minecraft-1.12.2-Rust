use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use crate::net::minecraft::entity::Entity::Entity;
use crate::net::minecraft::entity::EntityHanging::EntityHanging;
use crate::net::minecraft::entity::EntityLeashKnot::EntityLeashKnot;
use crate::net::minecraft::entity::item::EntityItemFrame::EntityItemFrame;
use crate::net::minecraft::entity::item::EntityPainting::{EntityPainting, PaintingArt};
use crate::net::minecraft::entity::EntityLivingBase;
use crate::net::minecraft::entity::ai::attributes::AbstractAttributeMap::AbstractAttributeMap;
use crate::net::minecraft::entity::IJumpingMount::IJumpingMount;
use crate::net::minecraft::entity::passive::AbstractHorse::AbstractHorse;
use crate::net::minecraft::entity::projectile::EntityShulkerBullet::EntityShulkerBullet;
use crate::net::minecraft::entity::projectile::EntityFishHook::{EntityFishHook, FishHookState};
use crate::net::minecraft::entity::EntityAreaEffectCloud::EntityAreaEffectCloud;
use crate::net::minecraft::util::EnumParticleTypes::EnumParticleTypes;
use crate::net::minecraft::entity::projectile::EntityFireball::EntityFireball;
use crate::net::minecraft::entity::projectile::EntityLargeFireball::EntityLargeFireball;
use crate::net::minecraft::entity::projectile::EntitySmallFireball::EntitySmallFireball;
use crate::net::minecraft::entity::projectile::EntityDragonFireball::EntityDragonFireball;
use crate::net::minecraft::entity::projectile::EntityWitherSkull::EntityWitherSkull;
use crate::net::minecraft::entity::projectile::ProjectileHelper::ProjectileHelper;
use crate::net::minecraft::entity::item::EntityBoat::{BoatStatus, BoatType, EntityBoat};
use crate::net::minecraft::entity::item::EntityMinecart::{EntityMinecart, MinecartType};
use crate::net::minecraft::entity::item::EntityEnderCrystal::EntityEnderCrystal;
use crate::compat::Java::JavaRandom;
use crate::net::minecraft::block::BlockLiquid;
use crate::net::minecraft::block::BlockLiquid::LiquidMaterial;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP;
use crate::net::minecraft::client::audio::LocalSoundEvent::LocalSoundEvent;
use crate::net::minecraft::util::SoundCategory::SoundCategory;
use crate::net::minecraft::client::particle::ParticleSpawnRequest::ParticleSpawnRequest;
use crate::net::minecraft::inventory::EntityEquipment::EntityEquipment;
use crate::net::minecraft::inventory::EntityEquipmentSlot::EntityEquipmentSlot;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::datasync::DataSerializers::DataValue;
use crate::net::minecraft::network::datasync::EntityDataManager::EntityDataManager;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::EnumHand::EnumHand;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::math::Vec3d::Vec3d;

/// Exact object discriminator values consumed by MCP
/// `NetHandlerPlayClient.handleSpawnObject` in protocol 340.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectSpawnType {
    Boat,
    Item,
    AreaEffectCloud,
    Minecart,
    PrimedTnt,
    EnderCrystal,
    TippedArrow,
    Snowball,
    Egg,
    LargeFireball,
    SmallFireball,
    EnderPearl,
    WitherSkull,
    ShulkerBullet,
    LlamaSpit,
    FallingBlock,
    ItemFrame,
    EyeOfEnder,
    Potion,
    ExperienceBottle,
    FireworkRocket,
    LeashKnot,
    ArmorStand,
    EvokerFangs,
    FishHook,
    SpectralArrow,
    DragonFireball,
    Unknown(i32),
}

impl ObjectSpawnType {
    pub const fn isFireball(self) -> bool {
        matches!(
            self,
            Self::LargeFireball | Self::SmallFireball | Self::WitherSkull | Self::DragonFireball
        )
    }

    pub const fn fromPacketType(value: i32) -> Self {
        match value {
            1 => Self::Boat,
            2 => Self::Item,
            3 => Self::AreaEffectCloud,
            10 => Self::Minecart,
            50 => Self::PrimedTnt,
            51 => Self::EnderCrystal,
            60 => Self::TippedArrow,
            61 => Self::Snowball,
            62 => Self::Egg,
            63 => Self::LargeFireball,
            64 => Self::SmallFireball,
            65 => Self::EnderPearl,
            66 => Self::WitherSkull,
            67 => Self::ShulkerBullet,
            68 => Self::LlamaSpit,
            70 => Self::FallingBlock,
            71 => Self::ItemFrame,
            72 => Self::EyeOfEnder,
            73 => Self::Potion,
            75 => Self::ExperienceBottle,
            76 => Self::FireworkRocket,
            77 => Self::LeashKnot,
            78 => Self::ArmorStand,
            79 => Self::EvokerFangs,
            90 => Self::FishHook,
            91 => Self::SpectralArrow,
            93 => Self::DragonFireball,
            other => Self::Unknown(other),
        }
    }

    pub const fn packetType(self) -> i32 {
        match self {
            Self::Boat => 1,
            Self::Item => 2,
            Self::AreaEffectCloud => 3,
            Self::Minecart => 10,
            Self::PrimedTnt => 50,
            Self::EnderCrystal => 51,
            Self::TippedArrow => 60,
            Self::Snowball => 61,
            Self::Egg => 62,
            Self::LargeFireball => 63,
            Self::SmallFireball => 64,
            Self::EnderPearl => 65,
            Self::WitherSkull => 66,
            Self::ShulkerBullet => 67,
            Self::LlamaSpit => 68,
            Self::FallingBlock => 70,
            Self::ItemFrame => 71,
            Self::EyeOfEnder => 72,
            Self::Potion => 73,
            Self::ExperienceBottle => 75,
            Self::FireworkRocket => 76,
            Self::LeashKnot => 77,
            Self::ArmorStand => 78,
            Self::EvokerFangs => 79,
            Self::FishHook => 90,
            Self::SpectralArrow => 91,
            Self::DragonFireball => 93,
            Self::Unknown(value) => value,
        }
    }

    pub const fn isLivingBase(self) -> bool { matches!(self, Self::ArmorStand) }
}

/// Entry resolved through the numeric `EntityList` registry used by
/// `SPacketSpawnMob`. The registry name is the 1.12.2 ResourceLocation path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MobEntityType {
    pub id: i32,
    pub registryName: &'static str,
}

impl MobEntityType {
    pub const fn fromId(id: i32) -> Option<Self> {
        let registryName = match id {
            4 => "elder_guardian",
            5 => "wither_skeleton",
            6 => "stray",
            23 => "husk",
            27 => "zombie_villager",
            28 => "skeleton_horse",
            29 => "zombie_horse",
            31 => "donkey",
            32 => "mule",
            34 => "evocation_illager",
            35 => "vex",
            36 => "vindication_illager",
            37 => "illusion_illager",
            50 => "creeper",
            51 => "skeleton",
            52 => "spider",
            53 => "giant",
            54 => "zombie",
            55 => "slime",
            56 => "ghast",
            57 => "zombie_pigman",
            58 => "enderman",
            59 => "cave_spider",
            60 => "silverfish",
            61 => "blaze",
            62 => "magma_cube",
            63 => "ender_dragon",
            64 => "wither",
            65 => "bat",
            66 => "witch",
            67 => "endermite",
            68 => "guardian",
            69 => "shulker",
            90 => "pig",
            91 => "sheep",
            92 => "cow",
            93 => "chicken",
            94 => "squid",
            95 => "wolf",
            96 => "mooshroom",
            97 => "snowman",
            98 => "ocelot",
            99 => "villager_golem",
            100 => "horse",
            101 => "rabbit",
            102 => "polar_bear",
            103 => "llama",
            105 => "parrot",
            120 => "villager",
            _ => return None,
        };
        Some(Self { id, registryName })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClientEntityKind {
    Object {
        objectType: ObjectSpawnType,
        data: i32,
        /// Raw protocol velocity/acceleration vector divided by 8000.
        spawnVelocity: [f64; 3],
    },
    Mob { entityType: MobEntityType },
    ExperienceOrb { xpValue: i16 },
    Painting {
        title: String,
        hangingPosition: BlockPos,
        facing: EnumFacing,
    },
}

/// Rust heterogeneous-world equivalent for concrete non-player subclasses that
/// have entered through their exact 1.12.2 spawn packet but whose individual
/// AI/model class has not yet been ported. This is authoritative synchronized
/// state, not a rendered placeholder.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityOtherClient {
    pub entity: Entity,
    pub entityId: i32,
    pub uniqueId: Option<Uuid>,
    pub kind: ClientEntityKind,
    /// MCP `EntityHanging.hangingPosition`, present for paintings, item frames and leash knots.
    pub hangingPosition: Option<BlockPos>,
    /// MCP `EntityHanging.facingDirection`; leash knots deliberately have no facing.
    pub hangingFacing: Option<EnumFacing>,
    /// Resolved motive table entry for `EntityPainting`.
    pub paintingArt: Option<PaintingArt>,
    pub serverPosX: i64,
    pub serverPosY: i64,
    pub serverPosZ: i64,
    pub prevRenderYawOffset: f32,
    pub renderYawOffset: f32,
    pub prevRotationYawHead: f32,
    pub rotationYawHead: f32,
    pub dataManager: EntityDataManager,
    pub attributeMap: AbstractAttributeMap,
    pub equipment: EntityEquipment,
    pub health: f32,
    pub hurtTime: i32,
    pub maxHurtTime: i32,
    pub deathTime: i32,
    pub hurtResistantTime: i32,
    pub maxHurtResistantTime: i32,
    pub attackedAtYaw: f32,
    pub limbSwing: f32,
    pub prevLimbSwingAmount: f32,
    pub limbSwingAmount: f32,
    pub prevSwingProgress: f32,
    pub swingProgress: f32,
    swingProgressInt: i32,
    isSwingInProgress: bool,
    pub swingingHand: EnumHand,
    /// Client-only EntityItem animation phase created by its constructor.
    pub hoverStart: f32,
    /// EntityXPOrb client animation counter.
    pub xpColor: i32,
    /// EntityArrow impact shake counter.
    pub arrowShake: i32,
    /// EntityTNTPrimed synchronized fuse, decremented by client onUpdate.
    pub tntFuse: i32,
    /// `EntityEnderCrystal.innerRotation`, initialized from the entity RNG
    /// and incremented once per client tick.
    pub enderCrystalInnerRotation: i32,
    /// Projectile block-impact state owned by EntityArrow on the client.
    pub inGround: bool,
    pub ticksInGround: i32,
    pub xpOrbAge: i32,
    pub lastStatusOpcode: Option<i8>,
    /// Client tick when EntityArmorStand status opcode 32 last arrived.
    pub armorStandPunchTick: Option<i32>,
    /// MCP `EntitySheep.sheepTimer`, driven by status opcode 10.
    pub sheepTimer: i32,
    /// MCP `EntityChicken` client flap state.
    pub wingRotation: f32,
    pub destPos: f32,
    pub oFlapSpeed: f32,
    pub oFlap: f32,
    pub wingRotDelta: f32,
    /// EntityCreeper client ignition interpolation fields.
    pub lastActiveTime: i32,
    pub timeSinceIgnited: i32,
    /// EntitySlime / EntityMagmaCube client squish animation fields.
    pub squishAmount: f32,
    pub squishFactor: f32,
    pub prevSquishFactor: f32,
    pub wasOnGround: bool,
    /// `EntityGuardian` client-only tail, spike and beam charge state.
    pub guardianTailAnimation: f32,
    pub guardianTailAnimationO: f32,
    pub guardianTailAnimationSpeed: f32,
    pub guardianSpikesAnimation: f32,
    pub guardianSpikesAnimationO: f32,
    pub guardianAttackTime: i32,
    pub guardianTouchedGround: bool,
    guardianRandom: JavaRandom,
    /// MCP `EntitySquid` client animation state. Motion remains server-authoritative;
    /// these fields reproduce the local pitch/yaw/tentacle interpolation.
    pub squidPitch: f32,
    pub squidPrevPitch: f32,
    pub squidYaw: f32,
    pub squidPrevYaw: f32,
    pub squidRotation: f32,
    pub squidPrevRotation: f32,
    pub squidTentacleAngle: f32,
    pub squidLastTentacleAngle: f32,
    squidRandomMotionSpeed: f32,
    squidRotationVelocity: f32,
    squidRotateSpeed: f32,
    /// MCP `EntityDragon` client-side animation history used by ModelDragon.
    pub dragonRingBuffer: [[f64; 3]; 64],
    pub dragonRingBufferIndex: i32,
    pub dragonPrevAnimTime: f32,
    pub dragonAnimTime: f32,
    pub dragonSlowed: bool,
    pub dragonDeathTicks: i32,
    /// `EntityShulker` client-only peek and attachment interpolation state.
    pub shulkerPrevPeekAmount: f32,
    pub shulkerPeekAmount: f32,
    pub shulkerCurrentAttachmentPosition: Option<BlockPos>,
    pub shulkerClientSideTeleportInterpolation: i32,
    /// `EntityFireball` normalized acceleration retained independently from motion.
    pub fireballAccelerationX: f64,
    pub fireballAccelerationY: f64,
    pub fireballAccelerationZ: f64,
    /// MCP `EntityFishHook` owner/state. Spawn-object `data` is the angler ID;
    /// metadata index 6 is caughtEntityId + 1.
    pub fishHookAnglerId: Option<i32>,
    pub fishHookCaughtEntityId: Option<i32>,
    pub fishHookState: FishHookState,
    pub fishHookInGround: bool,
    pub fishHookTicksInGround: i32,
    pub fishHookTicksInAir: i32,
    fishHookRandom: JavaRandom,
    pub pendingParticleSpawns: Vec<ParticleSpawnRequest>,
    pendingSoundEvents: Vec<LocalSoundEvent>,
    fireworkLaunchSoundPlayed: bool,
    areaEffectCloudRandom: JavaRandom,
    /// `EntityWolf` interested-head and wet-shake client animation state.
    pub wolfHeadRotationCourse: f32,
    pub wolfHeadRotationCourseOld: f32,
    pub wolfIsWet: bool,
    pub wolfIsShaking: bool,
    pub timeWolfIsShaking: f32,
    pub prevTimeWolfIsShaking: f32,
    /// `EntityRabbit` status-driven ten-tick jump animation.
    pub rabbitJumpTicks: i32,
    pub rabbitJumpDuration: i32,
    /// `EntityPolarBear` client standing interpolation.
    pub polarStandAnimation0: f32,
    pub polarStandAnimation: f32,
    /// `AbstractHorse` client-only pose interpolation and tail animation.
    pub horseHeadLean: f32,
    pub horsePrevHeadLean: f32,
    pub horseRearingAmount: f32,
    pub horsePrevRearingAmount: f32,
    pub horseMouthOpenness: f32,
    pub horsePrevMouthOpenness: f32,
    pub horseTailCounter: i32,
    /// `AbstractHorse` locally controlled travel state.
    pub horseJumpPower: f32,
    pub horseAllowStandSliding: bool,
    pub horseJumping: bool,
    pub horseJumpRearingCounter: i32,
    pub horseMoveStrafing: f32,
    pub horseMoveForward: f32,
    pub horseRiderYaw: f32,
    pub horseRiderPitch: f32,
    horseRandom: JavaRandom,
    /// `EntityIllusionIllager` client-only four-copy transition state.
    pub illusionTransitionTicks: i32,
    pub illusionOffsetsOld: [[f64; 3]; 4],
    pub illusionOffsetsNew: [[f64; 3]; 4],
    illusionRandom: JavaRandom,
    /// MCP `EntityBoat.paddlePositions` and complete client-control state.
    pub boatPaddlePositions: [f32; 2],
    pub boatMomentum: f32,
    pub boatOutOfControlTicks: f32,
    pub boatDeltaRotation: f32,
    pub boatLeftInputDown: bool,
    pub boatRightInputDown: bool,
    pub boatForwardInputDown: bool,
    pub boatBackInputDown: bool,
    pub boatWaterLevel: f64,
    pub boatGlide: f32,
    pub boatStatus: Option<BoatStatus>,
    pub boatPreviousStatus: Option<BoatStatus>,
    pub boatLastYd: f64,
    /// MCP `EntityMinecart.velocityX/Y/Z`, restored when a direct-position packet starts.
    pub minecartVelocityX: f64,
    pub minecartVelocityY: f64,
    pub minecartVelocityZ: f64,
    /// MCP `EntityMinecartTNT.minecartTNTFuse`; status opcode 10 starts it at 80.
    pub minecartTntFuse: i32,
    /// `EntityLiving` leash holder ID updated by `SPacketEntityAttach`.
    pub leashHolderId: Option<i32>,
    interpTargetX: f64,
    interpTargetY: f64,
    interpTargetZ: f64,
    interpTargetYaw: f64,
    interpTargetPitch: f64,
    newPosRotationIncrements: i32,
}

impl EntityOtherClient {
    pub fn new(
        entityId: i32,
        uniqueId: Option<Uuid>,
        kind: ClientEntityKind,
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
    ) -> Self {
        let mut entity = Entity::default();
        let (width, height) = entity_size(&kind);
        entity.width = width;
        entity.height = height;
        entity.setPositionAndRotation(x, y, z, yaw, pitch);
        let mut hangingPosition = None;
        let mut hangingFacing = None;
        let mut paintingArt = None;
        match &kind {
            ClientEntityKind::Painting { title, hangingPosition: position, facing } => {
                let art = EntityPainting::art(title);
                let dimensions = art.data();
                EntityHanging::updateFacingWithBoundingBox(
                    &mut entity,
                    *position,
                    *facing,
                    dimensions.sizeX,
                    dimensions.sizeY,
                );
                hangingPosition = Some(*position);
                hangingFacing = Some(*facing);
                paintingArt = Some(art);
            }
            ClientEntityKind::Object { objectType: ObjectSpawnType::ItemFrame, data, .. } => {
                let position = BlockPos::new(x.floor() as i32, y.floor() as i32, z.floor() as i32);
                let facing = EnumFacing::getHorizontal(*data);
                EntityHanging::updateFacingWithBoundingBox(
                    &mut entity,
                    position,
                    facing,
                    EntityItemFrame::WIDTH_PIXELS,
                    EntityItemFrame::HEIGHT_PIXELS,
                );
                hangingPosition = Some(position);
                hangingFacing = Some(facing);
            }
            ClientEntityKind::Object { objectType: ObjectSpawnType::LeashKnot, .. } => {
                let position = BlockPos::new(x.floor() as i32, y.floor() as i32, z.floor() as i32);
                EntityLeashKnot::setHangingPosition(&mut entity, position);
                hangingPosition = Some(position);
            }
            _ => {}
        }
        let isShulker = matches!(
            &kind,
            ClientEntityKind::Mob { entityType } if entityType.registryName == "shulker"
        );
        if isShulker {
            entity.prevRotationYaw = 180.0;
            entity.rotationYaw = 180.0;
        }
        if matches!(&kind, ClientEntityKind::Object { objectType: ObjectSpawnType::ShulkerBullet | ObjectSpawnType::AreaEffectCloud, .. })
            || matches!(&kind, ClientEntityKind::Mob { entityType } if entityType.registryName == "ender_dragon")
        {
            // EntityDragon constructor sets noClip=true; ShulkerBullet and
            // AreaEffectCloud likewise own no-clip movement in vanilla.
            entity.noClip = true;
        }
        let fireballAcceleration = match &kind {
            ClientEntityKind::Object { objectType, spawnVelocity, .. }
                if objectType.isFireball() =>
            {
                EntityFireball::normalizedAcceleration(*spawnVelocity)
            }
            _ => [0.0; 3],
        };
        let mut dataManager = EntityDataManager::default();
        match &kind {
            ClientEntityKind::Object { objectType: ObjectSpawnType::Boat, .. } => {
                // EntityBoat#entityInit metadata keys 6..11.
                dataManager.setVarInt(6, 0);
                dataManager.setVarInt(7, 1);
                dataManager.setFloat(8, 0.0);
                dataManager.setVarInt(9, 0);
                dataManager.setBoolean(10, false);
                dataManager.setBoolean(11, false);
            }
            ClientEntityKind::Object { objectType: ObjectSpawnType::ItemFrame, .. } => {
                // EntityItemFrame#entityInit: ITEM key 6 and ROTATION key 7.
                dataManager.setEntryValues([(EntityItemFrame::ITEM_DATA_INDEX, DataValue::ItemStack(ItemStack::EMPTY))]);
                dataManager.setVarInt(EntityItemFrame::ROTATION_DATA_INDEX, 0);
            }
            ClientEntityKind::Object { objectType: ObjectSpawnType::Minecart, data, .. } => {
                // EntityMinecart#entityInit metadata keys 6..11; furnace POWERED is key 12.
                dataManager.setVarInt(6, 0);
                dataManager.setVarInt(7, 1);
                dataManager.setFloat(8, 0.0);
                dataManager.setVarInt(9, 0);
                dataManager.setVarInt(10, 6);
                dataManager.setBoolean(11, false);
                if MinecartType::byId(*data) == MinecartType::Furnace {
                    dataManager.setBoolean(12, false);
                }
            }
            ClientEntityKind::Object { objectType: ObjectSpawnType::FishHook, .. } => {
                dataManager.setVarInt(EntityFishHook::DATA_HOOKED_ENTITY_INDEX, 0);
            }
            ClientEntityKind::Object { objectType: ObjectSpawnType::AreaEffectCloud, .. } => {
                dataManager.setVarInt(EntityAreaEffectCloud::COLOR_INDEX, EntityAreaEffectCloud::DEFAULT_COLOR);
                dataManager.setFloat(EntityAreaEffectCloud::RADIUS_INDEX, EntityAreaEffectCloud::DEFAULT_SYNC_RADIUS);
                dataManager.setBoolean(EntityAreaEffectCloud::IGNORE_RADIUS_INDEX, false);
                dataManager.setVarInt(EntityAreaEffectCloud::PARTICLE_INDEX, EntityAreaEffectCloud::DEFAULT_PARTICLE.particleId());
                dataManager.setVarInt(EntityAreaEffectCloud::PARTICLE_PARAM_1_INDEX, 0);
                dataManager.setVarInt(EntityAreaEffectCloud::PARTICLE_PARAM_2_INDEX, 0);
            }
            ClientEntityKind::Object { objectType: ObjectSpawnType::WitherSkull, .. } => {
                dataManager.setBoolean(EntityWitherSkull::INVULNERABLE_DATA_INDEX, false);
            }
            ClientEntityKind::Object { objectType: ObjectSpawnType::EnderCrystal, .. } => {
                // EntityEnderCrystal#entityInit: optional beam target is absent
                // until metadata arrives; SHOW_BOTTOM defaults to true.
                dataManager.setBoolean(EntityEnderCrystal::SHOW_BOTTOM_DATA_INDEX, true);
            }
            ClientEntityKind::Mob { entityType } if entityType.registryName == "enderman" => {
                // EntityEnderman entityInit: CARRIED_BLOCK index 12, SCREAMING index 13.
                dataManager.setEntryValues([(12, DataValue::OptionalBlockState(None))]);
                dataManager.setBoolean(13, false);
            }
            ClientEntityKind::Mob { entityType } if entityType.registryName == "ender_dragon" => {
                // EntityDragon#entityInit. PhaseList.HOVER is id 10.
                dataManager.setVarInt(12, 10);
            }
            _ => {}
        }
        let hoverStart = hover_start(entityId, uniqueId);
        let mut guardianRandom = fresh_random(entityId ^ 0x4755_4152);
        let guardianTailAnimation = if matches!(
            &kind,
            ClientEntityKind::Mob { entityType }
                if matches!(entityType.registryName, "guardian" | "elder_guardian")
        ) {
            guardianRandom.next_f32()
        } else {
            0.0
        };
        match &kind {
            ClientEntityKind::ExperienceOrb { .. } => {
                let mut random = fresh_random(entityId);
                entity.rotationYaw = random.next_f32() * 360.0;
                entity.motionX = (random.next_f32() as f64 * 0.20000000298023224 - 0.10000000149011612) * 2.0;
                entity.motionY = random.next_f32() as f64 * 0.2 * 2.0;
                entity.motionZ = (random.next_f32() as f64 * 0.20000000298023224 - 0.10000000149011612) * 2.0;
            }
            ClientEntityKind::Object { objectType: ObjectSpawnType::Item, .. } => {
                // EntityItem's constructor seeds motion before the spawn packet's
                // data>0 velocity assignment overwrites it for tracked drops.
                let mut random = fresh_random(entityId);
                entity.rotationYaw = random.next_f32() * 360.0;
                entity.motionX = random.next_f32() as f64 * 0.20000000298023224 - 0.10000000149011612;
                entity.motionY = 0.20000000298023224;
                entity.motionZ = random.next_f32() as f64 * 0.20000000298023224 - 0.10000000149011612;
            }
            ClientEntityKind::Object { objectType: ObjectSpawnType::PrimedTnt, .. } => {
                let mut random = fresh_random(entityId);
                let angle = random.next_f32() * std::f32::consts::TAU;
                entity.motionX = -(angle.sin() as f64) * 0.02;
                entity.motionY = 0.20000000298023224;
                entity.motionZ = -(angle.cos() as f64) * 0.02;
            }
            ClientEntityKind::Object { objectType: ObjectSpawnType::FireworkRocket, .. } => {
                let mut random = fresh_random(entityId);
                let (gaussianX, gaussianZ) = next_gaussian_pair(&mut random);
                entity.motionX = gaussianX * 0.001;
                entity.motionY = 0.05;
                entity.motionZ = gaussianZ * 0.001;
            }
            _ => {}
        }
        let mut attributeMap = AbstractAttributeMap::default();
        if matches!(
            &kind,
            ClientEntityKind::Mob { entityType }
                if matches!(entityType.registryName, "horse" | "donkey" | "mule" | "skeleton_horse" | "zombie_horse" | "llama")
        ) {
            // AbstractHorse#applyEntityAttributes. Per-entity randomized values
            // arrive later through SPacketEntityProperties.
            attributeMap.registerAttribute("generic.maxHealth", AbstractHorse::DEFAULT_MAX_HEALTH);
            attributeMap.registerAttribute("generic.movementSpeed", AbstractHorse::DEFAULT_MOVEMENT_SPEED);
            attributeMap.registerAttribute("horse.jumpStrength", AbstractHorse::DEFAULT_JUMP_STRENGTH);
        }
        let fishHookAnglerId = match &kind {
            ClientEntityKind::Object { objectType: ObjectSpawnType::FishHook, data, .. } => Some(*data),
            _ => None,
        };
        let enderCrystalInnerRotation = if matches!(
            &kind,
            ClientEntityKind::Object { objectType: ObjectSpawnType::EnderCrystal, .. }
        ) {
            fresh_random(entityId ^ 0x454e_4443).next_i32_bound(100_000)
        } else {
            0
        };
        // EntitySquid ctor: rotationVelocity = 1 / (rand.nextFloat()+1) * 0.2.
        // Vanilla reseeds Entity.rand with 1 + its constructor-time entity id; the
        // client packet later replaces that id. `fresh_random` preserves the same
        // per-instance random distribution without inventing a fixed animation.
        let squidRotationVelocity = if matches!(
            &kind, ClientEntityKind::Mob { entityType } if entityType.registryName == "squid"
        ) {
            let mut random = fresh_random(entityId ^ 0x5351_5549);
            1.0 / (random.next_f32() + 1.0) * 0.2
        } else { 0.0 };
        let initialHealth = match &kind {
            ClientEntityKind::Mob { entityType } if entityType.registryName == "enderman" => 40.0,
            ClientEntityKind::Mob { entityType } if entityType.registryName == "squid" => 10.0,
            ClientEntityKind::Mob { entityType } if entityType.registryName == "ender_dragon" => 200.0,
            _ => 20.0,
        };
        Self {
            entity,
            entityId,
            uniqueId,
            kind,
            hangingPosition,
            hangingFacing,
            paintingArt,
            serverPosX: fixed_position(x),
            serverPosY: fixed_position(y),
            serverPosZ: fixed_position(z),
            prevRenderYawOffset: if isShulker { 180.0 } else { yaw },
            renderYawOffset: if isShulker { 180.0 } else { yaw },
            prevRotationYawHead: yaw,
            rotationYawHead: yaw,
            dataManager,
            attributeMap,
            equipment: EntityEquipment::default(),
            health: initialHealth,
            hurtTime: 0,
            maxHurtTime: 0,
            deathTime: 0,
            hurtResistantTime: 0,
            maxHurtResistantTime: 20,
            attackedAtYaw: 0.0,
            limbSwing: 0.0,
            prevLimbSwingAmount: 0.0,
            limbSwingAmount: 0.0,
            prevSwingProgress: 0.0,
            swingProgress: 0.0,
            swingProgressInt: 0,
            isSwingInProgress: false,
            swingingHand: EnumHand::MainHand,
            hoverStart,
            xpColor: 0,
            arrowShake: 0,
            tntFuse: 80,
            enderCrystalInnerRotation,
            inGround: false,
            ticksInGround: 0,
            xpOrbAge: 0,
            lastStatusOpcode: None,
            armorStandPunchTick: None,
            sheepTimer: 0,
            wingRotation: 0.0,
            destPos: 0.0,
            oFlapSpeed: 0.0,
            oFlap: 0.0,
            wingRotDelta: 1.0,
            lastActiveTime: 0,
            timeSinceIgnited: 0,
            squishAmount: 0.0,
            squishFactor: 0.0,
            prevSquishFactor: 0.0,
            wasOnGround: false,
            guardianTailAnimation,
            guardianTailAnimationO: guardianTailAnimation,
            guardianTailAnimationSpeed: 0.0,
            guardianSpikesAnimation: 0.0,
            guardianSpikesAnimationO: 0.0,
            guardianAttackTime: 0,
            guardianTouchedGround: false,
            guardianRandom,
            squidPitch: 0.0,
            squidPrevPitch: 0.0,
            squidYaw: 0.0,
            squidPrevYaw: 0.0,
            squidRotation: 0.0,
            squidPrevRotation: 0.0,
            squidTentacleAngle: 0.0,
            squidLastTentacleAngle: 0.0,
            squidRandomMotionSpeed: 0.0,
            squidRotationVelocity,
            squidRotateSpeed: 0.0,
            dragonRingBuffer: [[0.0; 3]; 64],
            dragonRingBufferIndex: -1,
            dragonPrevAnimTime: 0.0,
            dragonAnimTime: 0.0,
            dragonSlowed: false,
            dragonDeathTicks: 0,
            shulkerPrevPeekAmount: 0.0,
            shulkerPeekAmount: 0.0,
            shulkerCurrentAttachmentPosition: None,
            shulkerClientSideTeleportInterpolation: 0,
            fireballAccelerationX: fireballAcceleration[0],
            fireballAccelerationY: fireballAcceleration[1],
            fireballAccelerationZ: fireballAcceleration[2],
            fishHookAnglerId,
            fishHookCaughtEntityId: None,
            fishHookState: FishHookState::Flying,
            fishHookInGround: false,
            fishHookTicksInGround: 0,
            fishHookTicksInAir: 0,
            fishHookRandom: fresh_random(entityId ^ 0x4649_5348),
            pendingParticleSpawns: Vec::new(),
            pendingSoundEvents: Vec::new(),
            fireworkLaunchSoundPlayed: false,
            areaEffectCloudRandom: fresh_random(entityId ^ 0x434c_4f55),
            wolfHeadRotationCourse: 0.0,
            wolfHeadRotationCourseOld: 0.0,
            wolfIsWet: false,
            wolfIsShaking: false,
            timeWolfIsShaking: 0.0,
            prevTimeWolfIsShaking: 0.0,
            rabbitJumpTicks: 0,
            rabbitJumpDuration: 0,
            polarStandAnimation0: 0.0,
            polarStandAnimation: 0.0,
            horseHeadLean: 0.0,
            horsePrevHeadLean: 0.0,
            horseRearingAmount: 0.0,
            horsePrevRearingAmount: 0.0,
            horseMouthOpenness: 0.0,
            horsePrevMouthOpenness: 0.0,
            horseTailCounter: 0,
            horseJumpPower: 0.0,
            horseAllowStandSliding: false,
            horseJumping: false,
            horseJumpRearingCounter: 0,
            horseMoveStrafing: 0.0,
            horseMoveForward: 0.0,
            horseRiderYaw: yaw,
            horseRiderPitch: pitch,
            horseRandom: fresh_random(entityId ^ 0x484F_5253),
            illusionTransitionTicks: 0,
            illusionOffsetsOld: [[0.0; 3]; 4],
            illusionOffsetsNew: [[0.0; 3]; 4],
            illusionRandom: fresh_random(entityId ^ 0x494C_4C55),
            boatPaddlePositions: [0.0; 2],
            boatMomentum: 0.0,
            boatOutOfControlTicks: 0.0,
            boatDeltaRotation: 0.0,
            boatLeftInputDown: false,
            boatRightInputDown: false,
            boatForwardInputDown: false,
            boatBackInputDown: false,
            boatWaterLevel: 0.0,
            boatGlide: 0.0,
            boatStatus: None,
            boatPreviousStatus: None,
            boatLastYd: 0.0,
            minecartVelocityX: 0.0,
            minecartVelocityY: 0.0,
            minecartVelocityZ: 0.0,
            minecartTntFuse: -1,
            leashHolderId: None,
            interpTargetX: x,
            interpTargetY: y,
            interpTargetZ: z,
            interpTargetYaw: yaw as f64,
            interpTargetPitch: pitch as f64,
            newPosRotationIncrements: 0,
        }
    }

    pub fn isLivingBase(&self) -> bool {
        match &self.kind {
            ClientEntityKind::Mob { .. } => true,
            ClientEntityKind::Object { objectType, .. } => objectType.isLivingBase(),
            ClientEntityKind::ExperienceOrb { .. } | ClientEntityKind::Painting { .. } => false,
        }
    }

    /// MCP `Entity.canBePushed` dispatch used by boat/minecart collision
    /// boxes. Living entities inherit EntityLivingBase except for the source
    /// overrides represented here.
    pub fn canBePushed(&self) -> bool {
        if self.entity.isDead { return false; }
        match &self.kind {
            ClientEntityKind::Mob { entityType } => {
                if matches!(entityType.registryName, "bat" | "parrot") { return false; }
                if matches!(entityType.registryName, "horse" | "donkey" | "mule" | "skeleton_horse" | "zombie_horse" | "llama") {
                    return self.entity.passengerIds.is_empty();
                }
                true
            }
            ClientEntityKind::Object { objectType, .. } => matches!(objectType, ObjectSpawnType::Boat | ObjectSpawnType::Minecart),
            ClientEntityKind::ExperienceOrb { .. } | ClientEntityKind::Painting { .. } => false,
        }
    }

    /// MCP `Entity.canBeCollidedWith` dispatch for the concrete protocol
    /// entity represented by this heterogeneous client entry. The base Entity
    /// implementation is false; living entities and the exact overriding
    /// object classes below opt in. This is used by EntityRenderer.getMouseOver
    /// and must not make dropped items/projectiles attackable by accident.
    pub fn canBeCollidedWith(&self) -> bool {
        if self.entity.isDead { return false; }
        match &self.kind {
            ClientEntityKind::Mob { .. } => true,
            ClientEntityKind::Painting { .. } => true,
            ClientEntityKind::ExperienceOrb { .. } => false,
            ClientEntityKind::Object { objectType, .. } => match objectType {
                ObjectSpawnType::Boat
                | ObjectSpawnType::Minecart
                | ObjectSpawnType::EnderCrystal
                | ObjectSpawnType::ItemFrame
                | ObjectSpawnType::LeashKnot
                | ObjectSpawnType::ShulkerBullet => true,
                ObjectSpawnType::LargeFireball => EntityLargeFireball::COLLIDABLE,
                ObjectSpawnType::ArmorStand => (self.armorStandStatus() & 0x10) == 0,
                _ => false,
            },
        }
    }

    /// MCP `Entity.getCollisionBorderSize`. Only EntityFireball overrides the
    /// base zero border among the currently represented targetable entities.
    pub fn collisionBorderSize(&self) -> f64 {
        match &self.kind {
            ClientEntityKind::Object { objectType, .. } if matches!(
                objectType,
                ObjectSpawnType::LargeFireball
                    | ObjectSpawnType::SmallFireball
                    | ObjectSpawnType::WitherSkull
                    | ObjectSpawnType::DragonFireball
            ) => 1.0,
            _ => 0.0,
        }
    }

    pub fn enderCrystalBeamTarget(&self) -> Option<BlockPos> {
        if matches!(
            &self.kind,
            ClientEntityKind::Object { objectType: ObjectSpawnType::EnderCrystal, .. }
        ) {
            EntityEnderCrystal::beamTarget(&self.dataManager)
        } else {
            None
        }
    }

    pub fn enderCrystalShouldShowBottom(&self) -> bool {
        matches!(
            &self.kind,
            ClientEntityKind::Object { objectType: ObjectSpawnType::EnderCrystal, .. }
        ) && EntityEnderCrystal::shouldShowBottom(&self.dataManager)
    }

    pub fn isChild(&self) -> bool {
        match &self.kind {
            ClientEntityKind::Mob { entityType } if matches!(entityType.registryName, "zombie" | "husk" | "zombie_villager" | "zombie_pigman") => {
                self.dataManager.boolean(12, false)
            }
            ClientEntityKind::Object { objectType: ObjectSpawnType::ArmorStand, .. } => {
                (self.dataManager.byte(11, 0) & 0x01) != 0
            }
            ClientEntityKind::Mob { entityType } if matches!(
                entityType.registryName,
                "pig" | "sheep" | "cow" | "chicken" | "mooshroom"
                    | "wolf" | "ocelot" | "rabbit" | "polar_bear"
                    | "horse" | "donkey" | "mule" | "skeleton_horse" | "zombie_horse" | "llama"
                    | "villager"
            ) => self.dataManager.boolean(12, false),
            _ => false,
        }
    }

    pub fn boatTimeSinceHit(&self) -> i32 { self.dataManager.varInt(6, 0) }
    pub fn boatForwardDirection(&self) -> i32 { self.dataManager.varInt(7, 1) }
    pub fn boatDamageTaken(&self) -> f32 { self.dataManager.float(8, 0.0) }
    pub fn boatType(&self) -> BoatType { BoatType::byId(self.dataManager.varInt(9, 0)) }
    pub fn updateBoatInputs(&mut self, left: bool, right: bool, forward: bool, back: bool) {
        self.boatLeftInputDown = left;
        self.boatRightInputDown = right;
        self.boatForwardInputDown = forward;
        self.boatBackInputDown = back;
    }
    pub fn boatPaddleState(&self, paddle: usize) -> bool {
        paddle < 2 && self.dataManager.boolean(10 + paddle as u8, false) && !self.entity.passengerIds.is_empty()
    }
    pub fn boatRowingTime(&self, paddle: usize, partialTicks: f32) -> f32 {
        if paddle >= 2 { return 0.0; }
        let current = self.boatPaddlePositions[paddle];
        EntityBoat::rowingTime(
            self.boatPaddleState(paddle),
            current - EntityBoat::PADDLE_STEP,
            current,
            partialTicks,
        )
    }

    pub fn minecartType(&self) -> MinecartType {
        match &self.kind {
            ClientEntityKind::Object { objectType: ObjectSpawnType::Minecart, data, .. } => MinecartType::byId(*data),
            _ => MinecartType::Rideable,
        }
    }
    pub fn minecartRollingAmplitude(&self) -> i32 { self.dataManager.varInt(6, 0) }
    pub fn minecartRollingDirection(&self) -> i32 { self.dataManager.varInt(7, 1) }
    pub fn minecartDamage(&self) -> f32 { self.dataManager.float(8, 0.0) }
    pub fn minecartHasDisplayTile(&self) -> bool { self.dataManager.boolean(11, false) }
    pub fn minecartDisplayStateId(&self) -> i32 {
        if self.minecartHasDisplayTile() {
            // EntityMinecart DISPLAY_TILE is a VARINT produced by
            // Block#getStateId (block id in low 12 bits, metadata in high 4),
            // not the chunk palette's block<<4|metadata key used by our model
            // registry.
            EntityMinecart::fromMcpBlockStateId(self.dataManager.varInt(9, 0))
        } else {
            self.minecartType().defaultDisplayStateId(self.dataManager.boolean(12, false))
        }
    }
    pub fn minecartDisplayOffset(&self) -> i32 {
        if self.minecartHasDisplayTile() {
            self.dataManager.varInt(10, 6)
        } else {
            self.minecartType().defaultDisplayOffset()
        }
    }
    pub fn minecartTntFuse(&self) -> i32 { self.minecartTntFuse }

    pub fn armorStandStatus(&self) -> i8 { self.dataManager.byte(11, 0) }

    /// Exact non-normal `Entity#getPushReaction` cases present in the 1.12.2
    /// client entity set. Area-effect clouds always ignore piston/shulker
    /// pushes; armor stands do so only while their marker bit is set.
    pub fn ignoresShulkerBoxPush(&self) -> bool {
        matches!(
            &self.kind,
            ClientEntityKind::Object { objectType: ObjectSpawnType::AreaEffectCloud, .. }
        ) || matches!(
            &self.kind,
            ClientEntityKind::Object { objectType: ObjectSpawnType::ArmorStand, .. }
        ) && (self.armorStandStatus() & 0x10) != 0
    }
    pub fn armorStandRotation(&self, index: u8, fallback: [f32; 3]) -> [f32; 3] {
        self.dataManager.rotations(index, fallback)
    }

    pub fn pigSaddled(&self) -> bool { self.dataManager.boolean(13, false) }

    pub fn creeperState(&self) -> i32 { self.dataManager.varInt(12, -1) }
    pub fn creeperPowered(&self) -> bool { self.dataManager.boolean(13, false) }
    pub fn creeperIgnited(&self) -> bool { self.dataManager.boolean(14, false) }
    pub fn slimeSize(&self) -> i32 { self.dataManager.varInt(12, 1).max(1) }

    /// EntityEnderman metadata keys created after EntityLiving.AI_FLAGS.
    pub fn endermanHeldBlockStateId(&self) -> Option<i32> {
        self.dataManager.optionalBlockState(12)
    }
    pub fn endermanScreaming(&self) -> bool { self.dataManager.boolean(13, false) }

    /// EntityDragon PHASE metadata (PhaseList id).
    pub fn dragonPhaseId(&self) -> i32 { self.dataManager.varInt(12, 10) }
    pub fn dragonPhaseStationary(&self) -> bool {
        matches!(self.dragonPhaseId(), 5 | 6 | 7 | 10)
    }

    /// MCP `EntityDragon#getMovementOffsets`.
    pub fn dragonMovementOffsets(&self, offset: i32, partialTicks: f32) -> [f64; 3] {
        if self.dragonRingBufferIndex < 0 {
            return [self.entity.rotationYaw as f64, self.entity.posY, 0.0];
        }
        let mut partial = if self.health <= 0.0 { 0.0 } else { partialTicks.clamp(0.0, 1.0) };
        partial = 1.0 - partial;
        let i = ((self.dragonRingBufferIndex - offset) & 63) as usize;
        let j = ((self.dragonRingBufferIndex - offset - 1) & 63) as usize;
        let yaw0 = self.dragonRingBuffer[i][0];
        let yawDelta = wrap_degrees_f64(self.dragonRingBuffer[j][0] - yaw0);
        [
            yaw0 + yawDelta * partial as f64,
            self.dragonRingBuffer[i][1]
                + (self.dragonRingBuffer[j][1] - self.dragonRingBuffer[i][1]) * partial as f64,
            self.dragonRingBuffer[i][2]
                + (self.dragonRingBuffer[j][2] - self.dragonRingBuffer[i][2]) * partial as f64,
        ]
    }

    /// MCP `EntityGuardian` metadata indices inherited after `EntityLivingBase`.
    pub fn guardianMoving(&self) -> bool { self.dataManager.boolean(12, false) }
    pub fn guardianTargetEntityId(&self) -> i32 { self.dataManager.varInt(13, 0) }
    pub fn guardianHasTarget(&self) -> bool { self.guardianTargetEntityId() != 0 }
    pub fn guardianTailAnimationAt(&self, partialTicks: f32) -> f32 {
        self.guardianTailAnimationO
            + (self.guardianTailAnimation - self.guardianTailAnimationO) * partialTicks.clamp(0.0, 1.0)
    }
    pub fn guardianSpikesAnimationAt(&self, partialTicks: f32) -> f32 {
        self.guardianSpikesAnimationO
            + (self.guardianSpikesAnimation - self.guardianSpikesAnimationO) * partialTicks.clamp(0.0, 1.0)
    }

    /// MCP `EntityShulker` metadata indices 12-15.
    pub fn shulkerAttachmentFacing(&self) -> EnumFacing {
        match self.dataManager.facing(12, EnumFacing::Down.index()) {
            0 => EnumFacing::Down,
            1 => EnumFacing::Up,
            2 => EnumFacing::North,
            3 => EnumFacing::South,
            4 => EnumFacing::West,
            5 => EnumFacing::East,
            _ => EnumFacing::Down,
        }
    }
    pub fn shulkerAttachmentPos(&self) -> Option<BlockPos> { self.dataManager.optionalBlockPos(13) }
    pub fn shulkerPeekTick(&self) -> i32 { self.dataManager.byte(14, 0) as i32 }
    pub fn shulkerColorMetadata(&self) -> u8 {
        let value = self.dataManager.byte(15, 10) as i32;
        if (0..16).contains(&value) { value as u8 } else { 0 }
    }
    pub fn shulkerClientPeekAmount(&self, partialTicks: f32) -> f32 {
        self.shulkerPrevPeekAmount
            + (self.shulkerPeekAmount - self.shulkerPrevPeekAmount) * partialTicks.clamp(0.0, 1.0)
    }
    pub const fn shulkerClientTeleportInterp(&self) -> i32 {
        self.shulkerClientSideTeleportInterpolation
    }
    pub const fn shulkerOldAttachPos(&self) -> Option<BlockPos> {
        self.shulkerCurrentAttachmentPosition
    }
    pub fn shulkerIsAttachedToBlock(&self) -> bool {
        self.shulkerCurrentAttachmentPosition.is_some() && self.shulkerAttachmentPos().is_some()
    }

    pub fn eyeHeight(&self) -> f32 {
        match &self.kind {
            // MCP 1.12.2 `EntityEnderman#getEyeHeight` is a fixed 2.55F rather than
            // the generic living-entity height fraction. Keep the exact source value
            // because it feeds ray/perspective-dependent client behavior.
            ClientEntityKind::Mob { entityType } if entityType.registryName == "enderman" => 2.55,
            ClientEntityKind::Mob { entityType }
                if matches!(entityType.registryName, "guardian" | "elder_guardian" | "squid") => self.entity.height * 0.5,
            ClientEntityKind::Mob { entityType } if entityType.registryName == "shulker" => 0.5,
            ClientEntityKind::Object { objectType: ObjectSpawnType::LeashKnot, .. } => EntityLeashKnot::EYE_HEIGHT,
            _ => self.entity.height * 0.85,
        }
    }

    /// `EntityTameable.TAMED`: sitting bit 0, angry bit 1 (wolf), tamed bit 2.
    pub fn tameableFlags(&self) -> u8 { self.dataManager.byte(13, 0) as u8 }
    pub fn tameableSitting(&self) -> bool { (self.tameableFlags() & 0x01) != 0 }
    pub fn wolfAngry(&self) -> bool { (self.tameableFlags() & 0x02) != 0 }
    pub fn tameableTamed(&self) -> bool { (self.tameableFlags() & 0x04) != 0 }
    pub fn wolfBegging(&self) -> bool { self.dataManager.boolean(16, false) }
    pub fn wolfCollarColor(&self) -> u8 { (self.dataManager.varInt(17, 14) & 15) as u8 }
    pub fn wolfTailRotation(&self) -> f32 {
        if self.wolfAngry() {
            1.5393804
        } else if self.tameableTamed() {
            let synchronizedHealth = self.dataManager.float(15, self.health);
            (0.55 - (20.0 - synchronizedHealth) * 0.02) * std::f32::consts::PI
        } else {
            std::f32::consts::PI / 5.0
        }
    }
    pub fn wolfInterestedAngle(&self, partialTicks: f32) -> f32 {
        (self.wolfHeadRotationCourseOld
            + (self.wolfHeadRotationCourse - self.wolfHeadRotationCourseOld) * partialTicks.clamp(0.0, 1.0))
            * 0.15
            * std::f32::consts::PI
    }
    pub fn wolfShakeAngle(&self, partialTicks: f32, offset: f32) -> f32 {
        let progress = ((self.prevTimeWolfIsShaking
            + (self.timeWolfIsShaking - self.prevTimeWolfIsShaking) * partialTicks.clamp(0.0, 1.0)
            + offset) / 1.8).clamp(0.0, 1.0);
        (progress * std::f32::consts::PI).sin()
            * (progress * std::f32::consts::PI * 11.0).sin()
            * 0.15
            * std::f32::consts::PI
    }
    pub fn wolfIsWet(&self) -> bool { self.wolfIsWet }
    pub fn wolfShadingWhileWet(&self, partialTicks: f32) -> f32 {
        0.75 + (self.prevTimeWolfIsShaking
            + (self.timeWolfIsShaking - self.prevTimeWolfIsShaking) * partialTicks.clamp(0.0, 1.0))
            / 2.0 * 0.25
    }
    pub fn entitySprinting(&self) -> bool { ((self.dataManager.byte(0, 0) as u8) & 0x08) != 0 }
    pub fn ocelotVariant(&self) -> i32 { self.dataManager.varInt(15, 0) }
    pub fn rabbitType(&self) -> i32 { self.dataManager.varInt(13, 0) }
    pub fn rabbitJumpCompletion(&self, partialTicks: f32) -> f32 {
        if self.rabbitJumpDuration == 0 {
            0.0
        } else {
            (self.rabbitJumpTicks as f32 + partialTicks.clamp(0.0, 1.0))
                / self.rabbitJumpDuration as f32
        }
    }
    pub fn polarBearStanding(&self) -> bool { self.dataManager.boolean(13, false) }
    pub fn polarBearStandingScale(&self, partialTicks: f32) -> f32 {
        (self.polarStandAnimation0
            + (self.polarStandAnimation - self.polarStandAnimation0) * partialTicks.clamp(0.0, 1.0))
            / 6.0
    }

    pub fn isHorseFamily(&self) -> bool {
        matches!(&self.kind, ClientEntityKind::Mob { entityType } if matches!(
            entityType.registryName,
            "horse" | "donkey" | "mule" | "skeleton_horse" | "zombie_horse" | "llama"
        ))
    }
    pub fn horseCanPassengerSteer(&self) -> bool {
        matches!(
            &self.kind,
            ClientEntityKind::Mob { entityType }
                if AbstractHorse::canBeSteered(entityType.registryName)
        ) && !self.entity.passengerIds.is_empty()
    }

    pub fn horseCanJump(&self) -> bool { IJumpingMount::canJump(self) }

    pub fn horseMovementSpeed(&self) -> f64 {
        self.attributeMap.getAttributeValue(
            "generic.movementSpeed",
            AbstractHorse::DEFAULT_MOVEMENT_SPEED,
        )
    }

    pub fn horseJumpStrength(&self) -> f64 {
        self.attributeMap.getAttributeValue(
            "horse.jumpStrength",
            AbstractHorse::DEFAULT_JUMP_STRENGTH,
        )
    }

    fn setHorseStatusFlag(&mut self, flag: u8, enabled: bool) {
        let mut status = self.horseStatus();
        if enabled { status |= flag; } else { status &= !flag; }
        self.dataManager.setByte(13, status as i8);
    }

    fn setHorseRearing(&mut self, rearing: bool) {
        if rearing { self.setHorseStatusFlag(AbstractHorse::STATUS_EATING_HAYSTACK, false); }
        self.setHorseStatusFlag(AbstractHorse::STATUS_REARING, rearing);
    }

    fn makeHorseRear(&mut self) {
        if self.horseCanPassengerSteer() {
            self.horseJumpRearingCounter = 1;
            self.setHorseRearing(true);
        }
    }

    /// Port of MCP `AbstractHorse#setJumpPower`.
    pub fn setHorseJumpPower(&mut self, jumpPowerIn: i32) {
        let Some(update) = AbstractHorse::jumpPowerUpdate(self.horseSaddled(), jumpPowerIn) else {
            return;
        };
        if update.allowStandSliding {
            self.horseAllowStandSliding = true;
        }
        if update.rear {
            self.makeHorseRear();
        }
        self.horseJumpPower = update.jumpPower;
    }

    /// Rider values consumed by `AbstractHorse#func_191986_a` on the next
    /// world entity tick, matching the previous-tick boat input ownership.
    pub fn updateHorseInputs(
        &mut self,
        riderYaw: f32,
        riderPitch: f32,
        moveStrafing: f32,
        moveForward: f32,
    ) {
        self.horseRiderYaw = riderYaw;
        self.horseRiderPitch = riderPitch;
        self.horseMoveStrafing = moveStrafing;
        self.horseMoveForward = moveForward;
    }

    pub fn horseStatus(&self) -> u8 { self.dataManager.byte(13, 0) as u8 }
    pub fn horseTame(&self) -> bool { self.horseStatus() & AbstractHorse::STATUS_TAME != 0 }
    pub fn horseSaddled(&self) -> bool { self.horseStatus() & AbstractHorse::STATUS_SADDLED != 0 }
    pub fn horseEatingHaystack(&self) -> bool { self.horseStatus() & AbstractHorse::STATUS_EATING_HAYSTACK != 0 }
    pub fn horseRearing(&self) -> bool { self.horseStatus() & AbstractHorse::STATUS_REARING != 0 }
    pub fn horseMouthOpen(&self) -> bool { self.horseStatus() & AbstractHorse::STATUS_MOUTH_OPEN != 0 }
    pub fn horseChested(&self) -> bool { self.dataManager.boolean(15, false) }
    pub fn horseBeingRidden(&self) -> bool { !self.entity.passengerIds.is_empty() }
    pub fn horseVariant(&self) -> i32 { self.dataManager.varInt(15, 0) }
    pub fn horseArmorOrdinal(&self) -> i32 { self.dataManager.varInt(16, 0).clamp(0, 3) }
    pub fn llamaStrength(&self) -> i32 { self.dataManager.varInt(16, 1).clamp(1, 5) }
    pub fn llamaDecorColor(&self) -> Option<u8> {
        let value = self.dataManager.varInt(17, -1);
        (0..16).contains(&value).then_some(value as u8)
    }
    pub fn llamaVariant(&self) -> i32 { self.dataManager.varInt(18, 0).clamp(0, 3) }
    pub fn horseGrassEatingAmount(&self, partialTicks: f32) -> f32 {
        self.horsePrevHeadLean + (self.horseHeadLean - self.horsePrevHeadLean) * partialTicks.clamp(0.0, 1.0)
    }
    pub fn horseRearingAmount(&self, partialTicks: f32) -> f32 {
        self.horsePrevRearingAmount + (self.horseRearingAmount - self.horsePrevRearingAmount) * partialTicks.clamp(0.0, 1.0)
    }
    pub fn horseMouthOpennessAngle(&self, partialTicks: f32) -> f32 {
        self.horsePrevMouthOpenness + (self.horseMouthOpenness - self.horsePrevMouthOpenness) * partialTicks.clamp(0.0, 1.0)
    }


    pub fn villagerProfession(&self) -> i32 { self.dataManager.varInt(13, 0).rem_euclid(6) }
    pub fn zombieVillagerConverting(&self) -> bool { self.dataManager.boolean(15, false) }
    pub fn zombieVillagerProfession(&self) -> i32 { self.dataManager.varInt(16, 0).rem_euclid(6) }
    pub fn witchDrinkingPotion(&self) -> bool { self.dataManager.boolean(12, false) }
    pub fn primaryHandSide(&self) -> crate::net::minecraft::util::EnumHandSide::EnumHandSide {
        if (self.dataManager.byte(11, 0) & 2) != 0 {
            crate::net::minecraft::util::EnumHandSide::EnumHandSide::Left
        } else {
            crate::net::minecraft::util::EnumHandSide::EnumHandSide::Right
        }
    }
    pub fn illagerFlags(&self) -> u8 { self.dataManager.byte(12, 0) as u8 }
    pub fn illagerAttacking(&self) -> bool { self.illagerFlags() & 1 != 0 }
    pub fn illagerSpellType(&self) -> u8 { self.dataManager.byte(13, 0).max(0) as u8 }
    pub fn illagerSpellcasting(&self) -> bool { self.illagerSpellType() > 0 }
    pub fn isInvisibleFlag(&self) -> bool { (self.dataManager.byte(0, 0) as u8 & 0x20) != 0 }
    pub fn illusionOffsets(&self, partialTicks: f32) -> [[f64; 3]; 4] {
        if self.illusionTransitionTicks <= 0 { return self.illusionOffsetsNew; }
        let blend = (((self.illusionTransitionTicks as f32 - partialTicks.clamp(0.0, 1.0)) / 3.0) as f64).powf(0.25);
        let mut result = [[0.0; 3]; 4];
        for i in 0..4 {
            for axis in 0..3 {
                result[i][axis] = self.illusionOffsetsNew[i][axis] * (1.0 - blend)
                    + self.illusionOffsetsOld[i][axis] * blend;
            }
        }
        result
    }

    pub fn sheepFleeceColor(&self) -> u8 { (self.dataManager.byte(13, 0) as u8) & 15 }

    pub fn sheepSheared(&self) -> bool { ((self.dataManager.byte(13, 0) as u8) & 16) != 0 }

    pub fn customName(&self) -> Option<&str> { self.dataManager.string(2).filter(|value| !value.is_empty()) }

    pub fn sheepHeadRotationPointY(&self, partialTicks: f32) -> f32 {
        if self.sheepTimer <= 0 {
            0.0
        } else if (4..=36).contains(&self.sheepTimer) {
            1.0
        } else if self.sheepTimer < 4 {
            (self.sheepTimer as f32 - partialTicks) / 4.0
        } else {
            -((self.sheepTimer - 40) as f32 - partialTicks) / 4.0
        }
    }

    pub fn sheepHeadRotationAngleX(&self, partialTicks: f32) -> f32 {
        if self.sheepTimer > 4 && self.sheepTimer <= 36 {
            let progress = ((self.sheepTimer - 4) as f32 - partialTicks) / 32.0;
            std::f32::consts::PI / 5.0
                + (std::f32::consts::PI * 7.0 / 100.0) * (progress * 28.7).sin()
        } else if self.sheepTimer > 0 {
            std::f32::consts::PI / 5.0
        } else {
            self.entity.rotationPitch.to_radians()
        }
    }

    pub fn chickenFlap(&self, partialTicks: f32) -> f32 {
        let rotation = self.oFlap + (self.wingRotation - self.oFlap) * partialTicks;
        let speed = self.oFlapSpeed + (self.destPos - self.oFlapSpeed) * partialTicks;
        (rotation.sin() + 1.0) * speed
    }

    pub fn swingArm(&mut self, hand: EnumHand) {
        if !self.isSwingInProgress || self.swingProgressInt >= 3 {
            self.swingProgressInt = -1;
            self.isSwingInProgress = true;
            self.swingingHand = hand;
        }
    }

    pub fn getSwingProgress(&self, partialTicks: f32) -> f32 {
        let mut difference = self.swingProgress - self.prevSwingProgress;
        if difference < 0.0 { difference += 1.0; }
        self.prevSwingProgress + difference * partialTicks.clamp(0.0, 1.0)
    }

    pub fn setServerPosition(&mut self, x: f64, y: f64, z: f64) {
        self.serverPosX = fixed_position(x);
        self.serverPosY = fixed_position(y);
        self.serverPosZ = fixed_position(z);
    }

    pub fn setPositionAndRotationDirect(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        increments: i32,
        _teleport: bool,
    ) {
        if matches!(&self.kind, ClientEntityKind::Object { objectType: ObjectSpawnType::Boat, .. }) {
            self.interpTargetX = x;
            self.interpTargetY = y;
            self.interpTargetZ = z;
            self.interpTargetYaw = yaw as f64;
            self.interpTargetPitch = pitch as f64;
            // EntityBoat ignores the packet increment and always uses ten steps.
            self.newPosRotationIncrements = EntityBoat::LERP_STEPS;
        } else if matches!(&self.kind, ClientEntityKind::Object { objectType: ObjectSpawnType::Minecart, .. }) {
            self.interpTargetX = x;
            self.interpTargetY = y;
            self.interpTargetZ = z;
            self.interpTargetYaw = yaw as f64;
            self.interpTargetPitch = pitch as f64;
            self.newPosRotationIncrements = increments + 2;
            self.entity.motionX = self.minecartVelocityX;
            self.entity.motionY = self.minecartVelocityY;
            self.entity.motionZ = self.minecartVelocityZ;
        } else if matches!(&self.kind, ClientEntityKind::Mob { entityType } if entityType.registryName == "shulker") {
            // MCP `EntityShulker#setPositionAndRotationDirect` ignores ordinary
            // interpolation packets; attachment metadata owns its position.
            self.newPosRotationIncrements = 0;
        } else if matches!(&self.kind, ClientEntityKind::Object { objectType: ObjectSpawnType::FishHook, .. }) {
            // MCP `EntityFishHook#setPositionAndRotationDirect` is deliberately empty.
            // The hook owns its client trajectory and only server metadata changes
            // its caught-entity state.
            self.newPosRotationIncrements = 0;
        } else if self.isLivingBase() {
            self.interpTargetX = x;
            self.interpTargetY = y;
            self.interpTargetZ = z;
            self.interpTargetYaw = yaw as f64;
            self.interpTargetPitch = pitch as f64;
            self.newPosRotationIncrements = increments;
        } else {
            // MCP base Entity implementation ignores increments and applies now.
            self.entity.setPosition(x, y, z);
            self.entity.rotationYaw = yaw;
            self.entity.rotationPitch = pitch;
        }
    }

    pub fn setVelocity(&mut self, x: f64, y: f64, z: f64) {
        self.entity.setVelocity(x, y, z);
        if matches!(&self.kind, ClientEntityKind::Object { objectType: ObjectSpawnType::Minecart, .. }) {
            self.minecartVelocityX = x;
            self.minecartVelocityY = y;
            self.minecartVelocityZ = z;
        }
    }

    pub fn setRotationYawHead(&mut self, yaw: f32) {
        self.rotationYawHead = yaw;
    }

    pub fn applyMetadata(&mut self, entries: impl IntoIterator<Item = (u8, DataValue)>) {
        let entries = entries.into_iter().collect::<Vec<_>>();
        let isGuardian = matches!(
            &self.kind,
            ClientEntityKind::Mob { entityType }
                if matches!(entityType.registryName, "guardian" | "elder_guardian")
        );
        let oldGuardianTarget = isGuardian.then(|| self.guardianTargetEntityId());
        let isShulker = matches!(
            &self.kind,
            ClientEntityKind::Mob { entityType } if entityType.registryName == "shulker"
        );
        let oldShulkerAttachment = isShulker.then(|| self.shulkerAttachmentPos());
        let isFishHook = matches!(&self.kind, ClientEntityKind::Object { objectType: ObjectSpawnType::FishHook, .. });
        let areaEffectCloudRadiusChanged = matches!(
            &self.kind,
            ClientEntityKind::Object { objectType: ObjectSpawnType::AreaEffectCloud, .. }
        ) && entries.iter().any(|(index, _)| *index == EntityAreaEffectCloud::RADIUS_INDEX);
        self.dataManager.setEntryValues(entries);
        if areaEffectCloudRadiusChanged {
            let radius = self.areaEffectCloudRadius();
            let x = self.entity.posX;
            let y = self.entity.posY;
            let z = self.entity.posZ;
            self.entity.setSize(EntityAreaEffectCloud::width(radius), EntityAreaEffectCloud::DEFAULT_HEIGHT);
            self.entity.setPosition(x, y, z);
        }
        if isFishHook {
            self.fishHookCaughtEntityId = EntityFishHook::hookedEntityId(
                self.dataManager.varInt(EntityFishHook::DATA_HOOKED_ENTITY_INDEX, 0),
            );
        }
        if oldGuardianTarget.is_some_and(|old| self.guardianTargetEntityId() != old) {
            // MCP `EntityGuardian#notifyDataManagerChange(TARGET_ENTITY)`.
            self.guardianAttackTime = 0;
        }
        if oldShulkerAttachment.is_some_and(|old| old != self.shulkerAttachmentPos())
            && !self.entity.isRiding()
        {
            // MCP `EntityShulker#notifyDataManagerChange(ATTACHED_BLOCK_POS)`.
            if let Some(blockPos) = self.shulkerAttachmentPos() {
                if self.shulkerCurrentAttachmentPosition.is_none() {
                    self.shulkerCurrentAttachmentPosition = Some(blockPos);
                } else {
                    self.shulkerClientSideTeleportInterpolation = 6;
                }
                let x = blockPos.x as f64 + 0.5;
                let y = blockPos.y as f64;
                let z = blockPos.z as f64 + 0.5;
                self.entity.setPosition(x, y, z);
                self.entity.prevPosX = x;
                self.entity.prevPosY = y;
                self.entity.prevPosZ = z;
                self.serverPosX = fixed_position(x);
                self.serverPosY = fixed_position(y);
                self.serverPosZ = fixed_position(z);
            }
        }
        let flags = self.dataManager.byte(0, 0);
        self.entity.sneaking = (flags & 0x02) != 0;
        if self.isLivingBase() {
            self.health = self.dataManager.float(7, self.health);
            self.refreshLivingDimensions();
        }
        if matches!(&self.kind, ClientEntityKind::Object { objectType: ObjectSpawnType::PrimedTnt, .. }) {
            self.tntFuse = self.dataManager.varInt(6, self.tntFuse);
        }
    }

    fn refreshLivingDimensions(&mut self) {
        let (width, height) = match &self.kind {
            ClientEntityKind::Object { objectType: ObjectSpawnType::ArmorStand, .. } => {
                let status = self.armorStandStatus();
                if status & 0x10 != 0 {
                    (0.0, 0.0)
                } else if status & 0x01 != 0 {
                    (0.25, 0.9875)
                } else {
                    (0.5, 1.975)
                }
            }
            ClientEntityKind::Mob { entityType } => match entityType.registryName {
                "zombie" | "husk" | "zombie_pigman" | "zombie_villager" => {
                    if self.isChild() { (0.3, 0.975) } else { (0.6, 1.95) }
                }
                "wither_skeleton" => (0.7, 2.4),
                "skeleton" | "stray" => (0.6, 1.99),
                "pig" => if self.isChild() { (0.45, 0.45) } else { (0.9, 0.9) },
                "sheep" => if self.isChild() { (0.45, 0.65) } else { (0.9, 1.3) },
                "cow" | "mooshroom" => if self.isChild() { (0.45, 0.7) } else { (0.9, 1.4) },
                "chicken" => if self.isChild() { (0.2, 0.35) } else { (0.4, 0.7) },
                "wolf" => if self.isChild() { (0.3, 0.425) } else { (0.6, 0.85) },
                "ocelot" => if self.isChild() { (0.3, 0.35) } else { (0.6, 0.7) },
                "rabbit" => if self.isChild() { (0.2, 0.25) } else { (0.4, 0.5) },
                "polar_bear" => if self.isChild() { (0.65, 0.7) } else { (1.3, 1.4) },
                "horse" | "donkey" | "mule" | "skeleton_horse" | "zombie_horse" => {
                    if self.isChild() { (0.6982422, 0.8) } else { (1.3964844, 1.6) }
                }
                "llama" => if self.isChild() { (0.45, 0.935) } else { (0.9, 1.87) },
                "villager" => if self.isChild() { (0.3, 0.975) } else { (0.6, 1.95) },
                "witch" | "vindication_illager" | "evocation_illager" | "illusion_illager" => (0.6, 1.95),
                "spider" => (1.4, 0.9),
                "cave_spider" => (0.7, 0.5),
                // Exact constructor sizes from MCP 1.12.2 EntityEnderman,
                // EntitySquid and EntityDragon. These feed Entity#getEntityBoundingBox
                // and therefore Render#shouldRender as well as interaction/collision.
                "enderman" => (0.6, 2.9),
                "squid" => (0.8, 0.8),
                "ender_dragon" => (16.0, 8.0),
                "creeper" => (0.6, 1.7),
                "slime" | "magma_cube" => {
                    let size = self.slimeSize() as f32;
                    (0.51000005 * size, 0.51000005 * size)
                }
                "guardian" => (0.85, 0.85),
                "elder_guardian" => (0.85 * 2.35, 0.85 * 2.35),
                "shulker" => (1.0, 1.0),
                _ => (self.entity.width, self.entity.height),
            },
            _ => (self.entity.width, self.entity.height),
        };
        if (width - self.entity.width).abs() > f32::EPSILON
            || (height - self.entity.height).abs() > f32::EPSILON
        {
            self.entity.width = width;
            self.entity.height = height;
            let (x, y, z) = (self.entity.posX, self.entity.posY, self.entity.posZ);
            self.entity.setPosition(x, y, z);
        }
        if matches!(&self.kind, ClientEntityKind::Object { objectType: ObjectSpawnType::ArmorStand, .. }) {
            self.entity.noClip = self.dataManager.boolean(5, false);
        }
    }

    pub fn entityItem(&self) -> Option<&ItemStack> {
        match &self.kind {
            ClientEntityKind::Object { objectType: ObjectSpawnType::Item, .. } => {
                self.dataManager.itemStack(6).filter(|stack| !stack.isEmpty())
            }
            _ => None,
        }
    }

    pub fn metadataItem(&self) -> Option<&ItemStack> {
        self.dataManager.itemStack(6).filter(|stack| !stack.isEmpty())
    }


    pub fn itemFrameDisplayedItem(&self) -> Option<&ItemStack> {
        if matches!(&self.kind, ClientEntityKind::Object { objectType: ObjectSpawnType::ItemFrame, .. }) {
            self.dataManager.itemStack(EntityItemFrame::ITEM_DATA_INDEX).filter(|stack| !stack.isEmpty())
        } else {
            None
        }
    }

    pub fn itemFrameRotation(&self) -> i32 {
        if matches!(&self.kind, ClientEntityKind::Object { objectType: ObjectSpawnType::ItemFrame, .. }) {
            EntityItemFrame::normalizedRotation(self.dataManager.varInt(EntityItemFrame::ROTATION_DATA_INDEX, 0))
        } else {
            0
        }
    }

    pub fn paintingArt(&self) -> Option<PaintingArt> { self.paintingArt }

    pub fn setItemStackToSlot(&mut self, slot: EntityEquipmentSlot, stack: ItemStack) {
        self.equipment.setItemStackToSlot(slot, stack);
    }

    pub fn setLeashHolderId(&mut self, entityId: Option<i32>) {
        self.leashHolderId = entityId;
    }

    pub fn handleStatusUpdate(&mut self, opcode: i8) {
        self.lastStatusOpcode = Some(opcode);
        match opcode {
            19 if matches!(&self.kind, ClientEntityKind::Mob { entityType } if entityType.registryName == "squid") => {
                // EntitySquid#handleStatusUpdate(19). The authoritative server
                // resets each completed rotation cycle; the remote client clamps
                // at 2PI while it waits for this status byte.
                self.squidRotation = 0.0;
            }
            8 if matches!(&self.kind, ClientEntityKind::Mob { entityType } if entityType.registryName == "wolf") => {
                self.wolfIsShaking = true;
                self.timeWolfIsShaking = 0.0;
                self.prevTimeWolfIsShaking = 0.0;
            }
            1 if matches!(&self.kind, ClientEntityKind::Mob { entityType } if entityType.registryName == "rabbit") => {
                self.rabbitJumpDuration = 10;
                self.rabbitJumpTicks = 0;
            }
            10 if self.minecartType() == MinecartType::Tnt
                && matches!(&self.kind, ClientEntityKind::Object { objectType: ObjectSpawnType::Minecart, .. }) =>
            {
                // EntityMinecartTNT#handleStatusUpdate -> ignite(). The remote
                // side does not play the priming sound here; that sound is
                // emitted by the authoritative server.
                self.minecartTntFuse = 80;
            }
            10 if matches!(&self.kind, ClientEntityKind::Mob { entityType } if entityType.registryName == "sheep") => {
                self.sheepTimer = 40;
            }
            32 if matches!(&self.kind, ClientEntityKind::Object { objectType: ObjectSpawnType::ArmorStand, .. }) => {
                self.armorStandPunchTick = Some(self.entity.ticksExisted);
            }
            2 | 33 | 36 | 37 => {
                if self.isLivingBase() {
                    self.limbSwingAmount = 1.5;
                    self.hurtResistantTime = self.maxHurtResistantTime;
                    self.maxHurtTime = 10;
                    self.hurtTime = self.maxHurtTime;
                    self.attackedAtYaw = 0.0;
                }
            }
            3 => {
                if self.isLivingBase() {
                    self.health = 0.0;
                }
            }
            _ => {}
        }
    }

    pub fn onUpdate(&mut self, world: &WorldClient, closestPlayer: Option<[f64; 3]>) {
        self.onUpdateWithLocalPlayer(world, closestPlayer, None);
    }

    pub fn onUpdateWithLocalPlayer(
        &mut self,
        world: &WorldClient,
        closestPlayer: Option<[f64; 3]>,
        localPlayerEntityId: Option<i32>,
    ) {
        self.onUpdateWithLocalPlayerState(world, closestPlayer, localPlayerEntityId, None);
    }

    pub fn onUpdateWithLocalPlayerState(
        &mut self,
        world: &WorldClient,
        closestPlayer: Option<[f64; 3]>,
        localPlayerEntityId: Option<i32>,
        localPlayerState: Option<([f64; 3], f32)>,
    ) {
        self.entity.ticksExisted = self.entity.ticksExisted.wrapping_add(1);
        if matches!(
            &self.kind,
            ClientEntityKind::Object { objectType: ObjectSpawnType::EnderCrystal, .. }
        ) {
            self.enderCrystalInnerRotation = self.enderCrystalInnerRotation.wrapping_add(1);
        }
        self.entity.prevPosX = self.entity.posX;
        self.entity.prevPosY = self.entity.posY;
        self.entity.prevPosZ = self.entity.posZ;
        self.entity.prevRotationYaw = self.entity.rotationYaw;
        self.entity.prevRotationPitch = self.entity.rotationPitch;
        self.prevRenderYawOffset = self.renderYawOffset;
        self.prevRotationYawHead = self.rotationYawHead;
        self.prevLimbSwingAmount = self.limbSwingAmount;
        self.prevSwingProgress = self.swingProgress;
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

        // EntityBoat/EntityMinecart decrement synchronized hit animation
        // fields before their client interpolation branch in onUpdate.
        if matches!(&self.kind, ClientEntityKind::Object { objectType: ObjectSpawnType::Boat, .. }) {
            let timeSinceHit = self.boatTimeSinceHit();
            if timeSinceHit > 0 { self.dataManager.setVarInt(6, timeSinceHit - 1); }
            let damageTaken = self.boatDamageTaken();
            if damageTaken > 0.0 { self.dataManager.setFloat(8, damageTaken - 1.0); }
        } else if matches!(&self.kind, ClientEntityKind::Object { objectType: ObjectSpawnType::Minecart, .. }) {
            let rollingAmplitude = self.minecartRollingAmplitude();
            if rollingAmplitude > 0 { self.dataManager.setVarInt(6, rollingAmplitude - 1); }
            let damage = self.minecartDamage();
            if damage > 0.0 { self.dataManager.setFloat(8, damage - 1.0); }
        }

        let localPlayerIsControllingPassenger = localPlayerEntityId
            .is_some_and(|playerId| self.entity.passengerIds.first() == Some(&playerId));
        let localControlsBoat = matches!(
            &self.kind,
            ClientEntityKind::Object { objectType: ObjectSpawnType::Boat, .. }
        ) && localPlayerIsControllingPassenger;
        let localCanSteerHorse = self.horseCanPassengerSteer() && localPlayerIsControllingPassenger;
        let localControlsHorse = localCanSteerHorse && self.horseSaddled();

        let isDragon = matches!(
            &self.kind, ClientEntityKind::Mob { entityType } if entityType.registryName == "ender_dragon"
        );
        // EntityDragon performs the remote interpolation inside its own
        // onLivingUpdate *after* recording the current pose into ringBuffer.
        if self.newPosRotationIncrements > 0 && !localControlsBoat && !localCanSteerHorse && !isDragon {
            let increments = self.newPosRotationIncrements as f64;
            let x = self.entity.posX + (self.interpTargetX - self.entity.posX) / increments;
            let y = self.entity.posY + (self.interpTargetY - self.entity.posY) / increments;
            let z = self.entity.posZ + (self.interpTargetZ - self.entity.posZ) / increments;
            let yawDelta = wrap_degrees_f64(self.interpTargetYaw - self.entity.rotationYaw as f64);
            let yaw = self.entity.rotationYaw as f64 + yawDelta / increments;
            let pitch = self.entity.rotationPitch as f64
                + (self.interpTargetPitch - self.entity.rotationPitch as f64) / increments;
            self.newPosRotationIncrements -= 1;
            self.entity.setPosition(x, y, z);
            self.entity.rotationYaw = yaw as f32;
            self.entity.rotationPitch = pitch as f32;
        }

        let updateKind = match &self.kind {
            ClientEntityKind::ExperienceOrb { .. } => 1,
            ClientEntityKind::Object { objectType: ObjectSpawnType::Item, .. } => 2,
            ClientEntityKind::Object { objectType: ObjectSpawnType::FallingBlock, .. } => 3,
            ClientEntityKind::Object { objectType: ObjectSpawnType::PrimedTnt, .. } => 4,
            ClientEntityKind::Object { objectType: ObjectSpawnType::TippedArrow | ObjectSpawnType::SpectralArrow, .. } => 5,
            ClientEntityKind::Object { objectType: ObjectSpawnType::Snowball, .. } => 6,
            ClientEntityKind::Object { objectType: ObjectSpawnType::Egg, .. } => 6,
            ClientEntityKind::Object { objectType: ObjectSpawnType::EnderPearl, .. } => 6,
            ClientEntityKind::Object { objectType: ObjectSpawnType::Potion, .. } => 7,
            ClientEntityKind::Object { objectType: ObjectSpawnType::ExperienceBottle, .. } => 8,
            ClientEntityKind::Object { objectType: ObjectSpawnType::EyeOfEnder, .. } => 9,
            ClientEntityKind::Object { objectType: ObjectSpawnType::FireworkRocket, .. } => 10,
            ClientEntityKind::Object { objectType: ObjectSpawnType::ShulkerBullet, .. } => 11,
            ClientEntityKind::Object { objectType: ObjectSpawnType::Boat, .. } => 12,
            ClientEntityKind::Object { objectType: ObjectSpawnType::Minecart, .. } => 13,
            ClientEntityKind::Object { objectType, .. } if objectType.isFireball() => 14,
            ClientEntityKind::Object { objectType: ObjectSpawnType::FishHook, .. } => 15,
            ClientEntityKind::Object { objectType: ObjectSpawnType::AreaEffectCloud, .. } => 16,
            _ => 0,
        };
        match updateKind {
            1 => self.updateExperienceOrb(world, closestPlayer),
            2 => self.updateEntityItem(world),
            3 => self.updateFallingBlock(world),
            4 => self.updatePrimedTnt(world),
            5 => self.updateArrow(world),
            6 => self.updateThrowable(world, 0.03),
            7 => self.updateThrowable(world, 0.05),
            8 => self.updateThrowable(world, 0.07),
            9 => self.updateEnderEye(),
            10 => self.updateFireworkRocket(world),
            11 => self.updateShulkerBullet(),
            12 => self.updateBoat(world, localControlsBoat),
            13 => self.updateMinecart(),
            14 => self.updateFireball(world),
            15 => self.updateFishHook(world, localPlayerEntityId, localPlayerState),
            16 => self.updateAreaEffectCloud(),
            _ => {}
        }
        if self.isHorseFamily() {
            self.updateHorse(world, localCanSteerHorse, localControlsHorse);
        }

        self.updatePassiveAnimationState(world);
        if self.isLivingBase() {
            self.updateLivingAnimationState();
        }

        if self.hurtTime > 0 { self.hurtTime -= 1; }
        if self.hurtResistantTime > 0 { self.hurtResistantTime -= 1; }
        if self.health <= 0.0 && self.isLivingBase() {
            if isDragon {
                // EntityDragon overrides EntityLivingBase#onDeathUpdate. Its
                // 200-tick death sequence is server-authoritative; the remote
                // client keeps rendering until SPacketDestroyEntities arrives.
                self.dragonDeathTicks = self.dragonDeathTicks.saturating_add(1);
                self.entity.setPosition(
                    self.entity.posX,
                    self.entity.posY + 0.10000000149011612,
                    self.entity.posZ,
                );
                self.entity.rotationYaw += 20.0;
                self.renderYawOffset = self.entity.rotationYaw;
            } else {
                self.deathTime = self.deathTime.saturating_add(1);
                if self.deathTime >= 20 { self.entity.isDead = true; }
            }
        }
        self.entity.firstUpdate = false;
    }

    fn updateHorse(
        &mut self,
        world: &WorldClient,
        localCanSteerHorse: bool,
        localControlsHorse: bool,
    ) {
        if self.horseJumpRearingCounter > 0 {
            self.horseJumpRearingCounter += 1;
            if self.horseJumpRearingCounter > AbstractHorse::REARING_CLEAR_TICK {
                self.horseJumpRearingCounter = 0;
                self.setHorseRearing(false);
            }
        }
        if !localCanSteerHorse { return; }

        self.entity.handleWaterMovement(world);
        if !localControlsHorse {
            // Unsaddled mounts take the ordinary EntityLivingBase travel branch
            // with their current motion; MCP does not add a bespoke horse damping term.
            self.travelHorseLiving(world, 0.0, 0.0, 0.02);
            return;
        }

        self.entity.rotationYaw = self.horseRiderYaw;
        self.entity.prevRotationYaw = self.entity.rotationYaw;
        self.entity.rotationPitch = self.horseRiderPitch * AbstractHorse::RIDER_PITCH_SCALE;
        self.renderYawOffset = self.entity.rotationYaw;
        self.rotationYawHead = self.renderYawOffset;

        let mut strafe = self.horseMoveStrafing * AbstractHorse::RIDER_STRAFE_SCALE;
        let mut forward = self.horseMoveForward;
        if forward <= 0.0 {
            forward *= AbstractHorse::REVERSE_SPEED_SCALE;
        }
        if self.entity.onGround
            && self.horseJumpPower == 0.0
            && self.horseRearing()
            && !self.horseAllowStandSliding
        {
            strafe = 0.0;
            forward = 0.0;
        }

        if self.horseJumpPower > 0.0 && !self.horseJumping && self.entity.onGround {
            self.entity.motionY = self.horseJumpStrength() * self.horseJumpPower as f64;
            self.horseJumping = true;
            let [impulseX, impulseZ] = AbstractHorse::forwardJumpImpulse(
                self.entity.rotationYaw,
                self.horseJumpPower,
                forward,
            );
            self.entity.motionX += impulseX;
            self.entity.motionZ += impulseZ;
            self.horseJumpPower = 0.0;
        }

        let movementSpeed = self.horseMovementSpeed() as f32;
        let jumpMovementFactor = movementSpeed * 0.1;
        self.travelHorseLiving(world, strafe, forward, jumpMovementFactor);

        if self.entity.onGround {
            self.horseJumpPower = 0.0;
            self.horseJumping = false;
        }

        self.prevLimbSwingAmount = self.limbSwingAmount;
        let deltaX = self.entity.posX - self.entity.prevPosX;
        let deltaZ = self.entity.posZ - self.entity.prevPosZ;
        let mut amount = ((deltaX * deltaX + deltaZ * deltaZ).sqrt() as f32) * 4.0;
        if amount > 1.0 { amount = 1.0; }
        self.limbSwingAmount += (amount - self.limbSwingAmount) * 0.4;
        self.limbSwing += self.limbSwingAmount;
    }

    fn travelHorseLiving(
        &mut self,
        world: &WorldClient,
        strafe: f32,
        forward: f32,
        jumpMovementFactor: f32,
    ) {
        if self.entity.isInWater() {
            let startY = self.entity.posY;
            self.entity.func_191958_b(strafe, 0.0, forward, 0.02);
            let (motionX, motionY, motionZ) = (self.entity.motionX, self.entity.motionY, self.entity.motionZ);
            self.entity.moveEntityLivingWithContext(world, self.entityId, motionX, motionY, motionZ);
            self.entity.motionX *= 0.8;
            self.entity.motionY *= 0.800000011920929;
            self.entity.motionZ *= 0.8;
            self.entity.motionY -= 0.02;
            if self.entity.isCollidedHorizontally
                && self.entity.isOffsetPositionInLiquid(
                    world,
                    self.entity.motionX,
                    self.entity.motionY + 0.6000000238418579 - self.entity.posY + startY,
                    self.entity.motionZ,
                )
            {
                self.entity.motionY = 0.30000001192092896;
            }
            return;
        }

        if self.entity.isInLava(world) {
            let startY = self.entity.posY;
            self.entity.func_191958_b(strafe, 0.0, forward, 0.02);
            let (motionX, motionY, motionZ) = (self.entity.motionX, self.entity.motionY, self.entity.motionZ);
            self.entity.moveEntityLivingWithContext(world, self.entityId, motionX, motionY, motionZ);
            self.entity.motionX *= 0.5;
            self.entity.motionY *= 0.5;
            self.entity.motionZ *= 0.5;
            self.entity.motionY -= 0.02;
            if self.entity.isCollidedHorizontally
                && self.entity.isOffsetPositionInLiquid(
                    world,
                    self.entity.motionX,
                    self.entity.motionY + 0.6000000238418579 - self.entity.posY + startY,
                    self.entity.motionZ,
                )
            {
                self.entity.motionY = 0.30000001192092896;
            }
            return;
        }

        let below = BlockPos::new(
            self.entity.posX.floor() as i32,
            (self.entity.boundingBox.min_y - 1.0).floor() as i32,
            self.entity.posZ.floor() as i32,
        );
        let mut friction = 0.91_f32;
        if self.entity.onGround { friction = world.getSlipperiness(below) * 0.91; }
        let accelerationFactor = 0.16277136 / (friction * friction * friction);
        let acceleration = if self.entity.onGround {
            self.horseMovementSpeed() as f32 * accelerationFactor
        } else {
            jumpMovementFactor
        };
        self.entity.func_191958_b(strafe, 0.0, forward, acceleration);

        friction = 0.91;
        if self.entity.onGround {
            let belowAfter = BlockPos::new(
                self.entity.posX.floor() as i32,
                (self.entity.boundingBox.min_y - 1.0).floor() as i32,
                self.entity.posZ.floor() as i32,
            );
            friction = world.getSlipperiness(belowAfter) * 0.91;
        }

        if EntityLivingBase::isOnLadder(world, &self.entity, false) {
            const LIMIT: f64 = EntityLivingBase::LADDER_HORIZONTAL_LIMIT;
            self.entity.motionX = self.entity.motionX.clamp(-LIMIT, LIMIT);
            self.entity.motionZ = self.entity.motionZ.clamp(-LIMIT, LIMIT);
            self.entity.fallDistance = 0.0;
            if self.entity.motionY < -LIMIT { self.entity.motionY = -LIMIT; }
        }

        let (motionX, motionY, motionZ) = (self.entity.motionX, self.entity.motionY, self.entity.motionZ);
        self.entity.moveEntityLivingWithContext(world, self.entityId, motionX, motionY, motionZ);
        if self.entity.isCollidedHorizontally && EntityLivingBase::isOnLadder(world, &self.entity, false) {
            self.entity.motionY = 0.2;
        }
        self.entity.motionY -= 0.08;
        self.entity.motionY *= 0.9800000190734863;
        self.entity.motionX *= friction as f64;
        self.entity.motionZ *= friction as f64;
    }

    fn updateBoat(&mut self, world: &WorldClient, localControlsBoat: bool) {
        self.boatPreviousStatus = self.boatStatus;
        let status = self.getBoatStatus(world);
        self.boatStatus = Some(status);

        if matches!(status, BoatStatus::UnderWater | BoatStatus::UnderFlowingWater) {
            self.boatOutOfControlTicks += 1.0;
        } else {
            self.boatOutOfControlTicks = 0.0;
        }

        if localControlsBoat {
            self.updateBoatMotion(world);
            self.controlBoat();
            self.boatLastYd = self.entity.motionY;
            let (motionX, motionY, motionZ) = (self.entity.motionX, self.entity.motionY, self.entity.motionZ);
            self.entity.moveEntityWithContext(world, self.entityId, true, motionX, motionY, motionZ);
        } else {
            // MCP EntityBoat#onUpdate clears velocity when the controlling
            // passenger is not the local user; packet interpolation owns pose.
            self.entity.motionX = 0.0;
            self.entity.motionY = 0.0;
            self.entity.motionZ = 0.0;
        }

        let hasController = !self.entity.passengerIds.is_empty();
        for paddle in 0..2 {
            let active = self.dataManager.boolean(10 + paddle as u8, false) && hasController;
            if active {
                self.boatPaddlePositions[paddle] += EntityBoat::PADDLE_STEP;
            } else {
                self.boatPaddlePositions[paddle] = 0.0;
            }
        }
    }

    fn getBoatStatus(&mut self, world: &WorldClient) -> BoatStatus {
        if let Some(status) = self.getUnderwaterStatus(world) {
            self.boatWaterLevel = self.entity.boundingBox.max_y;
            return status;
        }
        if self.checkBoatInWater(world) {
            return BoatStatus::InWater;
        }
        let glide = self.getBoatGlide(world);
        if glide > 0.0 {
            self.boatGlide = glide;
            BoatStatus::OnLand
        } else {
            BoatStatus::InAir
        }
    }

    fn getUnderwaterStatus(&self, world: &WorldClient) -> Option<BoatStatus> {
        let bounds = self.entity.boundingBox;
        let top = bounds.max_y + 0.001;
        let minX = bounds.min_x.floor() as i32;
        let maxX = bounds.max_x.ceil() as i32;
        let minY = bounds.max_y.floor() as i32;
        let maxY = top.ceil() as i32;
        let minZ = bounds.min_z.floor() as i32;
        let maxZ = bounds.max_z.ceil() as i32;
        let mut sourceWater = false;
        for x in minX..maxX {
            for y in minY..maxY {
                for z in minZ..maxZ {
                    let pos = BlockPos::new(x, y, z);
                    let state = world.getBlockState(pos);
                    if LiquidMaterial::fromState(state) == Some(LiquidMaterial::Water) {
                        let surface = y as f64 + BlockLiquid::getFilledPercentage(state, world, pos) as f64;
                        if top < surface {
                            if BlockLiquid::getLevel(state) != 0 {
                                return Some(BoatStatus::UnderFlowingWater);
                            }
                            sourceWater = true;
                        }
                    }
                }
            }
        }
        sourceWater.then_some(BoatStatus::UnderWater)
    }

    fn checkBoatInWater(&mut self, world: &WorldClient) -> bool {
        let bounds = self.entity.boundingBox;
        let minX = bounds.min_x.floor() as i32;
        let maxX = bounds.max_x.ceil() as i32;
        let minY = bounds.min_y.floor() as i32;
        let maxY = (bounds.min_y + 0.001).ceil() as i32;
        let minZ = bounds.min_z.floor() as i32;
        let maxZ = bounds.max_z.ceil() as i32;
        let mut found = false;
        self.boatWaterLevel = f64::MIN_POSITIVE;
        for x in minX..maxX {
            for y in minY..maxY {
                for z in minZ..maxZ {
                    let pos = BlockPos::new(x, y, z);
                    let state = world.getBlockState(pos);
                    if LiquidMaterial::fromState(state) == Some(LiquidMaterial::Water) {
                        let surface = y as f64 + BlockLiquid::getFilledPercentage(state, world, pos) as f64;
                        self.boatWaterLevel = self.boatWaterLevel.max(surface);
                        found |= bounds.min_y < surface;
                    }
                }
            }
        }
        found
    }

    fn getBoatGlide(&self, world: &WorldClient) -> f32 {
        let bounds = self.entity.boundingBox;
        let sample = AxisAlignedBB::new(
            bounds.min_x,
            bounds.min_y - 0.001,
            bounds.min_z,
            bounds.max_x,
            bounds.min_y,
            bounds.max_z,
        );
        let minX = sample.min_x.floor() as i32 - 1;
        let maxX = sample.max_x.ceil() as i32 + 1;
        let minY = sample.min_y.floor() as i32 - 1;
        let maxY = sample.max_y.ceil() as i32 + 1;
        let minZ = sample.min_z.floor() as i32 - 1;
        let maxZ = sample.max_z.ceil() as i32 + 1;
        let mut total = 0.0_f32;
        let mut count = 0_i32;
        for x in minX..maxX {
            for z in minZ..maxZ {
                let edgeCount = (x == minX || x == maxX - 1) as i32
                    + (z == minZ || z == maxZ - 1) as i32;
                if edgeCount == 2 { continue; }
                for y in minY..maxY {
                    if edgeCount > 0 && (y == minY || y == maxY - 1) { continue; }
                    let pos = BlockPos::new(x, y, z);
                    if world.getBlockCollisionBoxesAt(pos).into_iter().any(|box_| box_.intersects(sample)) {
                        total += world.blockSlipperiness(pos);
                        count += 1;
                    }
                }
            }
        }
        if count == 0 { 0.0 } else { total / count as f32 }
    }

    fn getWaterLevelAbove(&self, world: &WorldClient) -> f32 {
        let bounds = self.entity.boundingBox;
        let minX = bounds.min_x.floor() as i32;
        let maxX = bounds.max_x.ceil() as i32;
        let minY = bounds.max_y.floor() as i32;
        let maxY = (bounds.max_y - self.boatLastYd).ceil() as i32;
        let minZ = bounds.min_z.floor() as i32;
        let maxZ = bounds.max_z.ceil() as i32;
        for y in minY..maxY {
            let mut filled = 0.0_f32;
            for x in minX..maxX {
                for z in minZ..maxZ {
                    let pos = BlockPos::new(x, y, z);
                    let state = world.getBlockState(pos);
                    if LiquidMaterial::fromState(state) == Some(LiquidMaterial::Water) {
                        filled = filled.max(BlockLiquid::getFilledPercentage(state, world, pos));
                    }
                }
            }
            if filled < 1.0 { return y as f32 + filled; }
        }
        (maxY + 1) as f32
    }

    fn updateBoatMotion(&mut self, world: &WorldClient) {
        let gravity = -0.03999999910593033_f64;
        let mut verticalAcceleration = gravity;
        let mut buoyancy = 0.0_f64;
        self.boatMomentum = 0.05;
        let status = self.boatStatus.unwrap_or(BoatStatus::InAir);

        if self.boatPreviousStatus == Some(BoatStatus::InAir)
            && !matches!(status, BoatStatus::InAir | BoatStatus::OnLand)
        {
            self.boatWaterLevel = self.entity.boundingBox.min_y + self.entity.height as f64;
            let y = (self.getWaterLevelAbove(world) - self.entity.height) as f64 + 0.101;
            self.entity.setPosition(self.entity.posX, y, self.entity.posZ);
            self.entity.motionY = 0.0;
            self.boatLastYd = 0.0;
            self.boatStatus = Some(BoatStatus::InWater);
            return;
        }

        match status {
            BoatStatus::InWater => {
                buoyancy = (self.boatWaterLevel - self.entity.boundingBox.min_y) / self.entity.height as f64;
                self.boatMomentum = 0.9;
            }
            BoatStatus::UnderFlowingWater => {
                verticalAcceleration = -0.0007;
                self.boatMomentum = 0.9;
            }
            BoatStatus::UnderWater => {
                buoyancy = 0.009999999776482582;
                self.boatMomentum = 0.45;
            }
            BoatStatus::InAir => self.boatMomentum = 0.9,
            BoatStatus::OnLand => {
                self.boatMomentum = self.boatGlide;
                if !self.entity.passengerIds.is_empty() {
                    self.boatGlide /= 2.0;
                }
            }
        }

        self.entity.motionX *= self.boatMomentum as f64;
        self.entity.motionZ *= self.boatMomentum as f64;
        self.boatDeltaRotation *= self.boatMomentum;
        self.entity.motionY += verticalAcceleration;
        if buoyancy > 0.0 {
            self.entity.motionY += buoyancy * 0.06153846016296973;
            self.entity.motionY *= 0.75;
        }
    }

    fn controlBoat(&mut self) {
        if self.entity.passengerIds.is_empty() { return; }
        let mut acceleration = 0.0_f32;
        if self.boatLeftInputDown { self.boatDeltaRotation -= 1.0; }
        if self.boatRightInputDown { self.boatDeltaRotation += 1.0; }
        if self.boatRightInputDown != self.boatLeftInputDown
            && !self.boatForwardInputDown
            && !self.boatBackInputDown
        {
            acceleration += 0.005;
        }
        self.entity.rotationYaw += self.boatDeltaRotation;
        if self.boatForwardInputDown { acceleration += 0.04; }
        if self.boatBackInputDown { acceleration -= 0.005; }
        let yaw = self.entity.rotationYaw * 0.017453292;
        self.entity.motionX += (-yaw).sin() as f64 * acceleration as f64;
        self.entity.motionZ += yaw.cos() as f64 * acceleration as f64;
        self.dataManager.setBoolean(
            10,
            self.boatRightInputDown && !self.boatLeftInputDown || self.boatForwardInputDown,
        );
        self.dataManager.setBoolean(
            11,
            self.boatLeftInputDown && !self.boatRightInputDown || self.boatForwardInputDown,
        );
    }

    fn updateMinecart(&mut self) {
        // On the remote client the authoritative interpolation branch owns the
        // position. When no packet is active vanilla re-applies the same pose.
        if self.newPosRotationIncrements <= 0 {
            let (x, y, z) = (self.entity.posX, self.entity.posY, self.entity.posZ);
            self.entity.setPosition(x, y, z);
        }
        if self.minecartType() == MinecartType::Tnt && self.minecartTntFuse > 0 {
            // EntityMinecartTNT#onUpdate decrements before spawning its smoke.
            // The smoke particle remains an explicitly tracked particle-system
            // difference; the fuse state itself is exact and drives rendering.
            self.minecartTntFuse -= 1;
        }
    }

    fn updateSquidAnimationState(&mut self, world: &WorldClient) {
        // Direct remote-side subset of MCP EntitySquid#onLivingUpdate. The
        // server owns randomMotionVec and therefore motion packets; the client
        // owns only the visible interpolation/orientation state below.
        self.entity.handleWaterMovement(world);
        self.squidPrevPitch = self.squidPitch;
        self.squidPrevYaw = self.squidYaw;
        self.squidPrevRotation = self.squidRotation;
        self.squidLastTentacleAngle = self.squidTentacleAngle;
        self.squidRotation += self.squidRotationVelocity;

        if self.squidRotation as f64 > std::f64::consts::TAU {
            // world.isRemote branch: wait for server status opcode 19.
            self.squidRotation = std::f32::consts::TAU;
        }

        if self.entity.inWater {
            if self.squidRotation < std::f32::consts::PI {
                let f = self.squidRotation / std::f32::consts::PI;
                self.squidTentacleAngle = (f * f * std::f32::consts::PI).sin()
                    * std::f32::consts::PI * 0.25;
                if f as f64 > 0.75 {
                    self.squidRandomMotionSpeed = 1.0;
                    self.squidRotateSpeed = 1.0;
                } else {
                    self.squidRotateSpeed *= 0.8;
                }
            } else {
                self.squidTentacleAngle = 0.0;
                self.squidRandomMotionSpeed *= 0.9;
                self.squidRotateSpeed *= 0.99;
            }
            let horizontal = (self.entity.motionX * self.entity.motionX
                + self.entity.motionZ * self.entity.motionZ).sqrt() as f32;
            self.renderYawOffset += (
                -(self.entity.motionX.atan2(self.entity.motionZ) as f32).to_degrees()
                    - self.renderYawOffset
            ) * 0.1;
            self.entity.rotationYaw = self.renderYawOffset;
            self.squidYaw += std::f32::consts::PI * self.squidRotateSpeed * 1.5;
            self.squidPitch += (
                -(horizontal as f64).atan2(self.entity.motionY) as f32 * (180.0 / std::f32::consts::PI)
                    - self.squidPitch
            ) * 0.1;
        } else {
            self.squidTentacleAngle = self.squidRotation.sin().abs() * std::f32::consts::PI * 0.25;
            self.squidPitch += (-90.0 - self.squidPitch) * 0.02;
        }
    }

    fn updateDragonAnimationState(&mut self) {
        // Direct client branch of EntityDragon#onLivingUpdate, excluding phase
        // particle/sound side effects. It preserves the animation clock,
        // ring-buffer write order and remote interpolation used by ModelDragon.
        self.dragonPrevAnimTime = self.dragonAnimTime;
        // MCP EntityDragon#onLivingUpdate places yaw wrapping, AI/ring-buffer
        // maintenance and remote interpolation inside the health>0 branch.
        // Once dead, only the death particle/onDeathUpdate path remains active.
        if self.health <= 0.0 {
            return;
        }
        let horizontal = (self.entity.motionX * self.entity.motionX
            + self.entity.motionZ * self.entity.motionZ).sqrt() as f32;
        let mut increment = 0.2 / (horizontal * 10.0 + 1.0);
        increment *= (2.0_f64.powf(self.entity.motionY)) as f32;
        if self.dragonPhaseStationary() {
            self.dragonAnimTime += 0.1;
        } else if self.dragonSlowed {
            self.dragonAnimTime += increment * 0.5;
        } else {
            self.dragonAnimTime += increment;
        }

        self.entity.rotationYaw = wrap_degrees_f64(self.entity.rotationYaw as f64) as f32;
        let aiDisabled = (self.dataManager.byte(11, 0) & 0x01) != 0;
        if aiDisabled {
            self.dragonAnimTime = 0.5;
            return;
        }

        if self.dragonRingBufferIndex < 0 {
            for entry in &mut self.dragonRingBuffer {
                entry[0] = self.entity.rotationYaw as f64;
                entry[1] = self.entity.posY;
            }
        }
        self.dragonRingBufferIndex += 1;
        if self.dragonRingBufferIndex == 64 { self.dragonRingBufferIndex = 0; }
        let index = self.dragonRingBufferIndex as usize;
        self.dragonRingBuffer[index][0] = self.entity.rotationYaw as f64;
        self.dragonRingBuffer[index][1] = self.entity.posY;

        if self.newPosRotationIncrements > 0 {
            let increments = self.newPosRotationIncrements as f64;
            let x = self.entity.posX + (self.interpTargetX - self.entity.posX) / increments;
            let y = self.entity.posY + (self.interpTargetY - self.entity.posY) / increments;
            let z = self.entity.posZ + (self.interpTargetZ - self.entity.posZ) / increments;
            let yawDelta = wrap_degrees_f64(self.interpTargetYaw - self.entity.rotationYaw as f64);
            self.entity.rotationYaw = (self.entity.rotationYaw as f64 + yawDelta / increments) as f32;
            self.entity.rotationPitch = (self.entity.rotationPitch as f64
                + (self.interpTargetPitch - self.entity.rotationPitch as f64) / increments) as f32;
            self.newPosRotationIncrements -= 1;
            self.entity.setPosition(x, y, z);
        }
        self.renderYawOffset = self.entity.rotationYaw;
    }

    fn updatePassiveAnimationState(&mut self, world: &WorldClient) {
        let registryName = match &self.kind {
            ClientEntityKind::Mob { entityType } => entityType.registryName,
            _ => return,
        };
        if registryName == "sheep" && self.sheepTimer > 0 {
            self.sheepTimer -= 1;
        }
        if matches!(registryName, "guardian" | "elder_guardian") {
            self.updateGuardianAnimationState(world);
        }
        if registryName == "squid" {
            self.updateSquidAnimationState(world);
        }
        if registryName == "ender_dragon" {
            self.updateDragonAnimationState();
        }
        if registryName == "shulker" {
            self.updateShulkerAnimationState();
        }
        if registryName == "wolf" {
            self.wolfHeadRotationCourseOld = self.wolfHeadRotationCourse;
            let target = if self.wolfBegging() { 1.0 } else { 0.0 };
            self.wolfHeadRotationCourse += (target - self.wolfHeadRotationCourse) * 0.4;
            if world.isMaterialInBB(self.entity.boundingBox, LiquidMaterial::Water) {
                self.wolfIsWet = true;
                self.wolfIsShaking = false;
                self.timeWolfIsShaking = 0.0;
                self.prevTimeWolfIsShaking = 0.0;
            } else if (self.wolfIsWet || self.wolfIsShaking) && self.wolfIsShaking {
                self.prevTimeWolfIsShaking = self.timeWolfIsShaking;
                self.timeWolfIsShaking += 0.05;
                if self.prevTimeWolfIsShaking >= 2.0 {
                    self.wolfIsWet = false;
                    self.wolfIsShaking = false;
                    self.prevTimeWolfIsShaking = 0.0;
                    self.timeWolfIsShaking = 0.0;
                }
            }
        }
        if registryName == "rabbit" {
            if self.rabbitJumpTicks != self.rabbitJumpDuration {
                self.rabbitJumpTicks += 1;
            } else if self.rabbitJumpDuration != 0 {
                self.rabbitJumpTicks = 0;
                self.rabbitJumpDuration = 0;
            }
        }
        if registryName == "polar_bear" {
            self.polarStandAnimation0 = self.polarStandAnimation;
            self.polarStandAnimation = if self.polarBearStanding() {
                (self.polarStandAnimation + 1.0).clamp(0.0, 6.0)
            } else {
                (self.polarStandAnimation - 1.0).clamp(0.0, 6.0)
            };
        }
        if matches!(registryName, "horse" | "donkey" | "mule" | "skeleton_horse" | "zombie_horse" | "llama") {
            // MCP AbstractHorse.onLivingUpdate uses Entity.rand.nextInt(200)
            // on both logical sides before the server-only branch.
            if self.horseTailCounter == 0 && self.horseRandom.next_i32_bound(200) == 0 {
                self.horseTailCounter = 1;
            } else if self.horseTailCounter > 0 {
                self.horseTailCounter += 1;
                if self.horseTailCounter > 8 { self.horseTailCounter = 0; }
            }
            self.horsePrevHeadLean = self.horseHeadLean;
            if self.horseEatingHaystack() {
                self.horseHeadLean += (1.0 - self.horseHeadLean) * 0.4 + 0.05;
                self.horseHeadLean = self.horseHeadLean.min(1.0);
            } else {
                self.horseHeadLean += (0.0 - self.horseHeadLean) * 0.4 - 0.05;
                self.horseHeadLean = self.horseHeadLean.max(0.0);
            }
            self.horsePrevRearingAmount = self.horseRearingAmount;
            if self.horseRearing() {
                self.horseHeadLean = 0.0;
                self.horsePrevHeadLean = 0.0;
                self.horseRearingAmount += (1.0 - self.horseRearingAmount) * 0.4 + 0.05;
                self.horseRearingAmount = self.horseRearingAmount.min(1.0);
            } else {
                self.horseAllowStandSliding = false;
                self.horseRearingAmount += (0.8 * self.horseRearingAmount.powi(3) - self.horseRearingAmount) * 0.6 - 0.05;
                self.horseRearingAmount = self.horseRearingAmount.max(0.0);
            }
            self.horsePrevMouthOpenness = self.horseMouthOpenness;
            if self.horseMouthOpen() {
                self.horseMouthOpenness += (1.0 - self.horseMouthOpenness) * 0.7 + 0.05;
                self.horseMouthOpenness = self.horseMouthOpenness.min(1.0);
            } else {
                self.horseMouthOpenness += (0.0 - self.horseMouthOpenness) * 0.7 - 0.05;
                self.horseMouthOpenness = self.horseMouthOpenness.max(0.0);
            }
        }
        if registryName == "illusion_illager" && self.isInvisibleFlag() {
            self.illusionTransitionTicks -= 1;
            if self.illusionTransitionTicks < 0 { self.illusionTransitionTicks = 0; }
            if self.hurtTime != 1 && self.entity.ticksExisted % 1200 != 0 {
                if self.hurtTime == self.maxHurtTime - 1 {
                    self.illusionTransitionTicks = 3;
                    self.illusionOffsetsOld = self.illusionOffsetsNew;
                    self.illusionOffsetsNew = [[0.0; 3]; 4];
                }
            } else {
                self.illusionTransitionTicks = 3;
                self.illusionOffsetsOld = self.illusionOffsetsNew;
                for offset in &mut self.illusionOffsetsNew {
                    *offset = [
                        (-6 + self.illusionRandom.next_i32_bound(13)) as f64 * 0.5,
                        (self.illusionRandom.next_i32_bound(6) - 4).max(0) as f64,
                        (-6 + self.illusionRandom.next_i32_bound(13)) as f64 * 0.5,
                    ];
                }
            }
        }
        if registryName == "chicken" {
            self.oFlap = self.wingRotation;
            self.oFlapSpeed = self.destPos;
            self.destPos += if self.entity.onGround { -0.3 } else { 1.2 };
            self.destPos = self.destPos.clamp(0.0, 1.0);
            if !self.entity.onGround && self.wingRotDelta < 1.0 {
                self.wingRotDelta = 1.0;
            }
            self.wingRotDelta *= 0.9;
            if !self.entity.onGround && self.entity.motionY < 0.0 {
                self.entity.motionY *= 0.6;
            }
            self.wingRotation += self.wingRotDelta * 2.0;
        }
        if registryName == "creeper" {
            self.lastActiveTime = self.timeSinceIgnited;
            let state = if self.creeperIgnited() { 1 } else { self.creeperState() };
            self.timeSinceIgnited = (self.timeSinceIgnited + state).clamp(0, 30);
        }
        if matches!(registryName, "slime" | "magma_cube") {
            self.squishFactor += (self.squishAmount - self.squishFactor) * 0.5;
            self.prevSquishFactor = self.squishFactor;
            if self.entity.onGround && !self.wasOnGround {
                self.squishAmount = -0.5;
            } else if !self.entity.onGround && self.wasOnGround {
                self.squishAmount = 1.0;
            }
            self.wasOnGround = self.entity.onGround;
            self.squishAmount *= if registryName == "magma_cube" { 0.9 } else { 0.6 };
        }
    }

    fn updateShulkerAnimationState(&mut self) {
        // Client-visible attachment/peek branch of MCP
        // `EntityShulker#onUpdate`, plus its stationary living pose. Server
        // attachment validation and teleport choice are authoritative. The
        // source's client-side opening collision push is tracked separately
        // because WorldClient currently ticks its heterogeneous map detached.
        let targetPeek = self.shulkerPeekTick() as f32 * 0.01;
        self.shulkerPrevPeekAmount = self.shulkerPeekAmount;
        if self.shulkerPeekAmount > targetPeek {
            self.shulkerPeekAmount = (self.shulkerPeekAmount - 0.05).clamp(targetPeek, 1.0);
        } else if self.shulkerPeekAmount < targetPeek {
            self.shulkerPeekAmount = (self.shulkerPeekAmount + 0.05).clamp(0.0, targetPeek);
        }

        if self.entity.isRiding() {
            self.shulkerClientSideTeleportInterpolation = 0;
        } else if let Some(blockPos) = self.shulkerAttachmentPos() {
            if self.shulkerClientSideTeleportInterpolation > 0
                && self.shulkerCurrentAttachmentPosition.is_some()
            {
                self.shulkerClientSideTeleportInterpolation -= 1;
            } else {
                self.shulkerCurrentAttachmentPosition = Some(blockPos);
            }

            let x = blockPos.x as f64 + 0.5;
            let y = blockPos.y as f64;
            let z = blockPos.z as f64 + 0.5;
            self.entity.posX = x;
            self.entity.posY = y;
            self.entity.posZ = z;
            self.entity.prevPosX = x;
            self.entity.prevPosY = y;
            self.entity.prevPosZ = z;

            let currentExtension = 0.5
                - ((0.5 + self.shulkerPeekAmount) * std::f32::consts::PI).sin() as f64 * 0.5;
            let previousExtension = 0.5
                - ((0.5 + self.shulkerPrevPeekAmount) * std::f32::consts::PI).sin() as f64 * 0.5;
            let _extensionDelta = currentExtension - previousExtension;
            self.entity.boundingBox = match self.shulkerAttachmentFacing() {
                EnumFacing::Down => AxisAlignedBB::new(
                    x - 0.5, y, z - 0.5,
                    x + 0.5, y + 1.0 + currentExtension, z + 0.5,
                ),
                EnumFacing::Up => AxisAlignedBB::new(
                    x - 0.5, y - currentExtension, z - 0.5,
                    x + 0.5, y + 1.0, z + 0.5,
                ),
                EnumFacing::North => AxisAlignedBB::new(
                    x - 0.5, y, z - 0.5,
                    x + 0.5, y + 1.0, z + 0.5 + currentExtension,
                ),
                EnumFacing::South => AxisAlignedBB::new(
                    x - 0.5, y, z - 0.5 - currentExtension,
                    x + 0.5, y + 1.0, z + 0.5,
                ),
                EnumFacing::West => AxisAlignedBB::new(
                    x - 0.5, y, z - 0.5,
                    x + 0.5 + currentExtension, y + 1.0, z + 0.5,
                ),
                EnumFacing::East => AxisAlignedBB::new(
                    x - 0.5 - currentExtension, y, z - 0.5,
                    x + 0.5, y + 1.0, z + 0.5,
                ),
            };
        }

        self.entity.motionX = 0.0;
        self.entity.motionY = 0.0;
        self.entity.motionZ = 0.0;
        self.prevRenderYawOffset = 180.0;
        self.renderYawOffset = 180.0;
        self.entity.rotationYaw = 180.0;
    }

    fn updateGuardianAnimationState(&mut self, world: &WorldClient) {
        // Client-side branch of MCP `EntityGuardian#onLivingUpdate`. The
        // particle and sound calls remain deferred to their concrete systems.
        self.guardianTailAnimationO = self.guardianTailAnimation;
        let inWater = self.entity.handleWaterMovement(world);
        if !inWater {
            self.guardianTailAnimationSpeed = 2.0;
            let below = BlockPos::from_f64(self.entity.posX, self.entity.posY, self.entity.posZ).down(1);
            self.guardianTouchedGround = self.entity.motionY < 0.0
                && world.getBlockState(below).getBlock().isOpaqueCube();
        } else if self.guardianMoving() {
            if self.guardianTailAnimationSpeed < 0.5 {
                self.guardianTailAnimationSpeed = 4.0;
            } else {
                self.guardianTailAnimationSpeed += (0.5 - self.guardianTailAnimationSpeed) * 0.1;
            }
        } else {
            self.guardianTailAnimationSpeed += (0.125 - self.guardianTailAnimationSpeed) * 0.2;
        }
        self.guardianTailAnimation += self.guardianTailAnimationSpeed;

        self.guardianSpikesAnimationO = self.guardianSpikesAnimation;
        if !inWater {
            self.guardianSpikesAnimation = self.guardianRandom.next_f32();
        } else if self.guardianMoving() {
            self.guardianSpikesAnimation += (0.0 - self.guardianSpikesAnimation) * 0.25;
        } else {
            self.guardianSpikesAnimation += (1.0 - self.guardianSpikesAnimation) * 0.06;
        }

        if self.guardianHasTarget() {
            let duration = if matches!(
                &self.kind,
                ClientEntityKind::Mob { entityType } if entityType.registryName == "elder_guardian"
            ) { 60 } else { 80 };
            if self.guardianAttackTime < duration {
                self.guardianAttackTime += 1;
            }
            // Final client-side branch of `EntityGuardian#onLivingUpdate`.
            self.entity.rotationYaw = self.rotationYawHead;
        }
    }

    fn updateLivingAnimationState(&mut self) {
        let dx = self.entity.posX - self.entity.prevPosX;
        let dz = self.entity.posZ - self.entity.prevPosZ;
        let horizontalSquared = (dx * dx + dz * dz) as f32;
        let mut targetBodyYaw = self.renderYawOffset;
        if horizontalSquared > 0.0025000002 {
            let movementYaw = (dz.atan2(dx) as f32).to_degrees() - 90.0;
            let difference = (wrap_degrees_f32(self.entity.rotationYaw) - movementYaw).abs();
            targetBodyYaw = if difference > 95.0 && difference < 265.0 {
                movementYaw - 180.0
            } else {
                movementYaw
            };
        }
        if self.swingProgress > 0.0 {
            targetBodyYaw = self.entity.rotationYaw;
        }
        let delta = wrap_degrees_f32(targetBodyYaw - self.renderYawOffset);
        self.renderYawOffset += delta * 0.3;
        let mut relative = wrap_degrees_f32(self.entity.rotationYaw - self.renderYawOffset);
        relative = relative.clamp(-75.0, 75.0);
        self.renderYawOffset = self.entity.rotationYaw - relative;
        if relative * relative > 2500.0 {
            self.renderYawOffset += relative * 0.2;
        }

        let mut amount = ((dx * dx + dz * dz).sqrt() as f32) * 4.0;
        if amount > 1.0 { amount = 1.0; }
        self.limbSwingAmount += (amount - self.limbSwingAmount) * 0.4;
        self.limbSwing += self.limbSwingAmount;

        normalize_previous(self.entity.rotationYaw, &mut self.entity.prevRotationYaw);
        normalize_previous(self.renderYawOffset, &mut self.prevRenderYawOffset);
        normalize_previous(self.entity.rotationPitch, &mut self.entity.prevRotationPitch);
        normalize_previous(self.rotationYawHead, &mut self.prevRotationYawHead);
    }

    fn updateEntityItem(&mut self, world: &WorldClient) {
        self.entity.motionY -= 0.03999999910593033;
        let (motionX, motionY, motionZ) = (self.entity.motionX, self.entity.motionY, self.entity.motionZ);
        self.entity.moveEntity(world, motionX, motionY, motionZ);
        let mut friction = 0.98_f64;
        if self.entity.onGround {
            let below = BlockPos::new(
                self.entity.posX.floor() as i32,
                self.entity.boundingBox.min_y.floor() as i32 - 1,
                self.entity.posZ.floor() as i32,
            );
            friction = world.blockSlipperiness(below) as f64 * 0.98;
        }
        self.entity.motionX *= friction;
        self.entity.motionY *= 0.9800000190734863;
        self.entity.motionZ *= friction;
        if self.entity.onGround { self.entity.motionY *= -0.5; }
        self.entity.handleWaterMovement(world);
    }

    fn updateExperienceOrb(&mut self, world: &WorldClient, closestPlayer: Option<[f64; 3]>) {
        self.entity.motionY -= 0.029999999329447746;
        if let Some(target) = closestPlayer {
            let dx = (target[0] - self.entity.posX) / 8.0;
            let dy = (target[1] - self.entity.posY) / 8.0;
            let dz = (target[2] - self.entity.posZ) / 8.0;
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();
            let attraction = 1.0 - distance;
            if attraction > 0.0 && distance > 1.0e-7 {
                let strength = attraction * attraction * 0.1 / distance;
                self.entity.motionX += dx * strength;
                self.entity.motionY += dy * strength;
                self.entity.motionZ += dz * strength;
            }
        }
        let (motionX, motionY, motionZ) = (self.entity.motionX, self.entity.motionY, self.entity.motionZ);
        self.entity.moveEntity(world, motionX, motionY, motionZ);
        let mut friction = 0.98_f64;
        if self.entity.onGround {
            let below = BlockPos::new(
                self.entity.posX.floor() as i32,
                self.entity.boundingBox.min_y.floor() as i32 - 1,
                self.entity.posZ.floor() as i32,
            );
            friction = world.blockSlipperiness(below) as f64 * 0.98;
        }
        self.entity.motionX *= friction;
        self.entity.motionY *= 0.9800000190734863;
        self.entity.motionZ *= friction;
        if self.entity.onGround { self.entity.motionY *= -0.8999999761581421; }
        self.xpColor = self.xpColor.wrapping_add(1);
        self.xpOrbAge = self.xpOrbAge.saturating_add(1);
    }

    fn updateFallingBlock(&mut self, world: &WorldClient) {
        self.entity.motionY -= 0.03999999910593033;
        let (motionX, motionY, motionZ) = (self.entity.motionX, self.entity.motionY, self.entity.motionZ);
        self.entity.moveEntity(world, motionX, motionY, motionZ);
        self.entity.motionX *= 0.9800000190734863;
        self.entity.motionY *= 0.9800000190734863;
        self.entity.motionZ *= 0.9800000190734863;
    }

    fn updatePrimedTnt(&mut self, world: &WorldClient) {
        self.entity.motionY -= 0.03999999910593033;
        let (motionX, motionY, motionZ) = (self.entity.motionX, self.entity.motionY, self.entity.motionZ);
        self.entity.moveEntity(world, motionX, motionY, motionZ);
        self.entity.motionX *= 0.9800000190734863;
        self.entity.motionY *= 0.9800000190734863;
        self.entity.motionZ *= 0.9800000190734863;
        if self.entity.onGround {
            self.entity.motionX *= 0.699999988079071;
            self.entity.motionZ *= 0.699999988079071;
            self.entity.motionY *= -0.5;
        }
        self.tntFuse = self.tntFuse.saturating_sub(1);
        self.entity.handleWaterMovement(world);
    }

    fn updateThrowable(&mut self, world: &WorldClient, gravity: f64) {
        let x = self.entity.posX + self.entity.motionX;
        let y = self.entity.posY + self.entity.motionY;
        let z = self.entity.posZ + self.entity.motionZ;
        self.entity.posX = x;
        self.entity.posY = y;
        self.entity.posZ = z;
        self.updateProjectileRotation();
        self.entity.setPosition(x, y, z);
        let inWater = world.isMaterialInBB(
            self.entity.boundingBox,
            crate::net::minecraft::block::BlockLiquid::LiquidMaterial::Water,
        );
        let drag = if inWater { 0.8 } else { 0.99 };
        self.entity.motionX *= drag;
        self.entity.motionY *= drag;
        self.entity.motionZ *= drag;
        self.entity.motionY -= gravity;
    }

    /// Client half of MCP `EntityEnderEye#onUpdate`. Target steering,
    /// despawn/drop and portal particles remain server/effects owned.
    fn updateEnderEye(&mut self) {
        let x = self.entity.posX + self.entity.motionX;
        let y = self.entity.posY + self.entity.motionY;
        let z = self.entity.posZ + self.entity.motionZ;
        self.entity.posX = x;
        self.entity.posY = y;
        self.entity.posZ = z;
        self.updateProjectileRotation();
        self.entity.setPosition(x, y, z);
    }

    /// Client-visible half of MCP `EntityFireworkRocket#onUpdate`.
    /// Attached rockets are advanced after EntityPlayerSP so the exact owner
    /// look vector and freshly integrated local motion are available.
    fn updateFireworkRocket(&mut self, world: &WorldClient) {
        let attachedEntityId = self.dataManager.varInt(7, 0);
        if attachedEntityId <= 0 {
            self.entity.motionX *= 1.15;
            self.entity.motionZ *= 1.15;
            self.entity.motionY += 0.04;
            let (motionX, motionY, motionZ) = (self.entity.motionX, self.entity.motionY, self.entity.motionZ);
            self.entity.moveEntity(world, motionX, motionY, motionZ);
        }
        self.updateProjectileRotation();
        if !self.fireworkLaunchSoundPlayed && !self.dataManager.boolean(4, false) {
            self.pendingSoundEvents.push(LocalSoundEvent::positioned(
                "entity.firework.launch",
                SoundCategory::Ambient,
                [self.entity.posX as f32, self.entity.posY as f32, self.entity.posZ as f32],
                3.0,
                1.0,
            ));
            self.fireworkLaunchSoundPlayed = true;
        }
    }

    /// Attached-owner branch of MCP `EntityFireworkRocket#onUpdate`. This is
    /// called once after the externally owned local player has ticked.
    pub fn updateAttachedFireworkForLocalPlayer(&mut self, player: &mut EntityPlayerSP) -> bool {
        if !matches!(
            &self.kind,
            ClientEntityKind::Object { objectType: ObjectSpawnType::FireworkRocket, .. }
        ) || self.dataManager.varInt(7, 0) != player.entityId
        {
            return false;
        }
        if player.isElytraFlying() {
            let look = player.getLook(1.0);
            player.entity.motionX += look.x * 0.1 + (look.x * 1.5 - player.entity.motionX) * 0.5;
            player.entity.motionY += look.y * 0.1 + (look.y * 1.5 - player.entity.motionY) * 0.5;
            player.entity.motionZ += look.z * 0.1 + (look.z * 1.5 - player.entity.motionZ) * 0.5;
        }
        self.entity.setPosition(player.entity.posX, player.entity.posY, player.entity.posZ);
        self.entity.motionX = player.entity.motionX;
        self.entity.motionY = player.entity.motionY;
        self.entity.motionZ = player.entity.motionZ;
        self.updateProjectileRotation();
        true
    }

    pub fn takeSoundEvents(&mut self) -> Vec<LocalSoundEvent> {
        core::mem::take(&mut self.pendingSoundEvents)
    }

    fn updateShulkerBullet(&mut self) {
        // Client branch of MCP `EntityShulkerBullet#onUpdate`: authoritative
        // steering occurs server-side, while the client integrates the current
        // synchronized velocity and rotates toward it with factor 0.5.
        let x = self.entity.posX + self.entity.motionX;
        let y = self.entity.posY + self.entity.motionY;
        let z = self.entity.posZ + self.entity.motionZ;
        self.entity.setPosition(x, y, z);

        let (yaw, pitch) = ProjectileHelper::rotateTowardsMovement(
            self.entity.motionX,
            self.entity.motionY,
            self.entity.motionZ,
            &mut self.entity.prevRotationYaw,
            &mut self.entity.prevRotationPitch,
            EntityShulkerBullet::ROTATION_INTERPOLATION,
        );
        self.entity.rotationYaw = yaw;
        self.entity.rotationPitch = pitch;
        self.pendingParticleSpawns.push(ParticleSpawnRequest::new(
            EnumParticleTypes::EndRod,
            [self.entity.posX - self.entity.motionX, self.entity.posY - self.entity.motionY + 0.15, self.entity.posZ - self.entity.motionZ],
            [0.0, 0.0, 0.0],
            [0, 0],
        ));
    }

    fn updateArrow(&mut self, world: &WorldClient) {
        if self.arrowShake > 0 { self.arrowShake -= 1; }
        if self.inGround {
            self.ticksInGround = self.ticksInGround.saturating_add(1);
            return;
        }
        let start = Vec3d::new(self.entity.posX, self.entity.posY, self.entity.posZ);
        let end = Vec3d::new(
            self.entity.posX + self.entity.motionX,
            self.entity.posY + self.entity.motionY,
            self.entity.posZ + self.entity.motionZ,
        );
        if let Some(hit) = world.rayTraceBlocks(start, end, false, true, false) {
            let dx = hit.hitVec.x - self.entity.posX;
            let dy = hit.hitVec.y - self.entity.posY;
            let dz = hit.hitVec.z - self.entity.posZ;
            let length = (dx * dx + dy * dy + dz * dz).sqrt();
            if length > 1.0e-7 {
                self.entity.posX = hit.hitVec.x - dx / length * 0.05000000074505806;
                self.entity.posY = hit.hitVec.y - dy / length * 0.05000000074505806;
                self.entity.posZ = hit.hitVec.z - dz / length * 0.05000000074505806;
            }
            self.entity.motionX = dx;
            self.entity.motionY = dy;
            self.entity.motionZ = dz;
            self.inGround = true;
            self.arrowShake = 7;
            let (x, y, z) = (self.entity.posX, self.entity.posY, self.entity.posZ);
            self.entity.setPosition(x, y, z);
            return;
        }
        let x = self.entity.posX + self.entity.motionX;
        let y = self.entity.posY + self.entity.motionY;
        let z = self.entity.posZ + self.entity.motionZ;
        self.entity.posX = x;
        self.entity.posY = y;
        self.entity.posZ = z;
        self.updateProjectileRotation();
        self.entity.setPosition(x, y, z);
        let inWater = world.isMaterialInBB(
            self.entity.boundingBox,
            crate::net::minecraft::block::BlockLiquid::LiquidMaterial::Water,
        );
        let drag = if inWater { 0.6 } else { 0.99 };
        self.entity.motionX *= drag;
        self.entity.motionY *= drag;
        self.entity.motionZ *= drag;
        self.entity.motionY -= 0.05000000074505806;
    }

    fn updateFireball(&mut self, world: &WorldClient) {
        // EntityFireball#onUpdate first enters Entity#onEntityUpdate through
        // super.onUpdate; handleWaterMovement is the client-visible portion of
        // that base pass already owned by this shared Entity implementation.
        self.entity.handleWaterMovement(world);

        // EntityFireball#isFireballFiery is inherited by large/small fireballs
        // and overridden to false by dragon fireballs and wither skulls.
        let fiery = match &self.kind {
            ClientEntityKind::Object { objectType: ObjectSpawnType::LargeFireball, .. } => {
                EntityLargeFireball::FIERY
            }
            ClientEntityKind::Object { objectType: ObjectSpawnType::SmallFireball, .. } => {
                EntitySmallFireball::FIERY
            }
            ClientEntityKind::Object { objectType: ObjectSpawnType::DragonFireball, .. } => {
                EntityDragonFireball::FIERY
            }
            ClientEntityKind::Object { objectType: ObjectSpawnType::WitherSkull, .. } => {
                EntityWitherSkull::FIERY
            }
            _ => false,
        };
        if fiery && self.entity.fire < 20 {
            // Entity#setFire(1): fire = max(fire, 1 * 20). The shared client
            // entity path does not independently tick this field down yet, but
            // continually retaining 20 is equivalent while this fiery
            // projectile remains alive.
            self.entity.fire = 20;
        }

        // ProjectileHelper#forwardsRaycast and subclass onImpact are omitted on
        // the remote side only because every vanilla subclass impact mutation
        // is guarded by !world.isRemote; the authoritative server sends the
        // resulting removal, explosion, fire, damage or effect state.
        self.entity.posX += self.entity.motionX;
        self.entity.posY += self.entity.motionY;
        self.entity.posZ += self.entity.motionZ;
        let (yaw, pitch) = ProjectileHelper::rotateTowardsMovement(
            self.entity.motionX,
            self.entity.motionY,
            self.entity.motionZ,
            &mut self.entity.prevRotationYaw,
            &mut self.entity.prevRotationPitch,
            0.2,
        );
        self.entity.rotationYaw = yaw;
        self.entity.rotationPitch = pitch;

        let mut motionFactor = EntityFireball::DEFAULT_MOTION_FACTOR;
        if self.entity.isInWater() {
            for _ in 0..4 {
                self.pendingParticleSpawns.push(ParticleSpawnRequest::new(
                    EnumParticleTypes::WaterBubble,
                    [
                        self.entity.posX - self.entity.motionX * 0.25,
                        self.entity.posY - self.entity.motionY * 0.25,
                        self.entity.posZ - self.entity.motionZ * 0.25,
                    ],
                    [self.entity.motionX, self.entity.motionY, self.entity.motionZ],
                    [0, 0],
                ));
            }
            motionFactor = EntityFireball::WATER_MOTION_FACTOR;
        } else if matches!(
            &self.kind,
            ClientEntityKind::Object { objectType: ObjectSpawnType::WitherSkull, .. }
        ) {
            motionFactor = EntityWitherSkull::motionFactor(self.isWitherSkullInvulnerable());
        }

        self.entity.motionX += self.fireballAccelerationX;
        self.entity.motionY += self.fireballAccelerationY;
        self.entity.motionZ += self.fireballAccelerationZ;
        self.entity.motionX *= motionFactor;
        self.entity.motionY *= motionFactor;
        self.entity.motionZ *= motionFactor;
        let particleType = if matches!(
            &self.kind,
            ClientEntityKind::Object { objectType: ObjectSpawnType::DragonFireball, .. }
        ) {
            EnumParticleTypes::DragonBreath
        } else {
            EnumParticleTypes::SmokeNormal
        };
        self.pendingParticleSpawns.push(ParticleSpawnRequest::new(
            particleType,
            [self.entity.posX, self.entity.posY + 0.5, self.entity.posZ],
            [0.0, 0.0, 0.0],
            [0, 0],
        ));
        self.entity.setPosition(self.entity.posX, self.entity.posY, self.entity.posZ);
    }

    pub fn isWitherSkullInvulnerable(&self) -> bool {
        matches!(&self.kind, ClientEntityKind::Object { objectType: ObjectSpawnType::WitherSkull, .. })
            && self.dataManager.boolean(EntityWitherSkull::INVULNERABLE_DATA_INDEX, false)
    }

    fn updateFishHook(
        &mut self,
        world: &WorldClient,
        localPlayerEntityId: Option<i32>,
        localPlayerState: Option<([f64; 3], f32)>,
    ) {
        let Some(anglerId) = self.fishHookAnglerId else {
            self.entity.isDead = true;
            return;
        };
        let anglerPresent = localPlayerEntityId == Some(anglerId)
            || world.getEntityByID(anglerId).is_some();
        if !anglerPresent {
            self.entity.isDead = true;
            return;
        }

        if self.fishHookInGround {
            self.fishHookTicksInGround += 1;
            if self.fishHookTicksInGround >= EntityFishHook::MAX_GROUND_TICKS {
                self.entity.isDead = true;
                return;
            }
        }

        let blockPos = BlockPos::new(
            self.entity.posX.floor() as i32,
            self.entity.posY.floor() as i32,
            self.entity.posZ.floor() as i32,
        );
        let state = world.getBlockState(blockPos);
        let waterHeight = if LiquidMaterial::fromState(state) == Some(LiquidMaterial::Water) {
            BlockLiquid::getFilledPercentage(state, world, blockPos)
        } else { 0.0 };

        match self.fishHookState {
            FishHookState::Flying => {
                if self.fishHookCaughtEntityId.is_some() {
                    self.entity.motionX = 0.0;
                    self.entity.motionY = 0.0;
                    self.entity.motionZ = 0.0;
                    self.fishHookState = FishHookState::HookedInEntity;
                    return;
                }
                if waterHeight > 0.0 {
                    self.entity.motionX *= EntityFishHook::WATER_FLYING_XZ_FACTOR;
                    self.entity.motionY *= EntityFishHook::WATER_FLYING_Y_FACTOR;
                    self.entity.motionZ *= EntityFishHook::WATER_FLYING_XZ_FACTOR;
                    self.fishHookState = FishHookState::Bobbing;
                    return;
                }
                if !self.fishHookInGround && !self.entity.onGround && !self.entity.isCollidedHorizontally {
                    self.fishHookTicksInAir += 1;
                } else {
                    self.fishHookTicksInAir = 0;
                    self.entity.motionX = 0.0;
                    self.entity.motionY = 0.0;
                    self.entity.motionZ = 0.0;
                }
            }
            FishHookState::HookedInEntity => {
                let target = self.fishHookCaughtEntityId.and_then(|id| {
                    if localPlayerEntityId == Some(id) {
                        localPlayerState.map(|(position, height)| (position, height as f64))
                    } else {
                        world.getBaseEntityByID(id).map(|entity| {
                            ([entity.posX, entity.boundingBox.min_y, entity.posZ], entity.height as f64)
                        })
                    }
                });
                if let Some((position, height)) = target {
                    self.entity.setPosition(position[0], position[1] + height * 0.8, position[2]);
                } else {
                    self.fishHookCaughtEntityId = None;
                    self.fishHookState = FishHookState::Flying;
                }
                return;
            }
            FishHookState::Bobbing => {
                self.entity.motionX *= EntityFishHook::BOBBING_XZ_FACTOR;
                self.entity.motionZ *= EntityFishHook::BOBBING_XZ_FACTOR;
                let mut delta = self.entity.posY + self.entity.motionY
                    - blockPos.y as f64 - waterHeight as f64;
                if delta.abs() < 0.01 {
                    delta += delta.signum() * 0.1;
                }
                self.entity.motionY -= delta * self.fishHookRandom.next_f32() as f64 * 0.2;
            }
        }

        if LiquidMaterial::fromState(state) != Some(LiquidMaterial::Water) {
            self.entity.motionY -= EntityFishHook::GRAVITY;
        }
        self.entity.moveEntity(world, self.entity.motionX, self.entity.motionY, self.entity.motionZ);
        EntityFishHook::rotateTowardsMovement(
            &mut self.entity.prevRotationYaw,
            &mut self.entity.prevRotationPitch,
            &mut self.entity.rotationYaw,
            &mut self.entity.rotationPitch,
            [self.entity.motionX, self.entity.motionY, self.entity.motionZ],
        );
        self.entity.motionX *= EntityFishHook::DRAG;
        self.entity.motionY *= EntityFishHook::DRAG;
        self.entity.motionZ *= EntityFishHook::DRAG;
        self.entity.setPosition(self.entity.posX, self.entity.posY, self.entity.posZ);
    }

    fn updateAreaEffectCloud(&mut self) {
        // Remote branch of MCP `EntityAreaEffectCloud#onUpdate`. The entity has
        // no render geometry; every visible sample enters ParticleManager.
        self.entity.noClip = true;
        let ignoreRadius = self.areaEffectCloudIgnoresRadius();
        let radius = self.areaEffectCloudRadius();
        let particleType = self.areaEffectCloudParticle();
        let parameters = self.areaEffectCloudParticleParameters();
        let color = self.areaEffectCloudColor();
        if ignoreRadius {
            if self.areaEffectCloudRandom.next_bool() {
                for _ in 0..2 {
                    let angle = self.areaEffectCloudRandom.next_f32() * core::f32::consts::PI * 2.0;
                    let radial = self.areaEffectCloudRandom.next_f32().sqrt() * 0.2;
                    let offsetX = angle.cos() * radial;
                    let offsetZ = angle.sin() * radial;
                    let speed = if particleType == EnumParticleTypes::SpellMob {
                        let selected = if self.areaEffectCloudRandom.next_bool() { 0x00ff_ffff } else { color };
                        EntityAreaEffectCloud::colorComponents(selected)
                    } else {
                        [0.0, 0.0, 0.0]
                    };
                    self.pendingParticleSpawns.push(ParticleSpawnRequest::new(
                        particleType,
                        [self.entity.posX + offsetX as f64, self.entity.posY, self.entity.posZ + offsetZ as f64],
                        speed,
                        parameters,
                    ).withVisibility(false, true));
                }
            }
        } else {
            let count = EntityAreaEffectCloud::particleArea(radius).ceil().max(0.0) as usize;
            for _ in 0..count {
                let angle = self.areaEffectCloudRandom.next_f32() * core::f32::consts::PI * 2.0;
                let radial = self.areaEffectCloudRandom.next_f32().sqrt() * radius;
                let offsetX = angle.cos() * radial;
                let offsetZ = angle.sin() * radial;
                let speed = if particleType == EnumParticleTypes::SpellMob {
                    EntityAreaEffectCloud::colorComponents(color)
                } else {
                    [
                        (0.5 - self.areaEffectCloudRandom.next_f64()) * 0.15,
                        0.009999999776482582,
                        (0.5 - self.areaEffectCloudRandom.next_f64()) * 0.15,
                    ]
                };
                self.pendingParticleSpawns.push(ParticleSpawnRequest::new(
                    particleType,
                    [self.entity.posX + offsetX as f64, self.entity.posY, self.entity.posZ + offsetZ as f64],
                    speed,
                    parameters,
                ).withVisibility(false, true));
            }
        }
    }

    pub fn areaEffectCloudRadius(&self) -> f32 {
        self.dataManager.float(EntityAreaEffectCloud::RADIUS_INDEX, EntityAreaEffectCloud::DEFAULT_SYNC_RADIUS)
    }

    pub fn areaEffectCloudColor(&self) -> i32 {
        self.dataManager.varInt(EntityAreaEffectCloud::COLOR_INDEX, EntityAreaEffectCloud::DEFAULT_COLOR)
    }

    pub fn areaEffectCloudIgnoresRadius(&self) -> bool {
        self.dataManager.boolean(EntityAreaEffectCloud::IGNORE_RADIUS_INDEX, false)
    }

    pub fn areaEffectCloudParticle(&self) -> EnumParticleTypes {
        EnumParticleTypes::fromId(self.dataManager.varInt(
            EntityAreaEffectCloud::PARTICLE_INDEX,
            EntityAreaEffectCloud::DEFAULT_PARTICLE.particleId(),
        )).unwrap_or(EntityAreaEffectCloud::DEFAULT_PARTICLE)
    }

    pub fn areaEffectCloudParticleParameters(&self) -> [i32; 2] {
        [
            self.dataManager.varInt(EntityAreaEffectCloud::PARTICLE_PARAM_1_INDEX, 0),
            self.dataManager.varInt(EntityAreaEffectCloud::PARTICLE_PARAM_2_INDEX, 0),
        ]
    }

    pub fn takeParticleSpawns(&mut self) -> Vec<ParticleSpawnRequest> {
        core::mem::take(&mut self.pendingParticleSpawns)
    }

    fn updateProjectileRotation(&mut self) {
        let horizontal = (self.entity.motionX * self.entity.motionX + self.entity.motionZ * self.entity.motionZ).sqrt();
        let targetYaw = self.entity.motionX.atan2(self.entity.motionZ).to_degrees() as f32;
        let targetPitch = self.entity.motionY.atan2(horizontal).to_degrees() as f32;
        normalize_previous(targetYaw, &mut self.entity.prevRotationYaw);
        normalize_previous(targetPitch, &mut self.entity.prevRotationPitch);
        self.entity.rotationYaw = self.entity.prevRotationYaw + (targetYaw - self.entity.prevRotationYaw) * 0.2;
        self.entity.rotationPitch = self.entity.prevRotationPitch + (targetPitch - self.entity.prevRotationPitch) * 0.2;
    }

    pub fn isBurning(&self) -> bool {
        self.entity.fire > 0 || (self.dataManager.byte(0, 0) & 0x01) != 0
    }
}

/// Rust dynamic-world implementation of MCP `IJumpingMount` for the
/// `AbstractHorse` family represented by `EntityOtherClient`.
impl IJumpingMount for EntityOtherClient {
    fn setJumpPower(&mut self, jumpPowerIn: i32) {
        self.setHorseJumpPower(jumpPowerIn);
    }

    fn canJump(&self) -> bool {
        self.isHorseFamily() && self.horseSaddled()
    }
}

fn entity_size(kind: &ClientEntityKind) -> (f32, f32) {
    match kind {
        ClientEntityKind::Object { objectType, .. } => match objectType {
            ObjectSpawnType::Boat => (EntityBoat::WIDTH, EntityBoat::HEIGHT),
            ObjectSpawnType::FishHook => (EntityFishHook::WIDTH, EntityFishHook::HEIGHT),
            ObjectSpawnType::AreaEffectCloud => (
                EntityAreaEffectCloud::width(EntityAreaEffectCloud::DEFAULT_RADIUS),
                EntityAreaEffectCloud::DEFAULT_HEIGHT,
            ),
            ObjectSpawnType::Minecart => (EntityMinecart::WIDTH, EntityMinecart::HEIGHT),
            ObjectSpawnType::EnderCrystal => (EntityEnderCrystal::WIDTH, EntityEnderCrystal::HEIGHT),
            ObjectSpawnType::Item => (0.25, 0.25),
            ObjectSpawnType::FallingBlock | ObjectSpawnType::PrimedTnt => (0.98, 0.98),
            ObjectSpawnType::TippedArrow | ObjectSpawnType::SpectralArrow => (0.5, 0.5),
            ObjectSpawnType::Snowball
            | ObjectSpawnType::Egg
            | ObjectSpawnType::EnderPearl
            | ObjectSpawnType::EyeOfEnder
            | ObjectSpawnType::Potion
            | ObjectSpawnType::ExperienceBottle
            | ObjectSpawnType::FireworkRocket => (0.25, 0.25),
            ObjectSpawnType::ArmorStand => (0.5, 1.975),
            ObjectSpawnType::ItemFrame | ObjectSpawnType::LeashKnot => (EntityHanging::DEFAULT_WIDTH, EntityHanging::DEFAULT_HEIGHT),
            ObjectSpawnType::ShulkerBullet => (EntityShulkerBullet::WIDTH, EntityShulkerBullet::HEIGHT),
            ObjectSpawnType::LargeFireball => {
                (EntityLargeFireball::WIDTH, EntityLargeFireball::HEIGHT)
            },
            ObjectSpawnType::SmallFireball => {
                (EntitySmallFireball::WIDTH, EntitySmallFireball::HEIGHT)
            },
            ObjectSpawnType::DragonFireball => {
                (EntityDragonFireball::WIDTH, EntityDragonFireball::HEIGHT)
            },
            ObjectSpawnType::WitherSkull => {
                (EntityWitherSkull::WIDTH, EntityWitherSkull::HEIGHT)
            },
            _ => (0.6, 1.8),
        },
        ClientEntityKind::ExperienceOrb { .. } => (0.5, 0.5),
        ClientEntityKind::Mob { entityType } => match entityType.registryName {
            "zombie" | "husk" | "zombie_pigman" => (0.6, 1.95),
            "skeleton" | "stray" => (0.6, 1.99),
            "wither_skeleton" => (0.7, 2.4),
            "pig" => (0.9, 0.9),
            "sheep" => (0.9, 1.3),
            "cow" | "mooshroom" => (0.9, 1.4),
            "chicken" => (0.4, 0.7),
            "wolf" => (0.6, 0.85),
            "ocelot" => (0.6, 0.7),
            "rabbit" => (0.4, 0.5),
            "polar_bear" => (1.3, 1.4),
            "horse" | "donkey" | "mule" | "skeleton_horse" | "zombie_horse" => (1.3964844, 1.6),
            "llama" => (0.9, 1.87),
            "spider" => (1.4, 0.9),
            "cave_spider" => (0.7, 0.5),
            "enderman" => (0.6, 2.9),
            "squid" => (0.8, 0.8),
            "ender_dragon" => (16.0, 8.0),
            "creeper" => (0.6, 1.7),
            "ghast" => (4.0, 4.0),
            "guardian" => (0.85, 0.85),
            "elder_guardian" => (0.85 * 2.35, 0.85 * 2.35),
            "shulker" => (1.0, 1.0),
            "slime" | "magma_cube" => (0.51000005, 0.51000005),
            _ => (0.6, 1.8),
        },
        ClientEntityKind::Painting { .. } => (0.5, 0.5),
    }
}

fn fresh_random(entityId: i32) -> crate::compat::Java::JavaRandom {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let seed = time ^ sequence.rotate_left(21) ^ (entityId as u64).rotate_left(43);
    crate::compat::Java::JavaRandom::new(seed as i64)
}

fn next_gaussian_pair(random: &mut crate::compat::Java::JavaRandom) -> (f64, f64) {
    loop {
        let x = 2.0 * random.next_f64() - 1.0;
        let z = 2.0 * random.next_f64() - 1.0;
        let radiusSquared = x * x + z * z;
        if radiusSquared > 0.0 && radiusSquared < 1.0 {
            let multiplier = (-2.0 * radiusSquared.ln() / radiusSquared).sqrt();
            return (x * multiplier, z * multiplier);
        }
    }
}

fn hover_start(entityId: i32, _uniqueId: Option<Uuid>) -> f32 {
    let mut random = fresh_random(entityId);
    random.next_f32() * std::f32::consts::TAU
}

fn normalize_previous(current: f32, previous: &mut f32) {
    while current - *previous < -180.0 { *previous -= 360.0; }
    while current - *previous >= 180.0 { *previous += 360.0; }
}

fn fixed_position(value: f64) -> i64 { (value * 4096.0).floor() as i64 }

fn wrap_degrees_f32(mut value: f32) -> f32 {
    value %= 360.0;
    if value >= 180.0 { value -= 360.0; }
    if value < -180.0 { value += 360.0; }
    value
}

fn wrap_degrees_f64(mut value: f64) -> f64 {
    value %= 360.0;
    if value >= 180.0 { value -= 360.0; }
    if value < -180.0 { value += 360.0; }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mob_registry_matches_protocol_ids() {
        assert_eq!(MobEntityType::fromId(54).unwrap().registryName, "zombie");
        assert_eq!(MobEntityType::fromId(120).unwrap().registryName, "villager");
        assert!(MobEntityType::fromId(104).is_none()); // llama spit is an object packet entity
    }

    #[test]
    fn object_values_match_handle_spawn_object() {
        assert_eq!(ObjectSpawnType::fromPacketType(78), ObjectSpawnType::ArmorStand);
        assert_eq!(ObjectSpawnType::fromPacketType(90), ObjectSpawnType::FishHook);
    }

    #[test]
    fn enderman_squid_and_dragon_use_exact_source_dimensions() {
        for (id, expected) in [
            (58, (0.6, 2.9)),
            (94, (0.8, 0.8)),
            (63, (16.0, 8.0)),
        ] {
            let kind = ClientEntityKind::Mob { entityType: MobEntityType::fromId(id).unwrap() };
            assert_eq!(entity_size(&kind), expected);
            let entity = EntityOtherClient::new(id, None, kind, 0.0, 64.0, 0.0, 0.0, 0.0);
            assert_eq!((entity.entity.width, entity.entity.height), expected);
            if id == 58 {
                assert!((entity.eyeHeight() - 2.55).abs() < f32::EPSILON);
            } else if id == 94 {
                assert!((entity.eyeHeight() - 0.4).abs() < f32::EPSILON);
            }
        }
    }

    #[test]
    fn dead_dragon_stops_living_ring_buffer_but_runs_source_death_motion() {
        let world = WorldClient::new(1);
        let kind = ClientEntityKind::Mob { entityType: MobEntityType::fromId(63).unwrap() };
        let mut dragon = EntityOtherClient::new(63, None, kind, 4.0, 70.0, -3.0, 25.0, 0.0);
        dragon.dragonRingBufferIndex = 7;
        dragon.health = 0.0;
        let old_y = dragon.entity.posY;
        let old_yaw = dragon.entity.rotationYaw;
        dragon.onUpdate(&world, None);
        assert_eq!(dragon.dragonRingBufferIndex, 7);
        assert_eq!(dragon.dragonDeathTicks, 1);
        assert!((dragon.entity.posY - (old_y + 0.10000000149011612)).abs() < 1.0e-12);
        assert!((dragon.entity.rotationYaw - (old_yaw + 20.0)).abs() < 1.0e-5);
    }

    #[test]
    fn dropped_item_advances_locally_between_server_packets() {
        let world = WorldClient::new(0);
        let mut entity = EntityOtherClient::new(
            7,
            None,
            ClientEntityKind::Object {
                objectType: ObjectSpawnType::Item,
                data: 1,
                spawnVelocity: [0.3, 0.1, 0.0],
            },
            0.0,
            64.0,
            0.0,
            0.0,
            0.0,
        );
        entity.setVelocity(0.3, 0.1, 0.0);
        entity.onUpdate(&world, None);
        assert!((entity.entity.posX - 0.3).abs() < 1.0e-9);
        assert!((entity.entity.prevPosX - 0.0).abs() < 1.0e-9);
        assert!(entity.entity.posY > 64.0);
    }

    #[test]
    fn throwable_advances_and_preserves_previous_position_for_render_interpolation() {
        let world = WorldClient::new(0);
        let mut entity = EntityOtherClient::new(
            8,
            None,
            ClientEntityKind::Object {
                objectType: ObjectSpawnType::Snowball,
                data: 0,
                spawnVelocity: [0.5, 0.0, 0.0],
            },
            1.0,
            64.0,
            1.0,
            0.0,
            0.0,
        );
        entity.setVelocity(0.5, 0.0, 0.0);
        entity.onUpdate(&world, None);
        assert_eq!(entity.entity.prevPosX, 1.0);
        assert_eq!(entity.entity.posX, 1.5);
        assert!(entity.entity.motionY < 0.0);
    }

    fn boat_entity() -> EntityOtherClient {
        EntityOtherClient::new(41, None, ClientEntityKind::Object {
            objectType: ObjectSpawnType::Boat,
            data: 0,
            spawnVelocity: [0.0; 3],
        }, 0.0, 64.0, 0.0, 0.0, 0.0)
    }

    fn minecart_entity(data: i32) -> EntityOtherClient {
        EntityOtherClient::new(42, None, ClientEntityKind::Object {
            objectType: ObjectSpawnType::Minecart,
            data,
            spawnVelocity: [0.0; 3],
        }, 0.0, 64.0, 0.0, 0.0, 0.0)
    }

    fn horse_entity() -> EntityOtherClient {
        EntityOtherClient::new(100, None, ClientEntityKind::Mob {
            entityType: MobEntityType::fromId(100).unwrap(),
        }, 0.0, 64.0, 0.0, 0.0, 0.0)
    }

    #[test]
    fn abstract_horse_jump_power_matches_mcp_piecewise_formula() {
        let mut horse = horse_entity();
        horse.applyMetadata([(13, DataValue::Byte(4))]);
        horse.entity.setPassengers(vec![7]);

        horse.setHorseJumpPower(-1);
        assert!((horse.horseJumpPower - 0.4).abs() < 1.0e-6);
        assert!(!horse.horseRearing());

        horse.setHorseJumpPower(45);
        assert!((horse.horseJumpPower - 0.6).abs() < 1.0e-6);
        assert!(horse.horseAllowStandSliding);
        assert!(horse.horseRearing());
        assert_eq!(horse.horseJumpRearingCounter, 1);

        horse.setHorseJumpPower(90);
        assert_eq!(horse.horseJumpPower, 1.0);
    }

    #[test]
    fn boat_metadata_dimensions_ten_step_lerp_and_paddles_match_mcp() {
        let world = WorldClient::new(0);
        let mut entity = boat_entity();
        assert_eq!((entity.entity.width, entity.entity.height), (EntityBoat::WIDTH, EntityBoat::HEIGHT));
        assert_eq!(entity.boatType(), BoatType::Oak);
        assert_eq!(entity.boatForwardDirection(), 1);

        entity.applyMetadata([
            (6, DataValue::VarInt(4)),
            (8, DataValue::Float(7.0)),
            (9, DataValue::VarInt(5)),
            (10, DataValue::Boolean(true)),
            (11, DataValue::Boolean(false)),
        ]);
        entity.entity.setPassengers(vec![99]);
        entity.setPositionAndRotationDirect(10.0, 65.0, -5.0, 90.0, 20.0, 2, false);
        assert_eq!(entity.newPosRotationIncrements, EntityBoat::LERP_STEPS);
        entity.onUpdate(&world, None);

        assert!((entity.entity.posX - 1.0).abs() < 1.0e-9);
        assert!((entity.entity.posY - 64.1).abs() < 1.0e-9);
        assert!((entity.entity.posZ + 0.5).abs() < 1.0e-9);
        assert!((entity.entity.rotationYaw - 9.0).abs() < 1.0e-6);
        assert!((entity.entity.rotationPitch - 2.0).abs() < 1.0e-6);
        assert_eq!(entity.boatTimeSinceHit(), 3);
        assert_eq!(entity.boatDamageTaken(), 6.0);
        assert_eq!(entity.boatType(), BoatType::DarkOak);
        assert!((entity.boatPaddlePositions[0] - EntityBoat::PADDLE_STEP).abs() < 1.0e-6);
        assert_eq!(entity.boatPaddlePositions[1], 0.0);
        assert!((entity.boatRowingTime(0, 0.5) - EntityBoat::PADDLE_STEP * 0.5).abs() < 1.0e-6);
    }

    #[test]
    fn minecart_subclass_defaults_explicit_state_bridge_and_packet_lerp_match_mcp() {
        let mut entity = minecart_entity(1);
        assert_eq!((entity.entity.width, entity.entity.height), (EntityMinecart::WIDTH, EntityMinecart::HEIGHT));
        assert_eq!(entity.minecartType(), MinecartType::Chest);
        assert_eq!(entity.minecartDisplayStateId(), (54 << 4) | 2);
        assert_eq!(entity.minecartDisplayOffset(), 8);

        entity.applyMetadata([
            (9, DataValue::VarInt(54 | (2 << 12))),
            (10, DataValue::VarInt(12)),
            (11, DataValue::Boolean(true)),
        ]);
        assert_eq!(entity.minecartDisplayStateId(), (54 << 4) | 2);
        assert_eq!(entity.minecartDisplayOffset(), 12);

        entity.setVelocity(0.25, -0.5, 0.75);
        entity.setPositionAndRotationDirect(5.0, 66.0, 4.0, 45.0, 10.0, 3, false);
        assert_eq!(entity.newPosRotationIncrements, 5);
        assert_eq!([entity.entity.motionX, entity.entity.motionY, entity.entity.motionZ], [0.25, -0.5, 0.75]);
    }

    #[test]
    fn living_hurt_status_opcodes_drive_common_hurt_and_death_timers() {
        let mut entity = EntityOtherClient::new(
            54,
            None,
            ClientEntityKind::Mob { entityType: MobEntityType::fromId(54).unwrap() },
            0.0,
            64.0,
            0.0,
            0.0,
            0.0,
        );
        for opcode in [2_i8, 33, 36, 37] {
            entity.hurtTime = 0;
            entity.hurtResistantTime = 0;
            entity.handleStatusUpdate(opcode);
            assert_eq!(entity.maxHurtTime, 10);
            assert_eq!(entity.hurtTime, 10);
            assert_eq!(entity.hurtResistantTime, entity.maxHurtResistantTime);
        }
        entity.health = 20.0;
        entity.handleStatusUpdate(3);
        assert_eq!(entity.health, 0.0);
    }

    #[test]
    fn tnt_minecart_status_starts_eighty_tick_client_fuse() {
        let world = WorldClient::new(0);
        let mut entity = minecart_entity(3);
        assert_eq!(entity.minecartType(), MinecartType::Tnt);
        assert_eq!(entity.minecartTntFuse(), -1);
        entity.handleStatusUpdate(10);
        assert_eq!(entity.minecartTntFuse(), 80);
        entity.onUpdate(&world, None);
        assert_eq!(entity.minecartTntFuse(), 79);
    }

    fn shulker_entity() -> EntityOtherClient {
        EntityOtherClient::new(69, None, ClientEntityKind::Mob {
            entityType: MobEntityType::fromId(69).unwrap(),
        }, 0.0, 64.0, 0.0, 0.0, 0.0)
    }

    #[test]
    fn shulker_metadata_owns_attachment_position_peek_and_color() {
        let world = WorldClient::new(0);
        let mut entity = shulker_entity();
        entity.applyMetadata([
            (12, DataValue::Facing(EnumFacing::Down.index())),
            (13, DataValue::OptionalBlockPos(Some(BlockPos::new(3, 70, -2)))),
            (14, DataValue::Byte(100)),
            (15, DataValue::Byte(10)),
        ]);
        assert_eq!(entity.shulkerAttachmentPos(), Some(BlockPos::new(3, 70, -2)));
        assert_eq!(entity.shulkerColorMetadata(), 10);
        assert_eq!([entity.entity.posX, entity.entity.posY, entity.entity.posZ], [3.5, 70.0, -1.5]);
        assert_eq!(entity.shulkerOldAttachPos(), Some(BlockPos::new(3, 70, -2)));
        entity.onUpdate(&world, None);
        assert!((entity.shulkerPeekAmount - 0.05).abs() < 1.0e-6);
        assert_eq!(entity.entity.motionX, 0.0);
        assert_eq!(entity.renderYawOffset, 180.0);
        assert!(entity.entity.boundingBox.max_y > 71.0);
    }

    #[test]
    fn shulker_attachment_change_starts_exact_six_tick_client_interpolation() {
        let mut entity = shulker_entity();
        entity.applyMetadata([(13, DataValue::OptionalBlockPos(Some(BlockPos::new(1, 2, 3))))]);
        entity.applyMetadata([(13, DataValue::OptionalBlockPos(Some(BlockPos::new(5, 6, 7))))]);
        assert_eq!(entity.shulkerOldAttachPos(), Some(BlockPos::new(1, 2, 3)));
        assert_eq!(entity.shulkerAttachmentPos(), Some(BlockPos::new(5, 6, 7)));
        assert_eq!(entity.shulkerClientTeleportInterp(), 6);
    }

    #[test]
    fn shulker_bullet_uses_source_size_no_clip_motion_and_half_rotation_lerp() {
        let world = WorldClient::new(0);
        let mut entity = EntityOtherClient::new(67, None, ClientEntityKind::Object {
            objectType: ObjectSpawnType::ShulkerBullet,
            data: 0,
            spawnVelocity: [0.0; 3],
        }, 1.0, 2.0, 3.0, 0.0, 0.0);
        assert_eq!((entity.entity.width, entity.entity.height), (EntityShulkerBullet::WIDTH, EntityShulkerBullet::HEIGHT));
        assert!(entity.entity.noClip);
        entity.setVelocity(1.0, 0.0, 0.0);
        entity.onUpdate(&world, None);
        assert_eq!([entity.entity.posX, entity.entity.posY, entity.entity.posZ], [2.0, 2.0, 3.0]);
        assert!((entity.entity.rotationYaw - 45.0).abs() < 1.0e-5);
        assert!(entity.entity.rotationPitch.abs() < 1.0e-5);
    }

    #[test]
    fn first_renderer_family_uses_source_entity_dimensions() {
        assert_eq!(entity_size(&ClientEntityKind::Object {
            objectType: ObjectSpawnType::Item, data: 0, spawnVelocity: [0.0; 3],
        }), (0.25, 0.25));
        assert_eq!(entity_size(&ClientEntityKind::Object {
            objectType: ObjectSpawnType::FallingBlock, data: 0, spawnVelocity: [0.0; 3],
        }), (0.98, 0.98));
        assert_eq!(entity_size(&ClientEntityKind::Object {
            objectType: ObjectSpawnType::TippedArrow, data: 0, spawnVelocity: [0.0; 3],
        }), (0.5, 0.5));
        assert_eq!(entity_size(&ClientEntityKind::ExperienceOrb { xpValue: 1 }), (0.5, 0.5));
    }
    #[test]
    fn hanging_entities_use_source_anchor_facing_and_metadata_geometry() {
        let painting = EntityOtherClient::new(
            200,
            None,
            ClientEntityKind::Painting {
                title: "Pool".to_owned(),
                hangingPosition: BlockPos::new(10, 64, 20),
                facing: EnumFacing::North,
            },
            10.0, 64.0, 20.0, 0.0, 0.0,
        );
        assert_eq!(painting.paintingArt(), Some(PaintingArt::Pool));
        assert_eq!(painting.hangingPosition, Some(BlockPos::new(10, 64, 20)));
        assert_eq!(painting.hangingFacing, Some(EnumFacing::North));
        assert_eq!(painting.entity.rotationYaw, 180.0);
        assert!((painting.entity.boundingBox.max_x - painting.entity.boundingBox.min_x - 2.0).abs() < 1.0e-9);

        let frame = EntityOtherClient::new(
            201,
            None,
            ClientEntityKind::Object {
                objectType: ObjectSpawnType::ItemFrame,
                data: 3,
                spawnVelocity: [0.0; 3],
            },
            4.0, 70.0, -2.0, 0.0, 0.0,
        );
        assert_eq!(frame.hangingFacing, Some(EnumFacing::East));
        assert_eq!(frame.itemFrameRotation(), 0);
        assert!(frame.itemFrameDisplayedItem().is_none());
        assert_eq!((frame.entity.width, frame.entity.height), (0.5, 0.5));

        let knot = EntityOtherClient::new(
            202,
            None,
            ClientEntityKind::Object {
                objectType: ObjectSpawnType::LeashKnot,
                data: 0,
                spawnVelocity: [0.0; 3],
            },
            1.2, 65.9, -3.1, 0.0, 0.0,
        );
        assert_eq!([knot.entity.posX, knot.entity.posY, knot.entity.posZ], [1.5, 65.5, -3.5]);
        assert_eq!(knot.eyeHeight(), EntityLeashKnot::EYE_HEIGHT);
    }

}
