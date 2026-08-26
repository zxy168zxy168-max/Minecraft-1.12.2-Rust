use uuid::Uuid;

use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockBed::BlockBed;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::util::math::BlockPos::BlockPos;

use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::net::minecraft::client::entity::AbstractClientPlayer::AbstractClientPlayer;
use crate::net::minecraft::client::network::NetworkPlayerInfo::NetworkPlayerInfo;
use crate::net::minecraft::entity::ai::attributes::AbstractAttributeMap::AbstractAttributeMap;
use crate::net::minecraft::entity::Entity::Entity;
use crate::net::minecraft::inventory::EntityEquipment::EntityEquipment;
use crate::net::minecraft::inventory::EntityEquipmentSlot::EntityEquipmentSlot;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::datasync::DataSerializers::DataValue;
use crate::net::minecraft::network::datasync::EntityDataManager::EntityDataManager;
use crate::net::minecraft::util::EnumHand::EnumHand;
use crate::net::minecraft::util::EnumHandSide::EnumHandSide;

/// Network-controlled remote player. Position interpolation, body/head rotation,
/// limb movement and arm swing follow MCP 1.12.2 `EntityOtherPlayerMP` and the
/// inherited `EntityLivingBase` update path. Server positions remain fixed-point
/// 1/4096 values as used by `NetHandlerPlayClient.handleEntityMovement`.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityOtherPlayerMP {
    pub entity: Entity,
    pub entityId: i32,
    pub uniqueId: Uuid,
    pub gameProfile: GameProfile,
    /// Exact MCP superclass responsibility: profile plus the retained
    /// `AbstractClientPlayer#playerInfo` texture owner.
    pub abstractClientPlayer: AbstractClientPlayer,
    pub serverPosX: i64,
    pub serverPosY: i64,
    pub serverPosZ: i64,
    pub renderYawOffset: f32,
    pub prevRenderYawOffset: f32,
    pub rotationYawHead: f32,
    pub prevRotationYawHead: f32,
    pub limbSwing: f32,
    pub limbSwingAmount: f32,
    pub prevLimbSwingAmount: f32,
    pub swingProgress: f32,
    pub prevSwingProgress: f32,
    pub movedDistance: f32,
    pub prevMovedDistance: f32,
    pub chasingPosX: f64,
    pub chasingPosY: f64,
    pub chasingPosZ: f64,
    pub prevChasingPosX: f64,
    pub prevChasingPosY: f64,
    pub prevChasingPosZ: f64,
    pub onGroundSpeedFactor: f32,
    pub prevOnGroundSpeedFactor: f32,
    pub cameraYaw: f32,
    pub prevCameraYaw: f32,
    pub cameraPitch: f32,
    /// MCP `EntityLivingBase.ticksElytraFlying`.
    pub ticksElytraFlying: i32,
    pub equipment: EntityEquipment,
    pub health: f32,
    pub hurtTime: i32,
    pub maxHurtTime: i32,
    pub deathTime: i32,
    pub hurtResistantTime: i32,
    pub maxHurtResistantTime: i32,
    pub attackedAtYaw: f32,
    pub sleeping: bool,
    pub sleepTimer: i32,
    pub bedLocation: Option<BlockPos>,
    pub renderOffsetX: f32,
    pub renderOffsetY: f32,
    pub renderOffsetZ: f32,
    activeItemStack: ItemStack,
    activeItemStackUseCount: i32,
    activeItemEquipmentRevision: Option<u64>,
    mainHandEquipmentRevision: u64,
    offHandEquipmentRevision: u64,
    pub lastStatusOpcode: Option<i8>,
    swingProgressInt: i32,
    isSwingInProgress: bool,
    swingingHand: EnumHand,
    primaryHand: EnumHandSide,
    otherPlayerMPPosRotationIncrements: i32,
    otherPlayerMPX: f64,
    otherPlayerMPY: f64,
    otherPlayerMPZ: f64,
    otherPlayerMPYaw: f64,
    otherPlayerMPPitch: f64,
    pub dataManager: EntityDataManager,
    pub attributeMap: AbstractAttributeMap,
}

fn remote_player_attribute_map() -> AbstractAttributeMap {
    let mut map = AbstractAttributeMap::default();
    map.registerAttribute("generic.maxHealth", 20.0);
    map.registerAttribute("generic.movementSpeed", 0.10000000149011612);
    map.registerAttribute("generic.attackDamage", 1.0);
    map.registerAttribute("generic.attackSpeed", 4.0);
    map.registerAttribute("generic.luck", 0.0);
    map
}

impl EntityOtherPlayerMP {
    pub fn new(entityId: i32, uniqueId: Uuid, gameProfile: GameProfile) -> Self {
        let mut entity = Entity::default();
        entity.stepHeight = 1.0;
        entity.noClip = true;
        Self {
            entity,
            entityId,
            uniqueId,
            abstractClientPlayer: AbstractClientPlayer::new(gameProfile.clone()),
            gameProfile,
            serverPosX: 0,
            serverPosY: 0,
            serverPosZ: 0,
            renderYawOffset: 0.0,
            prevRenderYawOffset: 0.0,
            rotationYawHead: 0.0,
            prevRotationYawHead: 0.0,
            limbSwing: 0.0,
            limbSwingAmount: 0.0,
            prevLimbSwingAmount: 0.0,
            swingProgress: 0.0,
            prevSwingProgress: 0.0,
            movedDistance: 0.0,
            prevMovedDistance: 0.0,
            chasingPosX: 0.0,
            chasingPosY: 0.0,
            chasingPosZ: 0.0,
            prevChasingPosX: 0.0,
            prevChasingPosY: 0.0,
            prevChasingPosZ: 0.0,
            onGroundSpeedFactor: 0.0,
            prevOnGroundSpeedFactor: 0.0,
            cameraYaw: 0.0,
            prevCameraYaw: 0.0,
            cameraPitch: 0.0,
            ticksElytraFlying: 0,
            equipment: EntityEquipment::default(),
            health: 20.0,
            hurtTime: 0,
            maxHurtTime: 0,
            deathTime: 0,
            hurtResistantTime: 0,
            maxHurtResistantTime: 20,
            attackedAtYaw: 0.0,
            sleeping: false,
            sleepTimer: 0,
            bedLocation: None,
            renderOffsetX: 0.0,
            renderOffsetY: 0.0,
            renderOffsetZ: 0.0,
            activeItemStack: ItemStack::EMPTY,
            activeItemStackUseCount: 0,
            activeItemEquipmentRevision: None,
            mainHandEquipmentRevision: 0,
            offHandEquipmentRevision: 0,
            lastStatusOpcode: None,
            swingProgressInt: 0,
            isSwingInProgress: false,
            swingingHand: EnumHand::MainHand,
            primaryHand: EnumHandSide::Right,
            otherPlayerMPPosRotationIncrements: 0,
            otherPlayerMPX: 0.0,
            otherPlayerMPY: 0.0,
            otherPlayerMPZ: 0.0,
            otherPlayerMPYaw: 0.0,
            otherPlayerMPPitch: 0.0,
            dataManager: EntityDataManager::default(),
            attributeMap: remote_player_attribute_map(),
        }
    }

    pub fn setPlayerInfo(&mut self, playerInfo: Option<NetworkPlayerInfo>) {
        self.abstractClientPlayer.setPlayerInfo(playerInfo);
    }

    pub fn getPlayerInfo(&self) -> Option<&NetworkPlayerInfo> {
        self.abstractClientPlayer.getPlayerInfo()
    }

    pub fn isElytraFlying(&self) -> bool {
        (self.dataManager.byte(0, 0) & 0x80_u8 as i8) != 0
    }

    pub fn setServerPosition(&mut self, x: f64, y: f64, z: f64) {
        // MCP `EntityTracker.getPositionLong` uses floor, not nearest rounding.
        self.serverPosX = (x * 4096.0).floor() as i64;
        self.serverPosY = (y * 4096.0).floor() as i64;
        self.serverPosZ = (z * 4096.0).floor() as i64;
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
        self.otherPlayerMPX = x;
        self.otherPlayerMPY = y;
        self.otherPlayerMPZ = z;
        self.otherPlayerMPYaw = yaw as f64;
        self.otherPlayerMPPitch = pitch as f64;
        self.otherPlayerMPPosRotationIncrements = increments;
    }

    pub fn setVelocity(&mut self, x: f64, y: f64, z: f64) {
        self.entity.motionX = x;
        self.entity.motionY = y;
        self.entity.motionZ = z;
    }

    pub fn setRotationYawHead(&mut self, yaw: f32) {
        self.rotationYawHead = yaw;
    }

    pub fn applyMetadata(&mut self, entries: impl IntoIterator<Item = (u8, DataValue)>) {
        let previousHandStates = self.dataManager.byte(6, 0);
        self.dataManager.setEntryValues(entries);
        self.entity.sneaking = (self.dataManager.byte(0, 0) & 0x02) != 0;
        self.primaryHand = EnumHandSide::byId(self.dataManager.byte(14, 1) as i32);

        // EntityLivingBase#notifyDataManagerChange(HAND_STATES). Index 6 is
        // the inherited HAND_STATES DataParameter for players in 1.12.2.
        let handStates = self.dataManager.byte(6, 0);
        if handStates != previousHandStates {
            if self.isHandActive() && self.activeItemStack.isEmpty() {
                let activeHand = self.getActiveHand();
                self.activeItemStack = self.getHeldItem(activeHand).clone();
                if !self.activeItemStack.isEmpty() {
                    self.activeItemStackUseCount = self.activeItemStack.getMaxItemUseDuration();
                    self.activeItemEquipmentRevision = Some(self.equipmentRevision(activeHand));
                }
            } else if !self.isHandActive() && !self.activeItemStack.isEmpty() {
                self.resetActiveHandClient();
            }
        }
    }

    pub fn trySleepClient(&mut self, bedState: IBlockState, bedLocation: BlockPos) {
        self.entity.ridingEntityId = None;
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

    pub fn swingArm(&mut self, hand: EnumHand) {
        if !self.isSwingInProgress || self.swingProgressInt >= 3 {
            self.swingProgressInt = -1;
            self.isSwingInProgress = true;
            self.swingingHand = hand;
        }
    }

    pub fn onUpdate(&mut self) {
        self.entity.prevPosX = self.entity.posX;
        self.entity.prevPosY = self.entity.posY;
        self.entity.prevPosZ = self.entity.posZ;
        self.entity.prevRotationYaw = self.entity.rotationYaw;
        self.entity.prevRotationPitch = self.entity.rotationPitch;
        self.prevRenderYawOffset = self.renderYawOffset;
        self.prevRotationYawHead = self.rotationYawHead;
        self.prevSwingProgress = self.swingProgress;
        if self.sleeping {
            self.sleepTimer = (self.sleepTimer + 1).min(100);
            self.entity.motionX = 0.0;
            self.entity.motionY = 0.0;
            self.entity.motionZ = 0.0;
        } else if self.sleepTimer > 0 {
            self.sleepTimer += 1;
            if self.sleepTimer >= 110 {
                self.sleepTimer = 0;
            }
        }
        if self.hurtTime > 0 {
            self.hurtTime -= 1;
        }
        if self.hurtResistantTime > 0 {
            self.hurtResistantTime -= 1;
        }
        if self.health <= 0.0 {
            self.deathTime = self.deathTime.saturating_add(1);
            if self.deathTime >= 20 {
                self.entity.isDead = true;
            }
        } else {
            self.deathTime = 0;
        }

        // EntityLivingBase#onUpdate decrements active use before dispatching
        // the virtual onLivingUpdate implementation. Sleeping players keep
        // their authoritative bed position and do not run normal movement.
        self.updateActiveHand();
        if !self.sleeping {
            self.onLivingUpdate();
        }

        let dx = self.entity.posX - self.entity.prevPosX;
        let dz = self.entity.posZ - self.entity.prevPosZ;
        let horizontalSquared = (dx * dx + dz * dz) as f32;
        let mut desiredBodyYaw = self.renderYawOffset;
        let mut movedDistance = 0.0_f32;
        self.prevOnGroundSpeedFactor = self.onGroundSpeedFactor;
        let mut onGroundSpeedTarget = 0.0_f32;

        if horizontalSquared > 0.0025000002 {
            onGroundSpeedTarget = 1.0;
            movedDistance = horizontalSquared.sqrt() * 3.0;
            let movementYaw = (dz.atan2(dx) as f32).to_degrees() - 90.0;
            // Keep the original 1.12.2 expression. It deliberately wraps the
            // entity yaw before subtracting the movement yaw rather than
            // wrapping the completed subtraction.
            let difference = (wrap_degrees_f32(self.entity.rotationYaw) - movementYaw).abs();
            desiredBodyYaw = if difference > 95.0 && difference < 265.0 {
                movementYaw - 180.0
            } else {
                movementYaw
            };
        }

        if self.swingProgress > 0.0 {
            desiredBodyYaw = self.entity.rotationYaw;
        }

        if !self.entity.onGround {
            onGroundSpeedTarget = 0.0;
        }

        self.onGroundSpeedFactor += (onGroundSpeedTarget - self.onGroundSpeedFactor) * 0.3;
        movedDistance = self.updateDistance(desiredBodyYaw, movedDistance);
        self.prevMovedDistance = self.movedDistance;
        self.movedDistance += movedDistance;

        normalize_previous_angle(self.entity.rotationYaw, &mut self.entity.prevRotationYaw);
        normalize_previous_angle(self.renderYawOffset, &mut self.prevRenderYawOffset);
        normalize_previous_angle(
            self.entity.rotationPitch,
            &mut self.entity.prevRotationPitch,
        );
        normalize_previous_angle(self.rotationYawHead, &mut self.prevRotationYawHead);
        // Inherited EntityLivingBase#onUpdate advances this after body-yaw
        // normalization and follows synchronized flag 7 directly on clients.
        self.ticksElytraFlying = if self.isElytraFlying() {
            self.ticksElytraFlying.saturating_add(1)
        } else {
            0
        };

        // EntityOtherPlayerMP performs this additional limb-distance update
        // after the inherited EntityLivingBase update has completed.
        self.prevLimbSwingAmount = self.limbSwingAmount;
        let mut amount = ((dx * dx + dz * dz).sqrt() as f32) * 4.0;
        if amount > 1.0 {
            amount = 1.0;
        }
        self.limbSwingAmount += (amount - self.limbSwingAmount) * 0.4;
        self.limbSwing += self.limbSwingAmount;

        // EntityPlayer#onUpdate updates the trailing cape point only after the
        // complete EntityLivingBase update, including body yaw and walked
        // distance. EntityOtherPlayerMP shares that inherited ordering.
        self.updateCape();
    }

    /// MCP `EntityOtherPlayerMP.onLivingUpdate` followed by the inherited
    /// `EntityLivingBase.updateArmSwingProgress` timing used before body-yaw
    /// selection in `EntityLivingBase.onUpdate`.
    fn onLivingUpdate(&mut self) {
        if self.otherPlayerMPPosRotationIncrements > 0 {
            let increments = self.otherPlayerMPPosRotationIncrements as f64;
            let x = self.entity.posX + (self.otherPlayerMPX - self.entity.posX) / increments;
            let y = self.entity.posY + (self.otherPlayerMPY - self.entity.posY) / increments;
            let z = self.entity.posZ + (self.otherPlayerMPZ - self.entity.posZ) / increments;
            let yawDifference =
                wrap_degrees_f64(self.otherPlayerMPYaw - self.entity.rotationYaw as f64);
            self.entity.rotationYaw =
                (self.entity.rotationYaw as f64 + yawDifference / increments) as f32;
            self.entity.rotationPitch = (self.entity.rotationPitch as f64
                + (self.otherPlayerMPPitch - self.entity.rotationPitch as f64) / increments)
                as f32;
            self.otherPlayerMPPosRotationIncrements -= 1;
            self.entity.setPosition(x, y, z);
            self.entity.rotationYaw %= 360.0;
            self.entity.rotationPitch %= 360.0;
        }

        self.prevCameraYaw = self.cameraYaw;
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

        let mut horizontalMotion = (self.entity.motionX * self.entity.motionX
            + self.entity.motionZ * self.entity.motionZ)
            .sqrt() as f32;
        let mut verticalCamera =
            (-self.entity.motionY * 0.20000000298023224_f64).atan() as f32 * 15.0;
        if horizontalMotion > 0.1 {
            horizontalMotion = 0.1;
        }
        if !self.entity.onGround {
            horizontalMotion = 0.0;
        }
        if self.entity.onGround {
            verticalCamera = 0.0;
        }
        self.cameraYaw += (horizontalMotion - self.cameraYaw) * 0.4;
        self.cameraPitch += (verticalCamera - self.cameraPitch) * 0.8;
    }

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

    fn updateDistance(&mut self, targetYaw: f32, mut distance: f32) -> f32 {
        let delta = wrap_degrees_f32(targetYaw - self.renderYawOffset);
        self.renderYawOffset += delta * 0.3;
        let mut relative = wrap_degrees_f32(self.entity.rotationYaw - self.renderYawOffset);
        let backwards = !(-90.0..90.0).contains(&relative);
        relative = relative.clamp(-75.0, 75.0);
        self.renderYawOffset = self.entity.rotationYaw - relative;
        if relative * relative > 2500.0 {
            self.renderYawOffset += relative * 0.2;
        }
        if backwards {
            distance *= -1.0;
        }
        distance
    }

    pub fn setItemStackToSlot(&mut self, slot: EntityEquipmentSlot, stack: ItemStack) {
        // Java keeps object identity for the active ItemStack. A new equipment
        // packet replaces the slot object even when its value is identical, so
        // preserve that identity boundary with per-hand revisions in Rust.
        match slot {
            EntityEquipmentSlot::Mainhand => {
                self.mainHandEquipmentRevision = self.mainHandEquipmentRevision.wrapping_add(1);
            }
            EntityEquipmentSlot::Offhand => {
                self.offHandEquipmentRevision = self.offHandEquipmentRevision.wrapping_add(1);
            }
            EntityEquipmentSlot::Feet
            | EntityEquipmentSlot::Legs
            | EntityEquipmentSlot::Chest
            | EntityEquipmentSlot::Head => {}
        }
        self.equipment.setItemStackToSlot(slot, stack);
    }

    pub fn getHeldItemMainhand(&self) -> &ItemStack {
        self.equipment
            .getItemStackFromSlot(EntityEquipmentSlot::Mainhand)
    }

    pub fn getHeldItemOffhand(&self) -> &ItemStack {
        self.equipment
            .getItemStackFromSlot(EntityEquipmentSlot::Offhand)
    }

    pub fn getHeldItem(&self, hand: EnumHand) -> &ItemStack {
        match hand {
            EnumHand::MainHand => self.getHeldItemMainhand(),
            EnumHand::OffHand => self.getHeldItemOffhand(),
        }
    }

    pub const fn getPrimaryHand(&self) -> EnumHandSide {
        self.primaryHand
    }

    pub fn isHandActive(&self) -> bool {
        (self.dataManager.byte(6, 0) & 1) != 0
    }

    pub fn getActiveHand(&self) -> EnumHand {
        if (self.dataManager.byte(6, 0) & 2) != 0 {
            EnumHand::OffHand
        } else {
            EnumHand::MainHand
        }
    }

    pub const fn getItemInUseCount(&self) -> i32 {
        self.activeItemStackUseCount
    }

    fn equipmentRevision(&self, hand: EnumHand) -> u64 {
        match hand {
            EnumHand::MainHand => self.mainHandEquipmentRevision,
            EnumHand::OffHand => self.offHandEquipmentRevision,
        }
    }

    fn updateActiveHand(&mut self) {
        if !self.isHandActive() {
            return;
        }
        let activeHand = self.getActiveHand();
        if self.activeItemEquipmentRevision == Some(self.equipmentRevision(activeHand)) {
            // On the remote WorldClient, EntityLivingBase decrements the use
            // counter but does not finish the item locally when it reaches 0.
            self.activeItemStackUseCount -= 1;
        } else {
            self.resetActiveHandClient();
        }
    }

    fn resetActiveHandClient(&mut self) {
        self.activeItemStack = ItemStack::EMPTY;
        self.activeItemStackUseCount = 0;
        self.activeItemEquipmentRevision = None;
    }

    pub fn handleStatusUpdate(&mut self, opcode: i8) {
        self.lastStatusOpcode = Some(opcode);
        match opcode {
            2 | 33 | 36 | 37 => {
                self.limbSwingAmount = 1.5;
                self.hurtResistantTime = self.maxHurtResistantTime;
                self.maxHurtTime = 10;
                self.hurtTime = self.maxHurtTime;
                self.attackedAtYaw = 0.0;
            }
            3 => {
                self.health = 0.0;
            }
            _ => {}
        }
    }

    pub fn isBurning(&self) -> bool {
        self.entity.fire > 0 || (self.dataManager.byte(0, 0) & 0x01) != 0
    }

    pub fn isSneaking(&self) -> bool {
        self.entity.sneaking
    }

    pub fn skinParts(&self) -> u8 {
        self.dataManager.byte(13, 0) as u8
    }

    pub fn isInvisible(&self) -> bool {
        (self.dataManager.byte(0, 0) & 0x20) != 0
    }

    pub fn swingingArmIsLeft(&self) -> bool {
        let mainIsLeft = self.primaryHand == EnumHandSide::Left;
        match self.swingingHand {
            EnumHand::MainHand => mainIsLeft,
            EnumHand::OffHand => !mainIsLeft,
        }
    }
}

fn wrap_degrees_f32(mut value: f32) -> f32 {
    value %= 360.0;
    if value >= 180.0 {
        value -= 360.0;
    }
    if value < -180.0 {
        value += 360.0;
    }
    value
}

fn wrap_degrees_f64(mut value: f64) -> f64 {
    value %= 360.0;
    if value >= 180.0 {
        value -= 360.0;
    }
    if value < -180.0 {
        value += 360.0;
    }
    value
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
    use crate::com::mojang::authlib::minecraft::MinecraftProfileTexture::{
        MinecraftProfileTexture, TextureType,
    };
    use crate::net::minecraft::client::network::NetworkPlayerInfo::NetworkPlayerInfo;
    use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
    use crate::net::minecraft::world::GameType::GameType;
    use std::collections::BTreeMap;

    fn player() -> EntityOtherPlayerMP {
        let id = Uuid::nil();
        EntityOtherPlayerMP::new(1, id, GameProfile::new(Some(id), "Remote"))
    }

    #[test]
    fn direct_position_rotation_reaches_target_in_three_ticks() {
        let mut entity = player();
        entity
            .entity
            .setPositionAndRotation(0.0, 64.0, 0.0, 0.0, 0.0);
        entity.setPositionAndRotationDirect(3.0, 67.0, -6.0, 90.0, 30.0, 3, false);
        entity.entity.onGround = true;
        for _ in 0..3 {
            entity.onUpdate();
        }
        assert!((entity.entity.posX - 3.0).abs() < 1.0e-9);
        assert!((entity.entity.posY - 67.0).abs() < 1.0e-9);
        assert!((entity.entity.posZ + 6.0).abs() < 1.0e-9);
        assert!((entity.entity.rotationYaw - 90.0).abs() < 1.0e-5);
        assert!((entity.entity.rotationPitch - 30.0).abs() < 1.0e-5);
    }

    #[test]
    fn backwards_body_relation_reverses_moved_distance() {
        let mut entity = player();
        entity.entity.rotationYaw = 180.0;
        entity.renderYawOffset = 0.0;
        let distance = entity.updateDistance(0.0, 1.0);
        assert_eq!(distance, -1.0);
    }

    #[test]
    fn arm_swing_progress_is_updated_before_body_yaw_selection() {
        let mut entity = player();
        entity.entity.rotationYaw = 90.0;
        entity.entity.onGround = true;
        entity.swingArm(EnumHand::MainHand);
        entity.onUpdate();
        entity.onUpdate();
        assert!(entity.swingProgress > 0.0);
        assert!(entity.renderYawOffset > 0.0);
    }

    #[test]
    fn hand_state_metadata_tracks_active_stack_and_primary_hand() {
        let mut entity = player();
        let bow = ItemStack {
            itemId: 261,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        };
        entity.setItemStackToSlot(EntityEquipmentSlot::Mainhand, bow.clone());
        entity.applyMetadata([(6, DataValue::Byte(1)), (14, DataValue::Byte(0))]);

        assert!(entity.isHandActive());
        assert_eq!(entity.getActiveHand(), EnumHand::MainHand);
        assert_eq!(entity.getPrimaryHand(), EnumHandSide::Left);
        assert_eq!(entity.getHeldItemMainhand(), &bow);
        assert_eq!(entity.getItemInUseCount(), bow.getMaxItemUseDuration());

        entity.entity.onGround = true;
        entity.onUpdate();
        assert_eq!(entity.getItemInUseCount(), bow.getMaxItemUseDuration() - 1);

        // Replacing the equipment slot with an equal-valued packet stack is a
        // new Java object and must terminate the old active-stack reference.
        entity.setItemStackToSlot(EntityEquipmentSlot::Mainhand, bow.clone());
        entity.onUpdate();
        assert_eq!(entity.getItemInUseCount(), 0);

        entity.applyMetadata([(6, DataValue::Byte(0))]);
        assert!(!entity.isHandActive());
        assert_eq!(entity.getItemInUseCount(), 0);
    }

    #[test]
    fn cached_player_info_survives_tab_removal_and_keeps_texture_callbacks() {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000081").unwrap();
        let info = NetworkPlayerInfo::new(
            GameProfile::new(Some(id), "NpcBot"),
            GameType::Survival,
            0,
            None,
        );
        let mut entity = EntityOtherPlayerMP::new(81, id, info.getGameProfile().clone());
        entity.setPlayerInfo(Some(info.clone()));

        // Removing the tab-map owner drops this clone only. The entity cache
        // retains the shared async texture state just like AbstractClientPlayer.
        drop(info);
        let texture = MinecraftProfileTexture::new(
            "https://textures.minecraft.net/texture/bot",
            BTreeMap::from([("model".to_owned(), "slim".to_owned())]),
        );
        let cached = entity.getPlayerInfo().expect("cached player info");
        NetworkPlayerInfo::applyPlayerTexture(
            &cached.textureState(),
            TextureType::Skin,
            ResourceLocation::new("minecraft", "skins/bot"),
            &texture,
        );

        assert_eq!(cached.getLocationSkin().getPath(), "skins/bot");
        assert_eq!(cached.getSkinType(), "slim");
    }
}
