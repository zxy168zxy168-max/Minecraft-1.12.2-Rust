use std::collections::HashMap;

use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockLiquid::{self, LiquidMaterial};
use crate::net::minecraft::block::{
    BlockButton, BlockDoor, BlockEndPortalFrame, BlockEndRod, BlockFence, BlockFenceGate, BlockLadder, BlockLever,
    BlockPane, BlockPistonBase, BlockPistonExtension, BlockRailBase, BlockSign, BlockSkull, BlockSlime, BlockStairs, BlockTorch, BlockTrapDoor, BlockVine, BlockWall, BlockWeb,
};
use crate::net::minecraft::client::entity::EntityOtherPlayerMP::EntityOtherPlayerMP;
use crate::net::minecraft::client::entity::EntityOtherClient::{ClientEntityKind, EntityOtherClient, ObjectSpawnType};
use crate::net::minecraft::client::network::NetworkPlayerInfo::NetworkPlayerInfo;
use crate::net::minecraft::client::particle::ParticleEmitter::ParticleEmitter;
use crate::net::minecraft::client::particle::ParticleSpawnRequest::ParticleSpawnRequest;
use crate::net::minecraft::client::audio::LocalSoundEvent::LocalSoundEvent;
use crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP;
use crate::net::minecraft::util::SoundCategory::SoundCategory;
use crate::net::minecraft::inventory::EntityEquipmentSlot::EntityEquipmentSlot;
use crate::net::minecraft::entity::effect::EntityLightningBolt::EntityLightningBolt;
use crate::net::minecraft::entity::Entity::Entity;
use crate::net::minecraft::entity::IJumpingMount::IJumpingMount;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::datasync::DataSerializers::DataValue;
use crate::net::minecraft::network::PacketBuffer::CodecError;
use crate::net::minecraft::network::play::server::SPacketBlockAction::SPacketBlockAction;
use crate::net::minecraft::network::play::server::SPacketBlockChange::SPacketBlockChange;
use crate::net::minecraft::network::play::server::SPacketChunkData::SPacketChunkData;
use crate::net::minecraft::network::play::server::SPacketMultiBlockChange::SPacketMultiBlockChange;
use crate::net::minecraft::network::play::server::SPacketUpdateTileEntity::SPacketUpdateTileEntity;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::math::RayTraceResult::RayTraceResult;
use crate::net::minecraft::util::math::Vec3d::Vec3d;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::EnumParticleTypes::EnumParticleTypes;
use crate::net::minecraft::tileentity::TileEntityBeacon::TileEntityBeacon;
use crate::net::minecraft::tileentity::TileEntityBed::TileEntityBed;
use crate::net::minecraft::tileentity::TileEntityChest::TileEntityChest;
use crate::net::minecraft::tileentity::TileEntityEnderChest::TileEntityEnderChest;
use crate::net::minecraft::tileentity::TileEntityEnchantmentTable::TileEntityEnchantmentTable;
use crate::net::minecraft::tileentity::TileEntityEndPortal::TileEntityEndPortal;
use crate::net::minecraft::tileentity::TileEntityFlowerPot::TileEntityFlowerPot;
use crate::net::minecraft::tileentity::TileEntityPiston::TileEntityPiston;
use crate::net::minecraft::tileentity::TileEntityShulkerBox::TileEntityShulkerBox;
use crate::net::minecraft::tileentity::TileEntitySign::TileEntitySign;
use crate::net::minecraft::tileentity::TileEntitySkull::TileEntitySkull;
use crate::net::minecraft::world::EnumDifficulty::EnumDifficulty;
use crate::net::minecraft::world::EnumSkyBlock::EnumSkyBlock;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;
use crate::net::minecraft::world::biome::BiomeColorHelper::BiomeAccess;
use crate::net::minecraft::world::WorldProvider::WorldProvider;
use crate::net::minecraft::world::chunk::BlockStateContainer::BlockStateContainer;
use crate::net::minecraft::world::chunk::Chunk::Chunk;
use crate::net::minecraft::world::chunk::NibbleArray::NibbleArray;
use crate::net::minecraft::world::chunk::storage::ExtendedBlockStorage::ExtendedBlockStorage;


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityRayTraceHit {
    pub entityId: i32,
    pub hitVec: Vec3d,
    pub distance: f64,
}

#[derive(Debug, Clone)]
pub struct WorldClient {
    provider: WorldProvider,
    chunks: HashMap<(i32, i32), Chunk>,
    remotePlayers: HashMap<i32, EntityOtherPlayerMP>,
    nonPlayerEntities: HashMap<i32, EntityOtherClient>,
    weatherEffects: HashMap<i32, EntityLightningBolt>,
    skullTileEntities: HashMap<BlockPos, TileEntitySkull>,
    beaconTileEntities: HashMap<BlockPos, TileEntityBeacon>,
    bedTileEntities: HashMap<BlockPos, TileEntityBed>,
    chestTileEntities: HashMap<BlockPos, TileEntityChest>,
    enderChestTileEntities: HashMap<BlockPos, TileEntityEnderChest>,
    enchantmentTableTileEntities: HashMap<BlockPos, TileEntityEnchantmentTable>,
    endPortalTileEntities: HashMap<BlockPos, TileEntityEndPortal>,
    flowerPotTileEntities: HashMap<BlockPos, TileEntityFlowerPot>,
    pistonTileEntities: HashMap<BlockPos, TileEntityPiston>,
    shulkerBoxTileEntities: HashMap<BlockPos, TileEntityShulkerBox>,
    signTileEntities: HashMap<BlockPos, TileEntitySign>,
    lastLightningBolt: i32,
    /// WorldInfo difficulty from `NetHandlerPlayClient#handleServerDifficulty`.
    difficulty: EnumDifficulty,
    totalWorldTime: i64,
    worldTime: i64,
    doDaylightCycle: bool,
    revision: u64,
    pendingParticles: Vec<ParticleSpawnRequest>,
    pendingSounds: Vec<LocalSoundEvent>,
    particleEmitters: Vec<ParticleEmitter>,
}

impl WorldClient {
    pub fn new(dimensionIn: i32) -> Self {
        Self {
            provider: WorldProvider::new(dimensionIn),
            chunks: HashMap::new(),
            remotePlayers: HashMap::new(),
            nonPlayerEntities: HashMap::new(),
            weatherEffects: HashMap::new(),
            skullTileEntities: HashMap::new(),
            beaconTileEntities: HashMap::new(),
            bedTileEntities: HashMap::new(),
            chestTileEntities: HashMap::new(),
            enderChestTileEntities: HashMap::new(),
            enchantmentTableTileEntities: HashMap::new(),
            endPortalTileEntities: HashMap::new(),
            flowerPotTileEntities: HashMap::new(),
            pistonTileEntities: HashMap::new(),
            shulkerBoxTileEntities: HashMap::new(),
            signTileEntities: HashMap::new(),
            lastLightningBolt: 0,
            difficulty: EnumDifficulty::Normal,
            totalWorldTime: 0,
            worldTime: 0,
            doDaylightCycle: true,
            revision: 0,
            pendingParticles: Vec::new(),
            pendingSounds: Vec::new(),
            particleEmitters: Vec::new(),
        }
    }

    pub const fn providerHasSkyLight(&self) -> bool {
        self.provider.hasSkyLight()
    }

    /// MCP `World#removeAllEntities`, used by `Minecraft#setDimensionAndSpawnPlayer`.
    /// Loaded chunks and tile entities remain; ordinary entities are rebuilt by
    /// subsequent spawn packets after a respawn or dimension transfer.
    pub fn removeAllEntities(&mut self) {
        self.remotePlayers.clear();
        self.nonPlayerEntities.clear();
        self.weatherEffects.clear();
        self.pendingParticles.clear();
        self.pendingSounds.clear();
        self.particleEmitters.clear();
        self.lastLightningBolt = 0;
        self.revision = self.revision.wrapping_add(1);
    }

    /// MCP `WorldClient.tick`: world time plus the client entity update list.
    /// Weather and chunk-provider maintenance remain separate future ports.
    pub fn tick(&mut self) {
        self.tickWithPlayerTarget(None);
    }

    /// Same world tick with the local player's XP-attraction point supplied by
    /// `NetHandlerPlayClient`. Keeping it outside WorldClient mirrors vanilla's
    /// world-owned entity list while avoiding a duplicate local-player owner.
    pub fn tickWithPlayerTarget(&mut self, closestPlayer: Option<[f64; 3]>) {
        self.tickWithPlayerTargetAndLocalPlayer(closestPlayer, None);
    }

    pub fn tickWithPlayerTargetAndLocalPlayer(
        &mut self,
        closestPlayer: Option<[f64; 3]>,
        localPlayerEntityId: Option<i32>,
    ) {
        self.tickWithPlayerContext(closestPlayer, localPlayerEntityId, None);
    }

    pub fn tickWithPlayerContext(
        &mut self,
        closestPlayer: Option<[f64; 3]>,
        localPlayerEntityId: Option<i32>,
        localPlayerState: Option<([f64; 3], f32)>,
    ) {
        self.tickEntitiesWithPlayerContext(
            closestPlayer,
            localPlayerEntityId,
            localPlayerState,
        );
        self.tickTileEntitiesAfterPlayers(closestPlayer, None);
    }

    /// Entity half of MCP `World#updateEntities`. The local `EntityPlayerSP`
    /// is owned by `NetHandlerPlayClient`; that owner ticks it immediately
    /// after this phase and before `tickTileEntitiesAfterPlayers`.
    pub fn tickEntitiesWithPlayerContext(
        &mut self,
        closestPlayer: Option<[f64; 3]>,
        localPlayerEntityId: Option<i32>,
        localPlayerState: Option<([f64; 3], f32)>,
    ) {
        self.totalWorldTime = self.totalWorldTime.wrapping_add(1);
        if self.doDaylightCycle {
            self.worldTime = self.worldTime.wrapping_add(1);
        }
        if self.lastLightningBolt > 0 {
            self.lastLightningBolt -= 1;
        }
        for effect in self.weatherEffects.values_mut() {
            if effect.onUpdate() {
                self.lastLightningBolt = 2;
            }
        }
        self.weatherEffects.retain(|_, effect| !effect.entity.isDead);
        for player in self.remotePlayers.values_mut() {
            player.entity.ticksExisted = player.entity.ticksExisted.wrapping_add(1);
            player.onUpdate();
        }
        let entityIds: Vec<i32> = self.nonPlayerEntities.keys().copied().collect();
        for entityId in entityIds {
            let Some(mut entity) = self.nonPlayerEntities.remove(&entityId) else { continue; };
            entity.onUpdateWithLocalPlayerState(
                self,
                closestPlayer,
                localPlayerEntityId,
                localPlayerState,
            );
            self.pendingParticles.extend(entity.takeParticleSpawns());
            self.pendingSounds.extend(entity.takeSoundEvents());
            self.nonPlayerEntities.insert(entityId, entity);
        }
        self.tickParticleEmitters();
    }

    /// Tile-entity half of MCP `World#updateEntities`, invoked only after all
    /// ordinary entities, including the externally owned local player, have
    /// completed `onUpdate`. This ordering is required for shulker-box pushes:
    /// the local movement packet is selected before the lid sweep, exactly as
    /// in 1.12.2, preventing the server from applying the same displacement a
    /// second time and rubber-banding the player.
    pub fn tickTileEntitiesAfterPlayers(
        &mut self,
        closestPlayer: Option<[f64; 3]>,
        mut localPlayerEntity: Option<&mut Entity>,
    ) {
        // Client-side `TileEntityBeacon#update`: the beam/pyramid scan is
        // not supplied by a network property packet, so the client repeats
        // the source 80-tick world scan. Temporarily remove each tile to keep
        // the world available as the immutable IBlockAccess.
        let beaconPositions = self.beaconTileEntities.keys().copied().collect::<Vec<_>>();
        for pos in beaconPositions {
            let Some(mut beacon) = self.beaconTileEntities.remove(&pos) else { continue; };
            beacon.update(self.totalWorldTime, |sample| self.getBlockState(sample));
            self.beaconTileEntities.insert(pos, beacon);
        }
        for skull in self.skullTileEntities.values_mut() {
            skull.tick();
        }
        for chest in self.chestTileEntities.values_mut() { chest.update(); }
        for chest in self.enderChestTileEntities.values_mut() { chest.update(); }

        // `TileEntityPiston#update`: move intersecting entities using the old
        // progress, then advance by 0.5. Clone the small tile state so entities
        // can be removed from their owner maps while WorldClient remains the
        // immutable collision provider used by `Entity#moveEntity`.
        let pistonTicks = self.pistonTileEntities.values().cloned().collect::<Vec<_>>();
        let mut completedPistons = Vec::new();
        for piston in pistonTicks {
            if piston.progress >= 1.0 {
                completedPistons.push((piston.pos, piston.pistonState));
                continue;
            }
            let nextProgress = piston.nextProgress();
            if let Some(candidateBounds) = piston.sweptEntityBounds(nextProgress) {
                let remoteIds = self.remotePlayers.iter()
                    .filter_map(|(&entityId, player)| player.entity.boundingBox.intersects(candidateBounds).then_some(entityId))
                    .collect::<Vec<_>>();
                for entityId in remoteIds {
                    let Some(mut player) = self.remotePlayers.remove(&entityId) else { continue; };
                    self.moveEntityByPiston(&piston, &mut player.entity, nextProgress);
                    self.remotePlayers.insert(entityId, player);
                }

                let otherIds = self.nonPlayerEntities.iter()
                    .filter_map(|(&entityId, entity)| {
                        (!entity.ignoresShulkerBoxPush() && entity.entity.boundingBox.intersects(candidateBounds))
                            .then_some(entityId)
                    })
                    .collect::<Vec<_>>();
                for entityId in otherIds {
                    let Some(mut entity) = self.nonPlayerEntities.remove(&entityId) else { continue; };
                    self.moveEntityByPiston(&piston, &mut entity.entity, nextProgress);
                    self.nonPlayerEntities.insert(entityId, entity);
                }

                if let Some(entity) = localPlayerEntity.as_deref_mut() {
                    if entity.boundingBox.intersects(candidateBounds) {
                        self.moveEntityByPiston(&piston, entity, nextProgress);
                    }
                }
            }
            if let Some(active) = self.pistonTileEntities.get_mut(&piston.pos) {
                active.update();
            }
        }
        for (pos, movedState) in completedPistons {
            if self.getBlockState(pos).getBlockId() == 36 {
                let _ = self.invalidateRegionAndSetBlock(pos, movedState);
            } else {
                self.pistonTileEntities.remove(&pos);
            }
        }

        for shulker in self.shulkerBoxTileEntities.values_mut() { shulker.update(); }

        // `TileEntityShulkerBox#func_190589_G` runs once per animated tile and
        // moves every ordinary entity intersecting that lid's newly occupied
        // sweep. Preserve tile-first ordering rather than applying every box
        // to one entity at a time.
        let pushes = self.shulkerBoxTileEntities.values().filter_map(|shulker| {
            if !shulker.pushesEntitiesThisTick() {
                return None;
            }
            let state = self.getBlockState(shulker.pos);
            (219..=234).contains(&state.getBlockId()).then_some((
                shulker.clone(),
                EnumFacing::getFront(state.getMetadata()),
            ))
        }).collect::<Vec<_>>();
        for (shulker, facing) in pushes {
            let remoteIds = self.remotePlayers.keys().copied().collect::<Vec<_>>();
            for entityId in remoteIds {
                let Some(mut player) = self.remotePlayers.remove(&entityId) else { continue; };
                if let Some([x, y, z]) = shulker.pushDisplacement(facing, player.entity.boundingBox) {
                    player.entity.moveEntity(self, x, y, z);
                }
                self.remotePlayers.insert(entityId, player);
            }

            let otherIds = self.nonPlayerEntities.keys().copied().collect::<Vec<_>>();
            for entityId in otherIds {
                let Some(mut entity) = self.nonPlayerEntities.remove(&entityId) else { continue; };
                if !entity.ignoresShulkerBoxPush() {
                    if let Some([x, y, z]) = shulker.pushDisplacement(facing, entity.entity.boundingBox) {
                        entity.entity.moveEntity(self, x, y, z);
                    }
                }
                self.nonPlayerEntities.insert(entityId, entity);
            }

            if let Some(entity) = localPlayerEntity.as_deref_mut() {
                if let Some([x, y, z]) = shulker.pushDisplacement(facing, entity.boundingBox) {
                    entity.moveEntity(self, x, y, z);
                }
            }
        }

        let localPosition = localPlayerEntity.as_deref().map(|entity| [
            entity.posX,
            entity.posY,
            entity.posZ,
        ]).or(closestPlayer);
        let mut playerPositions = Vec::with_capacity(
            self.remotePlayers.len() + if localPosition.is_some() { 1 } else { 0 },
        );
        if let Some(position) = localPosition {
            playerPositions.push(position);
        }
        playerPositions.extend(self.remotePlayers.values().filter_map(|player| {
            (!player.entity.isDead).then_some([
                player.entity.posX,
                player.entity.posY,
                player.entity.posZ,
            ])
        }));
        for table in self.enchantmentTableTileEntities.values_mut() {
            let center = [
                table.pos.x as f64 + 0.5,
                table.pos.y as f64 + 0.5,
                table.pos.z as f64 + 0.5,
            ];
            let nearest = playerPositions
                .iter()
                .copied()
                .filter_map(|position| {
                    let dx = position[0] - center[0];
                    let dy = position[1] - center[1];
                    let dz = position[2] - center[2];
                    let distanceSquared = dx * dx + dy * dy + dz * dz;
                    (distanceSquared <= 9.0).then_some((distanceSquared, position))
                })
                .min_by(|left, right| left.0.total_cmp(&right.0))
                .map(|(_, position)| position);
            table.update(nearest);
        }

        let deadIds: Vec<i32> = self.remotePlayers
            .iter()
            .filter_map(|(&entityId, entity)| entity.entity.isDead.then_some(entityId))
            .chain(
                self.nonPlayerEntities
                    .iter()
                    .filter_map(|(&entityId, entity)| entity.entity.isDead.then_some(entityId)),
            )
            .collect();
        for entityId in deadIds {
            self.removeEntityFromWorld(entityId);
        }
    }


    fn moveEntityByPiston(
        &self,
        piston: &TileEntityPiston,
        entity: &mut Entity,
        nextProgress: f32,
    ) {
        let direction = piston.movementDirection();
        let distance = piston.primaryPushDistance(entity.boundingBox, nextProgress);
        if distance <= 0.0 {
            return;
        }
        let (dx, dy, dz) = direction.offsets();
        if piston.isMovingSlimeBlock() {
            match direction.axis() {
                crate::net::minecraft::util::EnumFacing::Axis::X => entity.motionX = dx as f64,
                crate::net::minecraft::util::EnumFacing::Axis::Y => entity.motionY = dy as f64,
                crate::net::minecraft::util::EnumFacing::Axis::Z => entity.motionZ = dz as f64,
            }
        }
        TileEntityPiston::withPushDirection(direction, || {
            entity.moveEntityPiston(
                self,
                distance * dx as f64,
                distance * dy as f64,
                distance * dz as f64,
            );
        });

        let progressDelta = (nextProgress - piston.progress) as f64;
        let correction = piston.retractionCorrectionDistance(entity.boundingBox, progressDelta);
        if correction > 0.0 {
            let outward = direction.opposite();
            let (ox, oy, oz) = outward.offsets();
            TileEntityPiston::withPushDirection(direction, || {
                entity.moveEntityPiston(
                    self,
                    correction * ox as f64,
                    correction * oy as f64,
                    correction * oz as f64,
                );
            });
        }
    }

    /// `EffectRenderer#emitParticleAtEntity`, represented here because the
    /// network-owned WorldClient is the authoritative owner of remote entity
    /// positions while the Vulkan renderer owns concrete particles.
    pub fn addParticleEmitter(
        &mut self,
        entity_id: i32,
        particle_type: EnumParticleTypes,
        lifetime: i32,
    ) -> bool {
        if self.getBaseEntityByID(entity_id).is_none() {
            return false;
        }
        self.particleEmitters.push(ParticleEmitter::new(
            entity_id,
            particle_type,
            lifetime,
        ));
        true
    }

    fn tickParticleEmitters(&mut self) {
        let mut emitters = core::mem::take(&mut self.particleEmitters);
        for emitter in &mut emitters {
            if let Some(entity) = self.getBaseEntityByID(emitter.attachedEntityId()).cloned() {
                self.pendingParticles.extend(emitter.onUpdate(&entity));
            }
        }
        emitters.retain(|emitter| !emitter.isExpired());
        self.particleEmitters = emitters;
    }

    /// Drains entity-authored client particle requests after the world tick.
    /// This keeps `WorldClient` as the entity owner while `Minecraft` retains
    /// MCP `ParticleManager` ownership and render-layer ordering.
    pub fn takeParticleSpawns(&mut self) -> Vec<ParticleSpawnRequest> {
        core::mem::take(&mut self.pendingParticles)
    }

    pub fn queueParticleSpawns(&mut self, requests: impl IntoIterator<Item = ParticleSpawnRequest>) {
        self.pendingParticles.extend(requests);
    }

    pub fn addEntityToWorld(&mut self, entityId: i32, entity: EntityOtherPlayerMP) -> Option<EntityOtherPlayerMP> {
        self.nonPlayerEntities.remove(&entityId);
        self.revision = self.revision.wrapping_add(1);
        self.remotePlayers.insert(entityId, entity)
    }

    /// MCP `World.addWeatherEffect`, used by protocol 340 global-entity
    /// packet type 1 (`EntityLightningBolt`). Weather effects remain separate
    /// from the regular loaded-entity map.
    pub fn addWeatherEffect(&mut self, effect: EntityLightningBolt) -> Option<EntityLightningBolt> {
        self.revision = self.revision.wrapping_add(1);
        self.weatherEffects.insert(effect.entityId, effect)
    }

    pub fn weatherEffects(&self) -> impl Iterator<Item = &EntityLightningBolt> {
        self.weatherEffects.values()
    }

    pub fn weatherEffectCount(&self) -> usize { self.weatherEffects.len() }
    pub const fn getLastLightningBolt(&self) -> i32 { self.lastLightningBolt }

    pub fn addNonPlayerEntityToWorld(
        &mut self,
        entityId: i32,
        entity: EntityOtherClient,
    ) -> Option<EntityOtherClient> {
        self.remotePlayers.remove(&entityId);
        self.revision = self.revision.wrapping_add(1);
        self.nonPlayerEntities.insert(entityId, entity)
    }

    /// MCP `WorldClient.removeEntityFromWorld` removes any concrete entity.
    /// The player return type is retained for existing player-render callers.
    pub fn removeEntityFromWorld(&mut self, entityId: i32) -> Option<EntityOtherPlayerMP> {
        self.clearRidingRelations(entityId);
        let removedPlayer = self.remotePlayers.remove(&entityId);
        let removedOther = self.nonPlayerEntities.remove(&entityId);
        if removedPlayer.is_some() || removedOther.is_some() {
            self.revision = self.revision.wrapping_add(1);
        }
        removedPlayer
    }

    pub fn getEntityByID(&self, entityId: i32) -> Option<&EntityOtherPlayerMP> {
        self.remotePlayers.get(&entityId)
    }

    pub fn getEntityByIDMut(&mut self, entityId: i32) -> Option<&mut EntityOtherPlayerMP> {
        self.remotePlayers.get_mut(&entityId)
    }

    /// Completes and refreshes the lazy `AbstractClientPlayer#getPlayerInfo`
    /// relationship. Java caches the same mutable NetworkPlayerInfo object that
    /// remains in the player-info map, so game-mode/display-name updates remain
    /// visible while the entry exists. Replacing the Rust clone preserves that
    /// behavior; REMOVE_PLAYER does not call this method, leaving the last
    /// resolved object on short-lived NPC/Bot entities.
    pub fn cachePlayerInfo(
        &mut self,
        uniqueId: uuid::Uuid,
        playerInfo: NetworkPlayerInfo,
    ) -> bool {
        let Some(player) = self
            .remotePlayers
            .values_mut()
            .find(|player| player.uniqueId == uniqueId)
        else {
            return false;
        };
        player.setPlayerInfo(Some(playerInfo));
        true
    }

    pub fn getNonPlayerEntityByID(&self, entityId: i32) -> Option<&EntityOtherClient> {
        self.nonPlayerEntities.get(&entityId)
    }

    pub fn getNonPlayerEntityByIDMut(&mut self, entityId: i32) -> Option<&mut EntityOtherClient> {
        self.nonPlayerEntities.get_mut(&entityId)
    }

    /// Client-world return value of the concrete target's
    /// `Entity#attackEntityFrom` implementation for a direct player attack.
    /// `EntityPlayer#attackTargetEntityWithCurrentItem` only applies its local
    /// knockback slowdown after this returns true. Ordinary mobs inherit
    /// `EntityLivingBase`, whose remote-world branch returns false; player,
    /// hanging and the listed object classes explicitly return true in MCP.
    pub fn clientAttackEntityFromReturnsTrue(&self, entityId: i32) -> bool {
        if self.remotePlayers.contains_key(&entityId) {
            // EntityOtherPlayerMP#attackEntityFrom always returns true.
            return true;
        }
        let Some(entity) = self.nonPlayerEntities.get(&entityId) else {
            return false;
        };
        match &entity.kind {
            ClientEntityKind::Painting { .. } => true,
            ClientEntityKind::Object { objectType, .. } => matches!(
                *objectType,
                ObjectSpawnType::Boat
                    | ObjectSpawnType::Minecart
                    | ObjectSpawnType::EnderCrystal
                    | ObjectSpawnType::LargeFireball
                    | ObjectSpawnType::ShulkerBullet
                    | ObjectSpawnType::ItemFrame
                    | ObjectSpawnType::LeashKnot
            ),
            ClientEntityKind::Mob { .. } | ClientEntityKind::ExperienceOrb { .. } => false,
        }
    }

    /// ID-backed equivalent of MCP `EntityBoat#updateInputs`, invoked by
    /// `EntityPlayerSP#updateRidden` after the passenger tick.
    pub fn setBoatInputs(
        &mut self,
        vehicleId: i32,
        left: bool,
        right: bool,
        forward: bool,
        back: bool,
    ) -> bool {
        let Some(vehicle) = self.nonPlayerEntities.get_mut(&vehicleId) else { return false; };
        if !matches!(
            &vehicle.kind,
            crate::net::minecraft::client::entity::EntityOtherClient::ClientEntityKind::Object {
                objectType: crate::net::minecraft::client::entity::EntityOtherClient::ObjectSpawnType::Boat,
                ..
            }
        ) {
            return false;
        }
        vehicle.updateBoatInputs(left, right, forward, back);
        true
    }

    /// Stores the controlling player's values for the client-predicted
    /// `AbstractHorse#func_191986_a` branch.
    pub fn setHorseInputs(
        &mut self,
        vehicleId: i32,
        riderYaw: f32,
        riderPitch: f32,
        moveStrafing: f32,
        moveForward: f32,
    ) -> bool {
        let Some(vehicle) = self.nonPlayerEntities.get_mut(&vehicleId) else { return false; };
        if !vehicle.isHorseFamily() { return false; }
        vehicle.updateHorseInputs(riderYaw, riderPitch, moveStrafing, moveForward);
        true
    }

    /// Client half of `IJumpingMount#setJumpPower`, invoked immediately when
    /// EntityPlayerSP releases the charged jump key.
    pub fn setHorseJumpPower(&mut self, vehicleId: i32, jumpPower: i32) -> bool {
        let Some(vehicle) = self.nonPlayerEntities.get_mut(&vehicleId) else { return false; };
        if !vehicle.isHorseFamily() || !IJumpingMount::canJump(vehicle) { return false; }
        IJumpingMount::setJumpPower(vehicle, jumpPower);
        true
    }

    /// MCP `Entity#getControllingPassenger` + `canPassengerSteer` client rule.
    /// On a remote world the vehicle is locally steerable only when its first
    /// passenger is the local user.
    pub fn localPlayerControlsVehicle(&self, vehicleId: i32, localPlayerId: i32) -> bool {
        self.baseEntity(vehicleId)
            .and_then(|entity| entity.passengerIds.first().copied())
            == Some(localPlayerId)
    }

    pub fn setEntityPositionAndRotation(
        &mut self,
        entityId: i32,
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
    ) -> bool {
        let Some(entity) = self.baseEntityMut(entityId) else { return false; };
        entity.setPositionAndRotation(x, y, z, yaw, pitch);
        true
    }

    pub fn remotePlayers(&self) -> impl Iterator<Item = &EntityOtherPlayerMP> {
        self.remotePlayers.values()
    }

    pub fn nonPlayerEntities(&self) -> impl Iterator<Item = &EntityOtherClient> {
        self.nonPlayerEntities.values()
    }

    pub fn nonPlayerEntityCount(&self) -> usize { self.nonPlayerEntities.len() }

    pub fn setEntityVelocity(&mut self, entityId: i32, x: f64, y: f64, z: f64) -> bool {
        if let Some(player) = self.remotePlayers.get_mut(&entityId) {
            player.setVelocity(x, y, z);
            return true;
        }
        if let Some(entity) = self.nonPlayerEntities.get_mut(&entityId) {
            entity.setVelocity(x, y, z);
            return true;
        }
        false
    }

    pub fn applyEntityMetadata(
        &mut self,
        entityId: i32,
        entries: impl IntoIterator<Item = (u8, DataValue)> + Clone,
    ) -> bool {
        if let Some(player) = self.remotePlayers.get_mut(&entityId) {
            player.applyMetadata(entries.clone());
            return true;
        }
        if let Some(entity) = self.nonPlayerEntities.get_mut(&entityId) {
            entity.applyMetadata(entries);
            return true;
        }
        false
    }

    pub fn setEntityRotationYawHead(&mut self, entityId: i32, yaw: f32) -> bool {
        if let Some(player) = self.remotePlayers.get_mut(&entityId) {
            player.setRotationYawHead(yaw);
            return true;
        }
        if let Some(entity) = self.nonPlayerEntities.get_mut(&entityId) {
            entity.setRotationYawHead(yaw);
            return true;
        }
        false
    }

    pub fn setEntityEquipment(
        &mut self,
        entityId: i32,
        slot: EntityEquipmentSlot,
        stack: ItemStack,
    ) -> bool {
        if let Some(player) = self.remotePlayers.get_mut(&entityId) {
            player.setItemStackToSlot(slot, stack);
            return true;
        }
        if let Some(entity) = self.nonPlayerEntities.get_mut(&entityId) {
            entity.setItemStackToSlot(slot, stack);
            return true;
        }
        false
    }

    /// Completes the attached-firework owner branch after EntityPlayerSP's
    /// externally owned tick, matching World#updateEntities ordering.
    pub fn updateAttachedFireworksForLocalPlayer(&mut self, player: &mut EntityPlayerSP) {
        let ids = self.nonPlayerEntities.keys().copied().collect::<Vec<_>>();
        for entityId in ids {
            let Some(entity) = self.nonPlayerEntities.get_mut(&entityId) else { continue; };
            entity.updateAttachedFireworkForLocalPlayer(player);
        }
    }

    pub fn queueSoundAtEntity(
        &mut self,
        entityId: i32,
        sound: impl AsRef<str>,
        volume: f32,
        pitch: f32,
    ) -> bool {
        let (position, category) = if let Some(player) = self.remotePlayers.get(&entityId) {
            ([player.entity.posX as f32, player.entity.posY as f32, player.entity.posZ as f32], SoundCategory::Players)
        } else if let Some(entity) = self.nonPlayerEntities.get(&entityId) {
            let category = match &entity.kind {
                crate::net::minecraft::client::entity::EntityOtherClient::ClientEntityKind::Mob { entityType }
                    if entityType.id == 65 => SoundCategory::Ambient,
                crate::net::minecraft::client::entity::EntityOtherClient::ClientEntityKind::Mob { entityType }
                    if matches!(entityType.id, 4..=6 | 23 | 27 | 34..=37 | 50..=69) => SoundCategory::Hostile,
                _ => SoundCategory::Neutral,
            };
            ([entity.entity.posX as f32, entity.entity.posY as f32, entity.entity.posZ as f32], category)
        } else {
            return false;
        };
        self.pendingSounds.push(LocalSoundEvent::positioned(sound, category, position, volume, pitch));
        true
    }

    pub fn takeSoundEvents(&mut self) -> Vec<LocalSoundEvent> {
        core::mem::take(&mut self.pendingSounds)
    }

    pub fn handleEntityStatus(&mut self, entityId: i32, opcode: i8) -> bool {
        if let Some(player) = self.remotePlayers.get_mut(&entityId) {
            player.handleStatusUpdate(opcode);
            return true;
        }
        if let Some(entity) = self.nonPlayerEntities.get_mut(&entityId) {
            entity.handleStatusUpdate(opcode);
            return true;
        }
        false
    }

    pub fn setPassengers(&mut self, vehicleId: i32, passengerIds: &[i32]) -> bool {
        let previous = if let Some(player) = self.remotePlayers.get_mut(&vehicleId) {
            player.entity.removePassengers()
        } else if let Some(entity) = self.nonPlayerEntities.get_mut(&vehicleId) {
            entity.entity.removePassengers()
        } else {
            return false;
        };

        for passengerId in previous {
            if let Some(passenger) = self.baseEntityMut(passengerId) {
                if passenger.ridingEntityId == Some(vehicleId) {
                    passenger.ridingEntityId = None;
                }
            }
        }

        for &passengerId in passengerIds {
            // MCP `Entity.startRiding(entity, true)` first dismounts the
            // passenger from any previous vehicle before adding it to the new
            // one. Preserve both sides of the ID-backed Rust relation.
            let oldVehicleId = self
                .baseEntity(passengerId)
                .and_then(|passenger| passenger.ridingEntityId);
            if let Some(oldVehicleId) = oldVehicleId.filter(|old| *old != vehicleId) {
                if let Some(oldVehicle) = self.baseEntityMut(oldVehicleId) {
                    oldVehicle.passengerIds.retain(|id| *id != passengerId);
                }
            }
            if let Some(passenger) = self.baseEntityMut(passengerId) {
                passenger.ridingEntityId = Some(vehicleId);
            }
        }

        if let Some(vehicle) = self.baseEntityMut(vehicleId) {
            vehicle.setPassengers(passengerIds.to_vec());
        }
        self.revision = self.revision.wrapping_add(1);
        true
    }

    /// MCP `handleEntityAttach` is the leash relation, not riding. Only
    /// `EntityLiving` subclasses consume this packet.
    pub fn attachEntity(&mut self, entityId: i32, leashHolderId: i32) -> bool {
        let Some(entity) = self.nonPlayerEntities.get_mut(&entityId) else { return false; };
        if !matches!(&entity.kind, crate::net::minecraft::client::entity::EntityOtherClient::ClientEntityKind::Mob { .. }) {
            return false;
        }
        entity.setLeashHolderId((leashHolderId >= 0).then_some(leashHolderId));
        self.revision = self.revision.wrapping_add(1);
        true
    }

    pub fn getBaseEntityByID(&self, entityId: i32) -> Option<&crate::net::minecraft::entity::Entity::Entity> {
        self.baseEntity(entityId)
    }

    fn baseEntity(&self, entityId: i32) -> Option<&crate::net::minecraft::entity::Entity::Entity> {
        if let Some(player) = self.remotePlayers.get(&entityId) {
            return Some(&player.entity);
        }
        self.nonPlayerEntities.get(&entityId).map(|entity| &entity.entity)
    }

    fn baseEntityMut(&mut self, entityId: i32) -> Option<&mut crate::net::minecraft::entity::Entity::Entity> {
        if let Some(player) = self.remotePlayers.get_mut(&entityId) {
            return Some(&mut player.entity);
        }
        self.nonPlayerEntities.get_mut(&entityId).map(|entity| &mut entity.entity)
    }

    fn clearRidingRelations(&mut self, entityId: i32) {
        for player in self.remotePlayers.values_mut() {
            if player.entity.ridingEntityId == Some(entityId) { player.entity.ridingEntityId = None; }
            player.entity.passengerIds.retain(|id| *id != entityId);
        }
        for entity in self.nonPlayerEntities.values_mut() {
            if entity.entity.ridingEntityId == Some(entityId) { entity.entity.ridingEntityId = None; }
            entity.entity.passengerIds.retain(|id| *id != entityId);
        }
    }

    /// MCP `World.getEntitiesInAABBexcluding` subset used by
    /// `EntityRenderer.getMouseOver`. It includes every concrete entity whose
    /// source class returns true from `canBeCollidedWith`, not just players.
    pub fn rayTraceEntities(
        &self,
        viewerEntityId: i32,
        viewerRidingEntityId: Option<i32>,
        viewerBox: AxisAlignedBB,
        eye: Vec3d,
        look: Vec3d,
        reach: f64,
        blockDistance: f64,
        extendedReach: bool,
    ) -> Option<EntityRayTraceHit> {
        let end = eye.add_vector(look.x * reach, look.y * reach, look.z * reach);
        let search = viewerBox
            .add_coord(look.x * reach, look.y * reach, look.z * reach)
            .expand_xyz(1.0);
        let viewerRoot = self.lowestRidingEntityId(viewerEntityId, viewerRidingEntityId);
        let mut nearest = blockDistance;
        let mut selected = None;

        let mut consider = |entityId: i32, bounds: AxisAlignedBB, collisionBorder: f64, ridingEntityId: Option<i32>| {
            if entityId == viewerEntityId || !bounds.intersects(search) { return; }
            let bounds = bounds.expand_xyz(collisionBorder);
            let intercept = bounds.calculate_intercept(eye, end);
            let (hit, distance) = if bounds.contains(eye) {
                (intercept.map_or(eye, |(hit, _)| hit), 0.0)
            } else if let Some((hit, _)) = intercept {
                (hit, eye.distance_to(hit))
            } else {
                return;
            };
            if distance < nearest || nearest == 0.0 {
                let targetRoot = self.lowestRidingEntityId(entityId, ridingEntityId);
                if targetRoot == viewerRoot {
                    if nearest == 0.0 {
                        selected = Some(EntityRayTraceHit { entityId, hitVec: hit, distance });
                    }
                } else {
                    nearest = distance;
                    selected = Some(EntityRayTraceHit { entityId, hitVec: hit, distance });
                }
            }
        };

        for player in self.remotePlayers.values() {
            if !player.entity.isDead {
                consider(
                    player.entityId,
                    player.entity.boundingBox,
                    0.0,
                    player.entity.ridingEntityId,
                );
            }
        }
        for entity in self.nonPlayerEntities.values() {
            if entity.canBeCollidedWith() {
                consider(
                    entity.entityId,
                    entity.entity.boundingBox,
                    entity.collisionBorderSize(),
                    entity.entity.ridingEntityId,
                );
            }
        }

        if !extendedReach && selected.is_some_and(|hit| hit.distance > 3.0) { None } else { selected }
    }

    pub fn lowestRidingEntityId(&self, entityId: i32, initialRiding: Option<i32>) -> i32 {
        let mut current = entityId;
        let mut riding = initialRiding;
        let mut remaining = self.remotePlayers.len() + self.nonPlayerEntities.len() + 1;
        while let Some(parent) = riding {
            if remaining == 0 { break; }
            remaining -= 1;
            current = parent;
            riding = self.baseEntity(parent).and_then(|entity| entity.ridingEntityId);
        }
        current
    }

    pub fn entityPosition(&self, entityId: i32) -> Option<[f64; 3]> {
        self.baseEntity(entityId).map(|entity| [entity.posX, entity.posY, entity.posZ])
    }

    pub fn setTotalWorldTime(&mut self, time: i64) {
        self.totalWorldTime = time;
    }

    /// MCP `WorldClient.setWorldTime`: a negative server value disables the
    /// daylight cycle and is stored as its positive magnitude.
    pub fn setWorldTime(&mut self, time: i64) {
        if time < 0 {
            self.worldTime = time.saturating_neg();
            self.doDaylightCycle = false;
        } else {
            self.worldTime = time;
            self.doDaylightCycle = true;
        }
    }

    pub const fn getTotalWorldTime(&self) -> i64 {
        self.totalWorldTime
    }

    pub const fn getWorldTime(&self) -> i64 {
        self.worldTime
    }

    pub const fn isDaylightCycleEnabled(&self) -> bool {
        self.doDaylightCycle
    }

    pub const fn getProvider(&self) -> &WorldProvider {
        &self.provider
    }

    pub fn getCelestialAngle(&self, partialTicks: f32) -> f32 {
        self.provider
            .calculateCelestialAngle(self.worldTime, partialTicks)
    }

    pub fn doPreChunk(&mut self, x: i32, z: i32, loadChunk: bool) {
        if loadChunk {
            self.chunks
                .entry((x, z))
                .or_insert_with(|| Chunk::new(x, z));
        } else {
            self.chunks.remove(&(x, z));
            self.skullTileEntities.retain(|pos, _| pos.x.div_euclid(16) != x || pos.z.div_euclid(16) != z);
            self.bedTileEntities.retain(|pos, _| pos.x.div_euclid(16) != x || pos.z.div_euclid(16) != z);
            self.chestTileEntities.retain(|pos, _| pos.x.div_euclid(16) != x || pos.z.div_euclid(16) != z);
            self.enderChestTileEntities.retain(|pos, _| pos.x.div_euclid(16) != x || pos.z.div_euclid(16) != z);
            self.enchantmentTableTileEntities.retain(|pos, _| pos.x.div_euclid(16) != x || pos.z.div_euclid(16) != z);
            self.pistonTileEntities.retain(|pos, _| pos.x.div_euclid(16) != x || pos.z.div_euclid(16) != z);
            self.shulkerBoxTileEntities.retain(|pos, _| pos.x.div_euclid(16) != x || pos.z.div_euclid(16) != z);
            self.signTileEntities.retain(|pos, _| pos.x.div_euclid(16) != x || pos.z.div_euclid(16) != z);
        }
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn skullTileEntities(&self) -> impl Iterator<Item = &TileEntitySkull> {
        self.skullTileEntities.values()
    }

    pub fn getTileEntitySkull(&self, pos: BlockPos) -> Option<&TileEntitySkull> {
        self.skullTileEntities.get(&pos)
    }

    /// `World#setTileEntity` subset for `TileEntitySkull`, used by both full
    /// chunk NBT and `SPacketUpdateTileEntity` action 4.
    pub fn applySkullTileEntityTag(&mut self, tag: &crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound) -> bool {
        let Some(skull) = TileEntitySkull::fromNbt(tag) else { return false; };
        // `NetHandlerPlayClient#handleUpdateTileEntity` only applies action 4
        // when the loaded TileEntity at the packet position is a skull. The
        // compact world represents that TileEntity through the block state,
        // so stale/out-of-order skull NBT must not create an invisible TE.
        if !BlockSkull::BlockSkull::isBlockSkull(self.getBlockState(skull.pos)) {
            return false;
        }
        self.skullTileEntities.insert(skull.pos, skull);
        self.revision = self.revision.wrapping_add(1);
        true
    }

    pub fn handleUpdateTileEntity(&mut self, packet: &SPacketUpdateTileEntity) -> bool {
        let mut tag = packet.getNbtCompound().clone();
        tag.setInteger("x", packet.getPos().x);
        tag.setInteger("y", packet.getPos().y);
        tag.setInteger("z", packet.getPos().z);
        match packet.getTileEntityType() {
            4 => {
                if !tag.hasKey("id") { tag.setString("id", "minecraft:skull"); }
                self.applySkullTileEntityTag(&tag)
            }
            5 => {
                if !tag.hasKey("id") { tag.setString("id", "minecraft:flower_pot"); }
                self.applySpecialTileEntityTag(&tag)
            }
            11 => {
                if !tag.hasKey("id") { tag.setString("id", "minecraft:bed"); }
                self.applySpecialTileEntityTag(&tag)
            }
            10 => {
                if !tag.hasKey("id") { tag.setString("id", "minecraft:shulker_box"); }
                self.applySpecialTileEntityTag(&tag)
            }
            9 => {
                if !tag.hasKey("id") { tag.setString("id", "minecraft:sign"); }
                self.applySpecialTileEntityTag(&tag)
            }
            _ => self.applySpecialTileEntityTag(&tag),
        }
    }


    pub fn beaconTileEntities(&self) -> impl Iterator<Item = &TileEntityBeacon> {
        self.beaconTileEntities.values()
    }

    pub fn bedTileEntities(&self) -> impl Iterator<Item = &TileEntityBed> {
        self.bedTileEntities.values()
    }

    pub fn chestTileEntities(&self) -> impl Iterator<Item = &TileEntityChest> {
        self.chestTileEntities.values()
    }

    pub fn getChestTileEntity(&self, pos: BlockPos) -> Option<&TileEntityChest> {
        self.chestTileEntities.get(&pos)
    }

    pub fn enderChestTileEntities(&self) -> impl Iterator<Item = &TileEntityEnderChest> {
        self.enderChestTileEntities.values()
    }

    pub fn getEnderChestTileEntity(&self, pos: BlockPos) -> Option<&TileEntityEnderChest> {
        self.enderChestTileEntities.get(&pos)
    }

    pub fn enchantmentTableTileEntities(&self) -> impl Iterator<Item = &TileEntityEnchantmentTable> {
        self.enchantmentTableTileEntities.values()
    }

    pub fn endPortalTileEntities(&self) -> impl Iterator<Item = &TileEntityEndPortal> {
        self.endPortalTileEntities.values()
    }

    /// `WorldInfo#setDifficulty` from `NetHandlerPlayClient#handleServerDifficulty`.
    pub fn setDifficulty(&mut self, difficulty: EnumDifficulty) {
        self.difficulty = difficulty;
    }

    pub fn getDifficulty(&self) -> EnumDifficulty { self.difficulty }


    pub fn flowerPotTileEntities(&self) -> impl Iterator<Item = &TileEntityFlowerPot> {
        self.flowerPotTileEntities.values()
    }

    pub fn getFlowerPotTileEntity(&self, pos: BlockPos) -> Option<&TileEntityFlowerPot> {
        self.flowerPotTileEntities.get(&pos)
    }

    pub fn pistonTileEntities(&self) -> impl Iterator<Item = &TileEntityPiston> {
        self.pistonTileEntities.values()
    }

    pub fn shulkerBoxTileEntities(&self) -> impl Iterator<Item = &TileEntityShulkerBox> {
        self.shulkerBoxTileEntities.values()
    }

    pub fn signTileEntities(&self) -> impl Iterator<Item = &TileEntitySign> {
        self.signTileEntities.values()
    }

    /// MCP `World#getTileEntity` subset used by `NetHandlerPlayClient#handleSignEditorOpen`.
    pub fn getSignTileEntity(&self, pos: BlockPos) -> Option<&TileEntitySign> {
        self.signTileEntities.get(&pos)
    }

    pub fn getSignTileEntityMut(&mut self, pos: BlockPos) -> Option<&mut TileEntitySign> {
        self.signTileEntities.get_mut(&pos)
    }

    /// Vanilla creates a temporary `TileEntitySign` when the server asks to
    /// edit a position whose tile entity has not arrived yet.
    pub fn getOrCreateSignTileEntity(&mut self, pos: BlockPos) -> &mut TileEntitySign {
        self.signTileEntities.entry(pos).or_insert_with(|| TileEntitySign::new(pos))
    }


    /// `World#setTileEntity` subset for bed/chest/piston special renderers.
    pub fn applySpecialTileEntityTag(
        &mut self,
        tag: &crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound,
    ) -> bool {
        let id = tag.getString("id");
        let pos = BlockPos::new(tag.getInteger("x"), tag.getInteger("y"), tag.getInteger("z"));
        let blockId = self.getBlockState(pos).getBlockId();
        let applied = match id.as_str() {
            "minecraft:beacon" | "Beacon" if blockId == TileEntityBeacon::BLOCK_ID => {
                if let Some(tile) = TileEntityBeacon::fromNbt(tag) {
                    self.beaconTileEntities.insert(pos, tile);
                    true
                } else { false }
            }
            "minecraft:bed" | "Bed" if blockId == 26 => {
                if let Some(tile) = TileEntityBed::fromNbt(tag) {
                    self.bedTileEntities.insert(pos, tile);
                    true
                } else { false }
            }
            "minecraft:chest" | "Chest" | "minecraft:trapped_chest"
                if matches!(blockId, 54 | 146) =>
            {
                if let Some(tile) = TileEntityChest::fromNbt(tag) {
                    self.chestTileEntities.insert(pos, tile);
                    true
                } else { false }
            }
            "minecraft:enchanting_table" | "EnchantTable" if blockId == 116 => {
                if let Some(tile) = TileEntityEnchantmentTable::fromNbt(tag) {
                    self.enchantmentTableTileEntities.insert(pos, tile);
                    true
                } else { false }
            }
            "minecraft:end_portal" | "Airportal" if blockId == 119 => {
                if let Some(tile) = TileEntityEndPortal::fromNbt(tag) {
                    self.endPortalTileEntities.insert(pos, tile);
                    true
                } else { false }
            }
            "minecraft:flower_pot" | "FlowerPot" if blockId == 140 => {
                if let Some(tile) = TileEntityFlowerPot::fromNbt(tag) {
                    self.flowerPotTileEntities.insert(pos, tile);
                    if let Some(chunk) = self.getChunkFromChunkCoordsMut(
                        pos.x.div_euclid(16),
                        pos.z.div_euclid(16),
                    ) {
                        chunk.markSectionDirty(pos.y.div_euclid(16) as usize);
                    }
                    true
                } else { false }
            }
            "minecraft:ender_chest" | "EnderChest" if blockId == 130 => {
                self.enderChestTileEntities
                    .entry(pos)
                    .or_insert_with(|| TileEntityEnderChest::new(pos));
                true
            }
            "minecraft:piston" | "Piston" if blockId == 36 => {
                if let Some(tile) = TileEntityPiston::fromNbt(tag) {
                    self.pistonTileEntities.insert(pos, tile);
                    true
                } else { false }
            }
            "minecraft:sign" | "Sign" if matches!(blockId, 63 | 68) => {
                if let Some(tile) = TileEntitySign::fromNbt(tag) {
                    self.signTileEntities.insert(pos, tile);
                    true
                } else { false }
            }
            "minecraft:shulker_box" | "ShulkerBox" if (219..=234).contains(&blockId) => {
                if let Some(tile) = TileEntityShulkerBox::fromNbt(tag, blockId - 219) {
                    self.shulkerBoxTileEntities.insert(pos, tile);
                    true
                } else { false }
            }
            _ => false,
        };
        if applied { self.revision = self.revision.wrapping_add(1); }
        applied
    }

    /// MCP `World#addBlockEvent` client dispatch for chest lid events.
    pub fn handleBlockAction(&mut self, packet: &SPacketBlockAction) -> bool {
        let pos = packet.getBlockPosition();
        let currentId = self.getBlockState(pos).getBlockId();
        if currentId != packet.getBlockTypeId() { return false; }
        let handled = match currentId {
            54 | 146 => self.chestTileEntities
                .entry(pos)
                .or_insert_with(|| TileEntityChest::new(pos))
                .receiveClientEvent(packet.getData1(), packet.getData2()),
            130 => self.enderChestTileEntities
                .entry(pos)
                .or_insert_with(|| TileEntityEnderChest::new(pos))
                .receiveClientEvent(packet.getData1(), packet.getData2()),
            219..=234 => self.shulkerBoxTileEntities
                .entry(pos)
                .or_insert_with(|| TileEntityShulkerBox::new(pos, currentId - 219))
                .receiveClientEvent(packet.getData1(), packet.getData2()),
            _ => false,
        };
        if handled { self.revision = self.revision.wrapping_add(1); }
        handled
    }


    pub fn getChunkFromChunkCoords(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.chunks.get(&(x, z))
    }

    pub fn getChunkFromChunkCoordsMut(&mut self, x: i32, z: i32) -> Option<&mut Chunk> {
        self.chunks.get_mut(&(x, z))
    }

    pub fn putChunk(&mut self, chunk: Chunk) {
        self.chunks
            .insert((chunk.xPosition, chunk.zPosition), chunk);
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn applyChunkData(&mut self, packet: &SPacketChunkData) -> Result<(), CodecError> {
        if packet.doChunkLoad() {
            self.doPreChunk(packet.getChunkX(), packet.getChunkZ(), true);
        }
        let mut chunk = self
            .chunks
            .remove(&(packet.getChunkX(), packet.getChunkZ()))
            .unwrap_or_else(|| Chunk::new(packet.getChunkX(), packet.getChunkZ()));
        let mut input = packet.getReadBuffer();
        let hasSkyLight = self.providerHasSkyLight();
        for sectionIndex in 0..16 {
            if packet.getExtractedSize() & (1 << sectionIndex) == 0 {
                if packet.doChunkLoad() {
                    chunk.setStorage(sectionIndex, None);
                }
                continue;
            }
            let data = BlockStateContainer::read(&mut input)?;
            if input.len() < 2048 {
                return Err(CodecError::UnexpectedEof);
            }
            let (blockLight, rest) = input.split_at(2048);
            input = rest;
            let blocklightArray = NibbleArray::fromStorage(blockLight.to_vec())
                .map_err(CodecError::InvalidData)?;
            let skylightArray = if hasSkyLight {
                if input.len() < 2048 {
                    return Err(CodecError::UnexpectedEof);
                }
                let (sky, rest) = input.split_at(2048);
                input = rest;
                Some(
                    NibbleArray::fromStorage(sky.to_vec())
                        .map_err(CodecError::InvalidData)?,
                )
            } else {
                None
            };
            chunk.setStorage(
                sectionIndex,
                Some(ExtendedBlockStorage::fromNetwork(
                    (sectionIndex * 16) as i32,
                    data,
                    blocklightArray,
                    skylightArray,
                )),
            );
        }
        if packet.doChunkLoad() {
            if input.len() < 256 {
                return Err(CodecError::UnexpectedEof);
            }
            chunk.setBiomeArray(&input[..256]);
            input = &input[256..];
        }
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread chunk bytes",
                input.len()
            )));
        }
        if packet.doChunkLoad() {
            let chunk_x = packet.getChunkX();
            let chunk_z = packet.getChunkZ();
            self.skullTileEntities.retain(|pos, _| {
                pos.x.div_euclid(16) != chunk_x || pos.z.div_euclid(16) != chunk_z
            });
            self.beaconTileEntities.retain(|pos, _| pos.x.div_euclid(16) != chunk_x || pos.z.div_euclid(16) != chunk_z);
            self.bedTileEntities.retain(|pos, _| pos.x.div_euclid(16) != chunk_x || pos.z.div_euclid(16) != chunk_z);
            self.chestTileEntities.retain(|pos, _| pos.x.div_euclid(16) != chunk_x || pos.z.div_euclid(16) != chunk_z);
            self.enderChestTileEntities.retain(|pos, _| pos.x.div_euclid(16) != chunk_x || pos.z.div_euclid(16) != chunk_z);
            self.enchantmentTableTileEntities.retain(|pos, _| pos.x.div_euclid(16) != chunk_x || pos.z.div_euclid(16) != chunk_z);
            self.endPortalTileEntities.retain(|pos, _| pos.x.div_euclid(16) != chunk_x || pos.z.div_euclid(16) != chunk_z);
            self.flowerPotTileEntities.retain(|pos, _| pos.x.div_euclid(16) != chunk_x || pos.z.div_euclid(16) != chunk_z);
            self.pistonTileEntities.retain(|pos, _| pos.x.div_euclid(16) != chunk_x || pos.z.div_euclid(16) != chunk_z);
            self.shulkerBoxTileEntities.retain(|pos, _| pos.x.div_euclid(16) != chunk_x || pos.z.div_euclid(16) != chunk_z);
            self.signTileEntities.retain(|pos, _| pos.x.div_euclid(16) != chunk_x || pos.z.div_euclid(16) != chunk_z);
        }
        self.putChunk(chunk);
        for tag in packet.getTileEntityTags() {
            if !self.applySkullTileEntityTag(tag) {
                self.applySpecialTileEntityTag(tag);
            }
        }
        Ok(())
    }

    pub fn invalidateRegionAndSetBlock(
        &mut self,
        pos: BlockPos,
        state: IBlockState,
    ) -> Result<IBlockState, String> {
        let chunkX = pos.x.div_euclid(16);
        let chunkZ = pos.z.div_euclid(16);
        let localX = pos.x.rem_euclid(16) as usize;
        let localZ = pos.z.rem_euclid(16) as usize;
        let hasSky = self.providerHasSkyLight();
        let chunk = self
            .chunks
            .entry((chunkX, chunkZ))
            .or_insert_with(|| Chunk::new(chunkX, chunkZ));
        let old = chunk.setBlockState(localX, pos.y.max(0) as usize, localZ, state, hasSky)?;
        if BlockSkull::BlockSkull::isBlockSkull(state) {
            // `BlockContainer#createNewTileEntity` creates a default
            // TileEntitySkull as soon as the block state enters the client
            // chunk. A later action-4 update replaces its type/profile/Rot.
            self.skullTileEntities
                .entry(pos)
                .or_insert_with(|| TileEntitySkull::new(pos));
        } else {
            self.skullTileEntities.remove(&pos);
        }
        let block_id = state.getBlockId();
        if block_id == TileEntityBeacon::BLOCK_ID {
            self.beaconTileEntities.entry(pos).or_insert_with(|| TileEntityBeacon::new(pos));
        } else {
            self.beaconTileEntities.remove(&pos);
        }
        if block_id == 26 {
            self.bedTileEntities.entry(pos).or_insert_with(|| TileEntityBed::new(pos));
        } else {
            self.bedTileEntities.remove(&pos);
        }
        if matches!(block_id, 54 | 146) {
            self.chestTileEntities.entry(pos).or_insert_with(|| TileEntityChest::new(pos));
        } else {
            self.chestTileEntities.remove(&pos);
        }
        if block_id == 116 {
            self.enchantmentTableTileEntities
                .entry(pos)
                .or_insert_with(|| TileEntityEnchantmentTable::new(pos));
        } else {
            self.enchantmentTableTileEntities.remove(&pos);
        }
        if block_id == 119 {
            self.endPortalTileEntities
                .entry(pos)
                .or_insert_with(|| TileEntityEndPortal::new(pos));
        } else {
            self.endPortalTileEntities.remove(&pos);
        }
        if block_id == crate::net::minecraft::block::BlockFlowerPot::BLOCK_ID {
            // `BlockContainer#createNewTileEntity` is invoked as soon as the
            // legacy state enters the client chunk. A subsequent action-5
            // packet supplies the authoritative item registry name/data.
            let legacy = state.getMetadata();
            self.flowerPotTileEntities
                .entry(pos)
                .or_insert_with(|| TileEntityFlowerPot::fromLegacyMetadata(pos, legacy));
        } else {
            self.flowerPotTileEntities.remove(&pos);
        }
        if block_id == 130 {
            self.enderChestTileEntities.entry(pos).or_insert_with(|| TileEntityEnderChest::new(pos));
        } else {
            self.enderChestTileEntities.remove(&pos);
        }
        if block_id != 36 {
            self.pistonTileEntities.remove(&pos);
        }
        if (219..=234).contains(&block_id) {
            self.shulkerBoxTileEntities
                .entry(pos)
                .or_insert_with(|| TileEntityShulkerBox::new(pos, block_id - 219));
        } else {
            self.shulkerBoxTileEntities.remove(&pos);
        }
        if matches!(block_id, 63 | 68) {
            self.signTileEntities
                .entry(pos)
                .or_insert_with(|| TileEntitySign::new(pos));
        } else {
            self.signTileEntities.remove(&pos);
        }
        self.revision = self.revision.wrapping_add(1);
        Ok(old)
    }

    pub fn handleBlockChange(&mut self, packet: &SPacketBlockChange) -> Result<(), String> {
        self.invalidateRegionAndSetBlock(packet.getBlockPosition(), packet.getBlockState())
            .map(|_| ())
    }

    pub fn handleMultiBlockChange(
        &mut self,
        packet: &SPacketMultiBlockChange,
    ) -> Result<(), String> {
        for update in packet.getChangedBlocks() {
            self.invalidateRegionAndSetBlock(update.getPos(), update.getBlockState())?;
        }
        Ok(())
    }

    /// MCP `Block#isReplaceable` for the source-confirmed replaceable states
    /// needed by `ItemBlock#canPlaceBlockOnSide`. Fluids and ordinary plants
    /// are not replaceable in 1.12.2; the explicit overrides below are.
    pub fn isBlockReplaceable(&self, pos: BlockPos) -> bool {
        let state = self.getBlockState(pos);
        match state.getBlockId() {
            0 | 31 | 32 | 106 => true, // air, tall grass, dead bush, vine
            78 => state.getMetadata() & 7 == 0, // one-layer snow only
            _ => false,
        }
    }

    /// Client-side equivalent of the placement gate in `World#func_190527_a`.
    /// The server remains authoritative over the actual block mutation. This
    /// method uses the received entity AABBs and the placed block's default
    /// collision shape instead of treating every packet as a successful place.
    pub fn mayPlace(
        &self,
        block: crate::net::minecraft::block::Block::Block,
        pos: BlockPos,
        skipCollisionCheck: bool,
        _side: EnumFacing,
        player: Option<&crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP>,
    ) -> bool {
        if !(0..256).contains(&pos.y) || !self.isBlockReplaceable(pos) {
            return false;
        }

        if !skipCollisionCheck {
            let state = IBlockState::fromGlobalStateId(
                crate::net::minecraft::block::Block::Block::getIdFromBlock(block) << 4,
            );
            for local in block.getCollisionBoxes(state) {
                let placed = local.offset(pos.x as f64, pos.y as f64, pos.z as f64);
                if player.is_some_and(|player| !player.entity.isDead && placed.intersects(player.entity.boundingBox)) {
                    return false;
                }
                if self.remotePlayers.values().any(|entity| !entity.entity.isDead && placed.intersects(entity.entity.boundingBox)) {
                    return false;
                }
                if self.nonPlayerEntities.values().any(|entity| !entity.entity.isDead && placed.intersects(entity.entity.boundingBox)) {
                    return false;
                }
            }
        }
        true
    }

    pub fn getBlockState(&self, pos: BlockPos) -> IBlockState {
        if !(0..256).contains(&pos.y) {
            return IBlockState::default();
        }
        let chunkX = pos.x.div_euclid(16);
        let chunkZ = pos.z.div_euclid(16);
        self.chunks
            .get(&(chunkX, chunkZ))
            .map(|chunk| {
                chunk.getBlockState(
                    pos.x.rem_euclid(16) as usize,
                    pos.y as usize,
                    pos.z.rem_euclid(16) as usize,
                )
            })
            .unwrap_or_default()
    }

    pub fn getLightFor(&self, lightType: EnumSkyBlock, mut pos: BlockPos) -> u8 {
        // MCP `World.getLightFor` clamps negative Y to the bottom layer before
        // resolving the chunk. Positions above the build limit return the
        // channel's default value.
        if pos.y < 0 {
            pos.y = 0;
        }
        if pos.y >= 256 {
            return lightType.defaultLightValue();
        }
        if lightType == EnumSkyBlock::Sky && !self.providerHasSkyLight() {
            return 0;
        }
        let chunkX = pos.x.div_euclid(16);
        let chunkZ = pos.z.div_euclid(16);
        let Some(chunk) = self.chunks.get(&(chunkX, chunkZ)) else {
            return lightType.defaultLightValue();
        };
        let Some(storage) = chunk.getBlockStorageArray()[pos.y as usize >> 4].as_ref() else {
            // A complete height map has not yet been ported. For received
            // multiplayer chunks, absent sections above terrain are sky-lit;
            // block light remains zero, matching the common Chunk branch.
            return if lightType == EnumSkyBlock::Sky && self.providerHasSkyLight() {
                15
            } else {
                0
            };
        };
        let x = pos.x.rem_euclid(16) as usize;
        let y = pos.y as usize & 15;
        let z = pos.z.rem_euclid(16) as usize;
        match lightType {
            EnumSkyBlock::Sky => storage.getExtSkylightValue(x, y, z),
            EnumSkyBlock::Block => storage.getExtBlocklightValue(x, y, z),
        }
    }

    /// MCP `World.getLightFromNeighborsFor` / `ChunkCache.getLightForExt`.
    pub fn getLightFromNeighborsFor(&self, lightType: EnumSkyBlock, mut pos: BlockPos) -> u8 {
        if lightType == EnumSkyBlock::Sky && !self.providerHasSkyLight() {
            return 0;
        }
        if pos.y < 0 {
            pos.y = 0;
        }
        if pos.y >= 256 || !self.isBlockLoaded(pos) {
            return lightType.defaultLightValue();
        }
        if self.getBlockState(pos).getBlock().useNeighborBrightness() {
            let mut result = 0;
            for facing in crate::net::minecraft::util::EnumFacing::EnumFacing::VALUES {
                let (dx, dy, dz) = facing.offsets();
                result = result.max(self.getLightFor(
                    lightType,
                    BlockPos::new(pos.x + dx, pos.y + dy, pos.z + dz),
                ));
                if result >= 15 {
                    break;
                }
            }
            result
        } else {
            self.getLightFor(lightType, pos)
        }
    }

    /// MCP `World.getCombinedLight`: sky in bits 20..23 and block in 4..7.
    pub fn getCombinedLight(&self, pos: BlockPos, lightValue: u8) -> u32 {
        let sky = self.getLightFromNeighborsFor(EnumSkyBlock::Sky, pos);
        let block = self
            .getLightFromNeighborsFor(EnumSkyBlock::Block, pos)
            .max(lightValue.min(15));
        ((sky as u32) << 20) | ((block as u32) << 4)
    }

    pub fn getCombinedLightLevel(&self, pos: BlockPos) -> u8 {
        let packed = self.getCombinedLight(pos, 0);
        ((packed >> 20) as u8 & 15).max(((packed >> 4) as u8) & 15)
    }
    /// Port of `World#containsAnyLiquid`. The scan bounds deliberately use
    /// floor(min) / ceil(max), matching the inclusive block-volume test in
    /// Minecraft 1.12.2 rather than treating liquids as collision boxes.
    pub fn containsAnyLiquid(&self, aabb: AxisAlignedBB) -> bool {
        let min_x = aabb.min_x.floor() as i32;
        let max_x = aabb.max_x.ceil() as i32;
        let min_y = aabb.min_y.floor() as i32;
        let max_y = aabb.max_y.ceil() as i32;
        let min_z = aabb.min_z.floor() as i32;
        let max_z = aabb.max_z.ceil() as i32;
        for x in min_x..max_x {
            for z in min_z..max_z {
                if !self.isBlockLoaded(BlockPos::new(x, 64, z)) {
                    continue;
                }
                for y in min_y..max_y {
                    if BlockLiquid::isLiquid(self.getBlockState(BlockPos::new(x, y, z))) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Port of `World#isMaterialInBB` for vanilla liquid materials.
    pub fn isMaterialInBB(&self, aabb: AxisAlignedBB, material: LiquidMaterial) -> bool {
        let min_x = aabb.min_x.floor() as i32;
        let max_x = aabb.max_x.ceil() as i32;
        let min_y = aabb.min_y.floor() as i32;
        let max_y = aabb.max_y.ceil() as i32;
        let min_z = aabb.min_z.floor() as i32;
        let max_z = aabb.max_z.ceil() as i32;
        for x in min_x..max_x {
            for z in min_z..max_z {
                if !self.isBlockLoaded(BlockPos::new(x, 64, z)) {
                    continue;
                }
                for y in min_y..max_y {
                    if material.contains(self.getBlockState(BlockPos::new(x, y, z))) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Direct port of `World#handleMaterialAcceleration`. Flow is accumulated
    /// from every intersecting liquid block and normalized once before the
    /// vanilla 0.014 acceleration is applied to the entity.
    pub fn handleMaterialAcceleration(
        &self,
        aabb: AxisAlignedBB,
        material: LiquidMaterial,
        entity: &mut Entity,
    ) -> bool {
        let min_x = aabb.min_x.floor() as i32;
        let max_x = aabb.max_x.ceil() as i32;
        let min_y = aabb.min_y.floor() as i32;
        let max_y = aabb.max_y.ceil() as i32;
        let min_z = aabb.min_z.floor() as i32;
        let max_z = aabb.max_z.ceil() as i32;

        // `World#isAreaLoaded` is a hard guard in the source method. A missing
        // client column must not be interpreted as air and produce a false
        // edge-current while chunks are streaming.
        for x in min_x..max_x {
            for z in min_z..max_z {
                if !self.isBlockLoaded(BlockPos::new(x, 64, z)) {
                    return false;
                }
            }
        }

        let mut found = false;
        let mut acceleration = Vec3d::ZERO;
        for x in min_x..max_x {
            for z in min_z..max_z {
                for y in min_y..max_y {
                    let pos = BlockPos::new(x, y, z);
                    let state = self.getBlockState(pos);
                    if !material.contains(state) {
                        continue;
                    }
                    let surface = y as f64 + 1.0
                        - BlockLiquid::getLiquidHeightPercent(BlockLiquid::getLevel(state)) as f64;
                    if max_y as f64 >= surface {
                        found = true;
                        acceleration = acceleration + BlockLiquid::getFlow(self, pos, state);
                    }
                }
            }
        }
        if acceleration.length_squared() > 0.0 && entity.isPushedByWater() {
            let acceleration = acceleration.normalize().scale(0.014);
            entity.motionX += acceleration.x;
            entity.motionY += acceleration.y;
            entity.motionZ += acceleration.z;
        }
        found
    }

    /// Block-volume scan used by MCP `Entity#doBlockCollisions`. The bounds
    /// contract by 0.001 before this call, matching the pooled BlockPos loop.
    pub fn intersectsBlockId(&self, aabb: AxisAlignedBB, blockId: i32) -> bool {
        let minX = aabb.min_x.floor() as i32;
        let maxX = aabb.max_x.floor() as i32;
        let minY = aabb.min_y.floor() as i32;
        let maxY = aabb.max_y.floor() as i32;
        let minZ = aabb.min_z.floor() as i32;
        let maxZ = aabb.max_z.floor() as i32;
        for x in minX..=maxX {
            for y in minY..=maxY {
                for z in minZ..=maxZ {
                    if self.getBlockState(BlockPos::new(x, y, z)).getBlockId() == blockId {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Entity-aware half of MCP 1.12.2 `World#getCollisionBoxes`. The block
    /// scan remains owned by `getCollisionBoxes`; this method then adds the
    /// exact entity boxes exposed by `getCollisionBoundingBox` and, for a
    /// moving boat/minecart, `getCollisionBox(other)`.
    pub fn getCollisionBoxesForEntity(
        &self,
        movingEntityId: i32,
        movingRidingEntityId: Option<i32>,
        movingCollidesWithPushableEntities: bool,
        aabb: AxisAlignedBB,
    ) -> Vec<AxisAlignedBB> {
        let mut collisions = self.getCollisionBoxes(aabb);
        let search = aabb.expand_xyz(0.25);
        let movingRoot = self.lowestRidingEntityId(movingEntityId, movingRidingEntityId);

        for (&entityId, player) in &self.remotePlayers {
            if entityId == movingEntityId || !player.entity.boundingBox.intersects(search) {
                continue;
            }
            let targetRoot = self.lowestRidingEntityId(entityId, player.entity.ridingEntityId);
            if targetRoot == movingRoot { continue; }
            if movingCollidesWithPushableEntities && !player.entity.isDead {
                let bounds = player.entity.boundingBox;
                if bounds.intersects(aabb) { collisions.push(bounds); }
            }
        }

        for (&entityId, entity) in &self.nonPlayerEntities {
            if entityId == movingEntityId || !entity.entity.boundingBox.intersects(search) {
                continue;
            }
            let targetRoot = self.lowestRidingEntityId(entityId, entity.entity.ridingEntityId);
            if targetRoot == movingRoot { continue; }

            // Only EntityBoat and EntityShulker override
            // `getCollisionBoundingBox` in 1.12.2. EntityMinecart explicitly
            // returns null and therefore is not a standable collision box.
            let targetOwnsCollisionBox = matches!(
                &entity.kind,
                crate::net::minecraft::client::entity::EntityOtherClient::ClientEntityKind::Object {
                    objectType: crate::net::minecraft::client::entity::EntityOtherClient::ObjectSpawnType::Boat,
                    ..
                }
            ) || matches!(
                &entity.kind,
                crate::net::minecraft::client::entity::EntityOtherClient::ClientEntityKind::Mob { entityType }
                    if entityType.registryName == "shulker"
            );
            if targetOwnsCollisionBox {
                let bounds = entity.entity.boundingBox;
                if bounds.intersects(aabb) { collisions.push(bounds); }
            }

            // EntityBoat/EntityMinecart#getCollisionBox returns the other
            // entity's box only when that other entity can be pushed.
            if movingCollidesWithPushableEntities && entity.canBePushed() {
                let bounds = entity.entity.boundingBox;
                if bounds.intersects(aabb) { collisions.push(bounds); }
            }
        }
        collisions
    }

    /// MCP block slipperiness used by EntityItem/EntityXPOrb ground drag.
    pub fn blockSlipperiness(&self, pos: BlockPos) -> f32 {
        match self.getBlockState(pos).getBlockId() {
            79 | 174 => 0.98,
            id if id == BlockSlime::BLOCK_ID => BlockSlime::SLIPPERINESS,
            _ => 0.6,
        }
    }

    /// Block-only collection corresponding to `World#func_191504_a`.
    /// Callers moving a concrete entity use `getCollisionBoxesForEntity` so
    /// boat/shulker collision boxes and vehicle push boxes are added afterward.
    pub fn getCollisionBoxes(&self, aabb: AxisAlignedBB) -> Vec<AxisAlignedBB> {
        // Block scan bounds and edge/corner skipping mirror
        // `World.func_191504_a` in MCP 1.12.2. World-border substitution and
        // entity boxes remain a separate pass in `getCollisionBoxesForEntity`.
        let min_x = aabb.min_x.floor() as i32 - 1;
        let max_x = aabb.max_x.ceil() as i32 + 1;
        let min_y = aabb.min_y.floor() as i32 - 1;
        let max_y = aabb.max_y.ceil() as i32 + 1;
        let min_z = aabb.min_z.floor() as i32 - 1;
        let max_z = aabb.max_z.ceil() as i32 + 1;
        let mut collisions = Vec::new();

        for x in min_x..max_x {
            for z in min_z..max_z {
                let outer_x = x == min_x || x == max_x - 1;
                let outer_z = z == min_z || z == max_z - 1;
                if outer_x && outer_z {
                    continue;
                }
                if !self.isBlockLoaded(BlockPos::new(x, 64, z)) {
                    continue;
                }

                for y in min_y..max_y {
                    if (outer_x || outer_z) && y == max_y - 1 {
                        continue;
                    }
                    let pos = BlockPos::new(x, y, z);
                    let state = self.getBlockState(pos);
                    let localBoxes = self.getBlockCollisionBoxesLocal(pos, state);
                    for local in localBoxes {
                        let world_box = local.offset(x as f64, y as f64, z as f64);
                        if world_box.intersects(aabb) {
                            collisions.push(world_box);
                        }
                    }
                }
            }
        }
        collisions
    }

    pub fn getBlockCollisionBoxesAt(&self, pos: BlockPos) -> Vec<AxisAlignedBB> {
        let state = self.getBlockState(pos);
        self.getBlockCollisionBoxesLocal(pos, state)
            .into_iter()
            .map(|bounds| bounds.offset(pos.x as f64, pos.y as f64, pos.z as f64))
            .collect()
    }

    fn getBlockCollisionBoxesLocal(&self, pos: BlockPos, state: IBlockState) -> Vec<AxisAlignedBB> {
        if state.getBlockId() == 36 {
            // MCP `BlockPistonMoving#addCollisionBoxToList` delegates to its
            // TileEntityPiston. The moving block therefore occupies a swept,
            // progress-dependent shape instead of the static one-block cube
            // that caused client standing and server correction divergence.
            self.pistonTileEntities.get(&pos)
                .map(TileEntityPiston::collisionBoxesLocal)
                .unwrap_or_default()
        } else if BlockDoor::isBlockDoor(state) {
            vec![BlockDoor::getBoundingBox(state, self, pos)]
        } else if BlockEndRod::isBlockEndRod(state) {
            vec![BlockEndRod::getBoundingBox(state)]
        } else if BlockPistonBase::BlockPistonBase::isPistonBase(state) {
            // MCP `BlockPistonBase#addCollisionBoxToList` contributes the
            // current base bounding box as the sole collision component.
            vec![BlockPistonBase::BlockPistonBase::getBoundingBox(state)]
        } else if BlockPistonExtension::BlockPistonExtension::isPistonHead(state) {
            // MCP `BlockPistonExtension#addCollisionBoxToList` contributes both
            // the head plate and the arm. `getBoundingBox` alone is only the
            // selection/model bound and must not be used for entity collision.
            BlockPistonExtension::BlockPistonExtension::collisionBoxes(state, false)
        } else if BlockEndPortalFrame::BlockEndPortalFrame::isBlockEndPortalFrame(state) {
            BlockEndPortalFrame::BlockEndPortalFrame::getCollisionBoxes(state)
        } else if BlockSkull::BlockSkull::isBlockSkull(state) {
            BlockSkull::BlockSkull::getCollisionBoxes(state)
        } else if BlockLadder::isBlockLadder(state) {
            vec![BlockLadder::getBoundingBox(state)]
        } else if BlockTrapDoor::isBlockTrapDoor(state) {
            vec![BlockTrapDoor::getBoundingBox(state)]
        } else if BlockTorch::isBlockTorch(state)
            || BlockRailBase::isRailBlock(state)
            || BlockSign::isBlockSign(state)
            || BlockWeb::isBlockWeb(state)
        {
            Vec::new()
        } else if BlockStairs::isBlockStairs(state) {
            let shape = BlockStairs::getStairsShape(state, self, pos);
            BlockStairs::getCollisionBoxList(state, shape)
        } else if BlockFence::isBlockFence(state) {
            BlockFence::getCollisionBoxes(BlockFence::connectionMask(state, self, pos))
        } else if BlockPane::isBlockPane(state) {
            BlockPane::getCollisionBoxes(BlockPane::connectionMask(self, pos))
        } else if BlockWall::isBlockWall(state) {
            BlockWall::getCollisionBoxes(BlockWall::connectionMask(self, pos))
        } else if BlockFenceGate::isBlockFenceGate(state) {
            BlockFenceGate::getCollisionBoxes(state)
        } else {
            state.getBlock().getCollisionBoxes(state)
        }
    }

    /// Block-local selection/ray bounds corresponding to MCP 1.12.2
    /// `IBlockState#getBoundingBox`. This is deliberately separate from
    /// entity collision: fences and walls collide to 1.5 blocks high while
    /// their selected model bounds remain at the visible-model height.
    fn getBlockSelectionBoundingBoxLocal(
        &self,
        pos: BlockPos,
        state: IBlockState,
    ) -> Option<AxisAlignedBB> {
        let block = state.getBlock();
        if block.isAir() {
            return None;
        }
        let id = crate::net::minecraft::block::Block::Block::getIdFromBlock(block);
        let bounds = if BlockDoor::isBlockDoor(state) {
            BlockDoor::getBoundingBox(state, self, pos)
        } else if BlockTorch::isBlockTorch(state) {
            BlockTorch::getBoundingBox(state)
        } else if BlockLadder::isBlockLadder(state) {
            BlockLadder::getBoundingBox(state)
        } else if BlockRailBase::isRailBlock(state) {
            BlockRailBase::getBoundingBox(state)
        } else if BlockSign::isBlockSign(state) {
            BlockSign::getBoundingBox(state)
        } else if BlockTrapDoor::isBlockTrapDoor(state) {
            BlockTrapDoor::getBoundingBox(state)
        } else if BlockVine::isBlockVine(state) {
            BlockVine::getBoundingBox(state, self, pos)
        } else if BlockWeb::isBlockWeb(state) {
            BlockWeb::getBoundingBox()
        } else if BlockLever::isBlockLever(state) {
            BlockLever::getBoundingBox(state)
        } else if BlockButton::isBlockButton(state) {
            BlockButton::getBoundingBox(state)
        } else if BlockEndRod::isBlockEndRod(state) {
            BlockEndRod::getBoundingBox(state)
        } else if BlockPistonBase::BlockPistonBase::isPistonBase(state) {
            // MCP `BlockPistonBase#getBoundingBox`: block selection uses the
            // single state-dependent base box, not the collision-list wrapper.
            BlockPistonBase::BlockPistonBase::getBoundingBox(state)
        } else if BlockPistonExtension::BlockPistonExtension::isPistonHead(state) {
            // MCP `BlockPistonExtension#getBoundingBox`: block selection uses
            // only the oriented head plate. The arm remains a second box only
            // in `getBlockCollisionBoxesLocal`/`addCollisionBoxToList`.
            BlockPistonExtension::BlockPistonExtension::getBoundingBox(state)
        } else if BlockEndPortalFrame::BlockEndPortalFrame::isBlockEndPortalFrame(state) {
            BlockEndPortalFrame::BlockEndPortalFrame::getBoundingBox()
        } else if BlockSkull::BlockSkull::isBlockSkull(state) {
            BlockSkull::BlockSkull::getBoundingBox(state)
        } else if BlockStairs::isBlockStairs(state) {
            // BlockStairs delegates getBoundingBox to the model block. The
            // model block used by all vanilla stairs has FULL_BLOCK_AABB.
            AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
        } else if BlockFence::isBlockFence(state) {
            BlockFence::getBoundingBox(BlockFence::connectionMask(state, self, pos))
        } else if BlockPane::isBlockPane(state) {
            BlockPane::getBoundingBox(BlockPane::connectionMask(self, pos))
        } else if BlockWall::isBlockWall(state) {
            BlockWall::getBoundingBox(BlockWall::connectionMask(self, pos))
        } else if BlockFenceGate::isBlockFenceGate(state) {
            BlockFenceGate::getBoundingBox(state, self, pos)
        } else if matches!(id, 54 | 146) {
            // BlockChest#getBoundingBox after checkForSurroundingChests.
            // Trapped and normal chests connect only to their own block type.
            let same = |at: BlockPos| self.getBlockState(at).getBlockId() == id;
            if same(pos.north(1)) {
                AxisAlignedBB::new(0.0625, 0.0, 0.0, 0.9375, 0.875, 0.9375)
            } else if same(pos.south(1)) {
                AxisAlignedBB::new(0.0625, 0.0, 0.0625, 0.9375, 0.875, 1.0)
            } else if same(pos.west(1)) {
                AxisAlignedBB::new(0.0, 0.0, 0.0625, 0.9375, 0.875, 0.9375)
            } else if same(pos.east(1)) {
                AxisAlignedBB::new(0.0625, 0.0, 0.0625, 1.0, 0.875, 0.9375)
            } else {
                AxisAlignedBB::new(0.0625, 0.0, 0.0625, 0.9375, 0.875, 0.9375)
            }
        } else if id == 130 {
            // Ender chests never join into a double chest.
            AxisAlignedBB::new(0.0625, 0.0, 0.0625, 0.9375, 0.875, 0.9375)
        } else if matches!(id, 219..=234) {
            // BlockShulkerBox#getBoundingBox returns FULL_BLOCK_AABB when its
            // TileEntity is absent. TileEntity animation state is not yet
            // stored by WorldClient, so preserve that exact source fallback.
            AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
        } else {
            let mut boxes = block.getCollisionBoxes(state);
            if boxes.is_empty() {
                boxes = self.getNonCollidingSelectionBoxesLocal(state, id);
            }
            let mut iter = boxes.into_iter();
            let first = iter.next()?;
            iter.fold(first, AxisAlignedBB::union)
        };
        Some(bounds)
    }

    /// Exact world-space selected box used by `RenderGlobal.drawSelectionBox`.
    pub fn getSelectedBoundingBox(&self, pos: BlockPos) -> Option<AxisAlignedBB> {
        let state = self.getBlockState(pos);
        self.getBlockSelectionBoundingBoxLocal(pos, state).map(|bounds| {
            bounds.offset(pos.x as f64, pos.y as f64, pos.z as f64)
        })
    }

    fn getNonCollidingSelectionBoxesLocal(
        &self,
        state: IBlockState,
        id: i32,
    ) -> Vec<AxisAlignedBB> {
        match id {
            6 | 31 | 32 | 37..=40 | 104..=106 | 115 | 127 | 141 | 142 | 175 => {
                vec![AxisAlignedBB::new(0.1, 0.0, 0.1, 0.9, 0.8, 0.9)]
            }
            50 | 75 | 76 => vec![AxisAlignedBB::new(0.4, 0.0, 0.4, 0.6, 0.6, 0.6)],
            55 => vec![AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 0.0625, 1.0)],
            59 | 207 => vec![AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 0.25, 1.0)],
            63 | 68 => vec![AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)],
            66 | 157 => vec![AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 0.125, 1.0)],
            78 => vec![AxisAlignedBB::new(
                0.0,
                0.0,
                0.0,
                1.0,
                ((state.getMetadata() & 7) + 1) as f64 * 0.125,
                1.0,
            )],
            83 => vec![AxisAlignedBB::new(0.125, 0.0, 0.125, 0.875, 1.0, 0.875)],
            90 | 119 | 209 => vec![AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)],
            111 => vec![AxisAlignedBB::new(0.0625, 0.0, 0.0625, 0.9375, 0.09375, 0.9375)],
            132 => vec![AxisAlignedBB::new(0.25, 0.0, 0.25, 0.75, 1.0, 0.75)],
            _ => Vec::new(),
        }
    }

    fn collisionRayTrace(
        &self,
        pos: BlockPos,
        start: Vec3d,
        end: Vec3d,
        stopOnLiquid: bool,
    ) -> Option<RayTraceResult> {
        let state = self.getBlockState(pos);
        let block = state.getBlock();
        if block.isAir() {
            return None;
        }
        let id = crate::net::minecraft::block::Block::Block::getIdFromBlock(block);
        if matches!(id, 8..=11) {
            if !stopOnLiquid || state.getMetadata() & 7 != 0 {
                return None;
            }
            return AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
                .offset(pos.x as f64, pos.y as f64, pos.z as f64)
                .calculate_intercept(start, end)
                .map(|(hit, side)| RayTraceResult::block(hit, side, pos));
        }

        // MCP `Block.collisionRayTrace` uses one selected bounding box,
        // not the entity-collision list. This also prevents fence/wall ray
        // hits from extending to their 1.5-block collision height.
        let boxes = self
            .getBlockSelectionBoundingBoxLocal(pos, state)
            .into_iter()
            .collect::<Vec<_>>();

        let mut closest: Option<(Vec3d, EnumFacing)> = None;
        for local in boxes {
            let world = local.offset(pos.x as f64, pos.y as f64, pos.z as f64);
            if let Some(candidate) = world.calculate_intercept(start, end) {
                if closest.map_or(true, |current| {
                    start.square_distance_to(candidate.0) < start.square_distance_to(current.0)
                }) {
                    closest = Some(candidate);
                }
            }
        }
        closest.map(|(hit, side)| RayTraceResult::block(hit, side, pos))
    }

    /// Direct port of MCP 1.12.2 `World.rayTraceBlocks` DDA traversal.
    pub fn rayTraceBlocks(
        &self,
        mut start: Vec3d,
        end: Vec3d,
        stopOnLiquid: bool,
        ignoreBlockWithoutBoundingBox: bool,
        returnLastUncollidableBlock: bool,
    ) -> Option<RayTraceResult> {
        if [start.x, start.y, start.z, end.x, end.y, end.z]
            .iter()
            .any(|value| value.is_nan())
        {
            return None;
        }

        let end_x = end.x.floor() as i32;
        let end_y = end.y.floor() as i32;
        let end_z = end.z.floor() as i32;
        let mut x = start.x.floor() as i32;
        let mut y = start.y.floor() as i32;
        let mut z = start.z.floor() as i32;
        let initial = BlockPos::new(x, y, z);
        if !ignoreBlockWithoutBoundingBox || !self.getBlockCollisionBoxesLocal(initial, self.getBlockState(initial)).is_empty() {
            if let Some(hit) = self.collisionRayTrace(initial, start, end, stopOnLiquid) {
                return Some(hit);
            }
        }

        let mut last_miss = None;
        for _ in 0..=200 {
            if [start.x, start.y, start.z].iter().any(|value| value.is_nan()) {
                return None;
            }
            if x == end_x && y == end_y && z == end_z {
                return returnLastUncollidableBlock.then_some(last_miss).flatten();
            }

            let (step_x, boundary_x) = if end_x > x {
                (true, x as f64 + 1.0)
            } else if end_x < x {
                (true, x as f64)
            } else {
                (false, 999.0)
            };
            let (step_y, boundary_y) = if end_y > y {
                (true, y as f64 + 1.0)
            } else if end_y < y {
                (true, y as f64)
            } else {
                (false, 999.0)
            };
            let (step_z, boundary_z) = if end_z > z {
                (true, z as f64 + 1.0)
            } else if end_z < z {
                (true, z as f64)
            } else {
                (false, 999.0)
            };

            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let dz = end.z - start.z;
            let mut tx = if step_x { (boundary_x - start.x) / dx } else { 999.0 };
            let mut ty = if step_y { (boundary_y - start.y) / dy } else { 999.0 };
            let mut tz = if step_z { (boundary_z - start.z) / dz } else { 999.0 };
            if tx == -0.0 { tx = -1.0E-4; }
            if ty == -0.0 { ty = -1.0E-4; }
            if tz == -0.0 { tz = -1.0E-4; }

            let side;
            if tx < ty && tx < tz {
                side = if end_x > x { EnumFacing::West } else { EnumFacing::East };
                start = Vec3d::new(boundary_x, start.y + dy * tx, start.z + dz * tx);
            } else if ty < tz {
                side = if end_y > y { EnumFacing::Down } else { EnumFacing::Up };
                start = Vec3d::new(start.x + dx * ty, boundary_y, start.z + dz * ty);
            } else {
                side = if end_z > z { EnumFacing::North } else { EnumFacing::South };
                start = Vec3d::new(start.x + dx * tz, start.y + dy * tz, boundary_z);
            }

            x = start.x.floor() as i32 - if side == EnumFacing::East { 1 } else { 0 };
            y = start.y.floor() as i32 - if side == EnumFacing::Up { 1 } else { 0 };
            z = start.z.floor() as i32 - if side == EnumFacing::South { 1 } else { 0 };
            let pos = BlockPos::new(x, y, z);
            let state = self.getBlockState(pos);
            let has_box = !self.getBlockCollisionBoxesLocal(pos, state).is_empty();
            if !ignoreBlockWithoutBoundingBox || has_box || matches!(crate::net::minecraft::block::Block::Block::getIdFromBlock(state.getBlock()), 90) {
                if let Some(hit) = self.collisionRayTrace(pos, start, end, stopOnLiquid) {
                    return Some(hit);
                }
                last_miss = Some(RayTraceResult::miss(start, side, pos));
            }
        }
        if returnLastUncollidableBlock { last_miss } else { None }
    }

    pub fn isBlockLoaded(&self, pos: BlockPos) -> bool {
        self.chunks.contains_key(&(pos.x.div_euclid(16), pos.z.div_euclid(16)))
    }

    pub fn getSlipperiness(&self, pos: BlockPos) -> f32 {
        self.getBlockState(pos).getBlock().getSlipperiness()
    }

    pub fn loadedChunkCount(&self) -> usize { self.chunks.len() }
    pub const fn getDimension(&self) -> i32 { self.provider.getDimension() }
    pub const fn revision(&self) -> u64 { self.revision }
    pub fn loadedChunks(&self) -> impl Iterator<Item = &Chunk> { self.chunks.values() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::com::mojang::authlib::GameProfile::GameProfile;
    use crate::net::minecraft::network::Packet::RawPacket;
    use crate::net::minecraft::network::PacketBuffer::{write_bool,write_i32_be,write_i64_be,write_var_i32};
    use crate::net::minecraft::network::play::server::SPacketChunkData::SPacketChunkData;
    use crate::net::minecraft::world::GameType::GameType;
    use uuid::Uuid;

    #[test]
    fn late_tab_entry_is_cached_and_refreshed_on_existing_remote_player() {
        let mut world = WorldClient::new(0);
        let uniqueId = Uuid::parse_str("12345678-1234-5678-9abc-def012345678").unwrap();
        let profile = GameProfile::new(Some(uniqueId), "LateBot");
        world.addEntityToWorld(
            9,
            EntityOtherPlayerMP::new(9, uniqueId, profile.clone()),
        );
        let info = NetworkPlayerInfo::new(profile, GameType::Survival, 0, None);

        assert!(world.cachePlayerInfo(uniqueId, info));
        assert!(world.cachePlayerInfo(
            uniqueId,
            NetworkPlayerInfo::new(
                GameProfile::new(Some(uniqueId), "LateBot"),
                GameType::Creative,
                1,
                None,
            ),
        ));
        let cached = world.getEntityByID(9).unwrap().getPlayerInfo().unwrap();
        assert_eq!(cached.getGameProfile().getName(), "LateBot");
        assert_eq!(cached.getGameType(), GameType::Creative);
        assert_eq!(cached.getResponseTime(), 1);
    }

    #[test]
    fn full_chunk_load_decodes_palette_light_and_biomes() {
        let mut section=Vec::new();
        section.push(4); write_var_i32(1,&mut section); write_var_i32(5,&mut section); write_var_i32(256,&mut section);
        for _ in 0..256 { write_i64_be(0,&mut section); }
        section.extend(std::iter::repeat(0x21).take(2048));
        section.extend(std::iter::repeat(0x43).take(2048));
        section.extend(0_u8..=255_u8);
        let mut payload=Vec::new();
        write_i32_be(2,&mut payload); write_i32_be(-3,&mut payload); write_bool(true,&mut payload);
        write_var_i32(1,&mut payload); write_var_i32(section.len() as i32,&mut payload); payload.extend_from_slice(&section); write_var_i32(0,&mut payload);
        let packet=SPacketChunkData::readPacketData(&RawPacket::new(0x20,payload)).unwrap();
        let mut world=WorldClient::new(0); world.applyChunkData(&packet).unwrap();
        let chunk=world.getChunkFromChunkCoords(2,-3).unwrap();
        assert_eq!(chunk.getGlobalStateId(0,0,0),5);
        let storage=chunk.getBlockStorageArray()[0].as_ref().unwrap();
        assert_eq!(storage.getExtBlocklightValue(0,0,0),1);
        assert_eq!(storage.getExtBlocklightValue(1,0,0),2);
        assert_eq!(storage.getExtSkylightValue(0,0,0),3);
        assert_eq!(storage.getExtSkylightValue(1,0,0),4);
        assert_eq!(chunk.getBiomeArray()[255],255);
    }


    #[test]
    fn end_full_chunk_omits_skylight_and_consumes_packet_exactly() {
        let mut section = Vec::new();
        section.push(4);
        write_var_i32(1, &mut section);
        write_var_i32(5, &mut section);
        write_var_i32(256, &mut section);
        for _ in 0..256 { write_i64_be(0, &mut section); }
        section.extend(std::iter::repeat(0x21).take(2048));
        section.extend(0_u8..=255_u8);

        let mut payload = Vec::new();
        write_i32_be(7, &mut payload);
        write_i32_be(9, &mut payload);
        write_bool(true, &mut payload);
        write_var_i32(1, &mut payload);
        write_var_i32(section.len() as i32, &mut payload);
        payload.extend_from_slice(&section);
        write_var_i32(0, &mut payload);

        let packet = SPacketChunkData::readPacketData(&RawPacket::new(0x20, payload)).unwrap();
        let mut world = WorldClient::new(1);
        world.applyChunkData(&packet).unwrap();
        let chunk = world.getChunkFromChunkCoords(7, 9).unwrap();
        let storage = chunk.getBlockStorageArray()[0].as_ref().unwrap();
        assert_eq!(storage.getGlobalStateId(0, 0, 0), 5);
        assert_eq!(storage.getExtBlocklightValue(0, 0, 0), 1);
        assert_eq!(storage.getExtBlocklightValue(1, 0, 0), 2);
        assert_eq!(storage.getExtSkylightValue(0, 0, 0), 0);
        assert_eq!(chunk.getBiomeArray()[255], 255);
    }

    #[test]
    fn selected_fence_box_uses_visible_height_not_collision_height() {
        let mut world = WorldClient::new(0);
        let pos = BlockPos::new(0, 64, 0);
        world.invalidateRegionAndSetBlock(pos, IBlockState::fromGlobalStateId(85 << 4)).unwrap();
        let selected = world.getSelectedBoundingBox(pos).expect("fence selected box");
        assert_eq!(selected.max_y, 65.0);
        let collision = world.getCollisionBoxes(AxisAlignedBB::new(0.4, 64.0, 0.4, 0.6, 66.0, 0.6));
        assert!(collision.iter().any(|bounds| bounds.max_y == 65.5));
    }

    #[test]
    fn selected_double_chest_expands_only_toward_same_chest_type() {
        let mut world = WorldClient::new(0);
        let left = BlockPos::new(0, 64, 0);
        world.invalidateRegionAndSetBlock(left, IBlockState::fromGlobalStateId(54 << 4)).unwrap();
        world.invalidateRegionAndSetBlock(left.east(1), IBlockState::fromGlobalStateId(54 << 4)).unwrap();
        let selected = world.getSelectedBoundingBox(left).expect("double chest selected box");
        assert_eq!((selected.min_x, selected.max_x), (0.0625, 1.0));
        assert_eq!((selected.min_z, selected.max_z), (0.0625, 0.9375));
    }

    #[test]
    fn stair_selected_box_is_full_model_block() {
        let mut world = WorldClient::new(0);
        let pos = BlockPos::new(2, 64, 2);
        world.invalidateRegionAndSetBlock(pos, IBlockState::fromGlobalStateId(53 << 4)).unwrap();
        assert_eq!(world.getSelectedBoundingBox(pos), Some(AxisAlignedBB::from_block(pos)));
    }

    #[test]
    fn negative_server_time_disables_daylight_cycle() {
        let mut world = WorldClient::new(0);
        world.setTotalWorldTime(100);
        world.setWorldTime(-6000);
        world.tick();
        assert_eq!(world.getTotalWorldTime(), 101);
        assert_eq!(world.getWorldTime(), 6000);
        assert!(!world.isDaylightCycleEnabled());
    }

    #[test]
    fn combined_light_preserves_sky_and_block_channels() {
        let mut world = WorldClient::new(0);
        let pos = BlockPos::new(0, 0, 0);
        let mut chunk = Chunk::new(0, 0);
        let data = BlockStateContainer::new();
        let mut block = vec![0_u8; 2048];
        let mut sky = vec![0_u8; 2048];
        block[0] = 0x07;
        sky[0] = 0x0C;
        chunk.setStorage(
            0,
            Some(ExtendedBlockStorage::fromNetwork(
                0,
                data,
                NibbleArray::fromStorage(block).unwrap(),
                Some(NibbleArray::fromStorage(sky).unwrap()),
            )),
        );
        world.putChunk(chunk);
        let packed = world.getCombinedLight(pos, 0);
        assert_eq!((packed >> 20) & 15, 12);
        assert_eq!((packed >> 4) & 15, 7);
    }
    #[test]
    fn end_portal_frame_uses_base_selection_and_eye_collision() {
        let mut world = WorldClient::new(0);
        let pos = BlockPos::new(0, 64, 0);
        let eye_state = IBlockState::fromGlobalStateId((120 << 4) | 4);
        world.invalidateRegionAndSetBlock(pos, eye_state).unwrap();
        let selected = world.getSelectedBoundingBox(pos).expect("portal frame selection");
        assert_eq!((selected.min_y, selected.max_y), (64.0, 64.8125));
        let collisions = world.getCollisionBoxes(AxisAlignedBB::new(0.4, 64.0, 0.4, 0.6, 65.1, 0.6));
        assert!(collisions.iter().any(|bounds| bounds.min_y == 64.8125 && bounds.max_y == 65.0));
    }

    #[test]
    fn skull_block_creates_default_tile_entity_and_has_outline_without_collision() {
        let mut world = WorldClient::new(0);
        let pos = BlockPos::new(1, 64, 1);
        let state = BlockSkull::BlockSkull::stateForFacing(EnumFacing::North);
        world.invalidateRegionAndSetBlock(pos, state).unwrap();
        assert!(world.getTileEntitySkull(pos).is_some());
        let selected = world.getSelectedBoundingBox(pos).expect("skull selection");
        assert_eq!((selected.min_z, selected.max_z), (1.5, 2.0));
        assert!(world.getCollisionBoxes(AxisAlignedBB::from_block(pos)).is_empty());
        world.invalidateRegionAndSetBlock(pos, IBlockState::fromGlobalStateId(0)).unwrap();
        assert!(world.getTileEntitySkull(pos).is_none());
    }

    #[test]
    fn skull_update_requires_a_loaded_skull_block_and_replaces_default_data() {
        let mut world = WorldClient::new(0);
        let pos = BlockPos::new(3, 70, -2);
        let mut tag = crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound::new();
        tag.setString("id", "minecraft:skull");
        tag.setInteger("x", pos.x); tag.setInteger("y", pos.y); tag.setInteger("z", pos.z);
        tag.setByte("SkullType", 4); tag.setByte("Rot", 9);
        assert!(!world.applySkullTileEntityTag(&tag));
        world.invalidateRegionAndSetBlock(pos, BlockSkull::BlockSkull::stateForFacing(EnumFacing::Up)).unwrap();
        assert!(world.applySkullTileEntityTag(&tag));
        let skull = world.getTileEntitySkull(pos).unwrap();
        assert_eq!(skull.getSkullType(), 4);
        assert_eq!(skull.getSkullRotation(), 9);
    }


    #[test]
    fn entity_mouse_over_targets_living_mobs_and_ignores_dropped_items() {
        use crate::net::minecraft::client::entity::EntityOtherClient::{ClientEntityKind, EntityOtherClient, MobEntityType, ObjectSpawnType};
        let mut world = WorldClient::new(0);
        let item = EntityOtherClient::new(
            2, None,
            ClientEntityKind::Object { objectType: ObjectSpawnType::Item, data: 0, spawnVelocity: [0.0; 3] },
            0.0, 64.0, 1.0, 0.0, 0.0,
        );
        world.addNonPlayerEntityToWorld(2, item);
        let cow = EntityOtherClient::new(
            3, None,
            ClientEntityKind::Mob { entityType: MobEntityType::fromId(92).unwrap() },
            0.0, 64.0, 2.0, 0.0, 0.0,
        );
        world.addNonPlayerEntityToWorld(3, cow);
        let hit = world.rayTraceEntities(
            1, None,
            AxisAlignedBB::new(-0.3, 64.0, -0.3, 0.3, 65.8, 0.3),
            Vec3d::new(0.0, 64.7, 0.0),
            Vec3d::new(0.0, 0.0, 1.0),
            4.5, 4.5, false,
        ).expect("cow should be targetable through non-collidable item");
        assert_eq!(hit.entityId, 3);
    }

    #[test]
    fn marker_armor_stand_is_not_mouse_over_target() {
        use crate::net::minecraft::client::entity::EntityOtherClient::{ClientEntityKind, EntityOtherClient, ObjectSpawnType};
        use crate::net::minecraft::network::datasync::DataSerializers::DataValue;
        let mut world = WorldClient::new(0);
        let mut stand = EntityOtherClient::new(
            4, None,
            ClientEntityKind::Object { objectType: ObjectSpawnType::ArmorStand, data: 0, spawnVelocity: [0.0; 3] },
            0.0, 64.0, 2.0, 0.0, 0.0,
        );
        stand.applyMetadata([(11, DataValue::Byte(0x10))]);
        world.addNonPlayerEntityToWorld(4, stand);
        assert!(world.rayTraceEntities(
            1, None,
            AxisAlignedBB::new(-0.3, 64.0, -0.3, 0.3, 65.8, 0.3),
            Vec3d::new(0.0, 64.7, 0.0), Vec3d::new(0.0, 0.0, 1.0),
            4.5, 4.5, false,
        ).is_none());
    }

    #[test]
    fn enchanting_table_block_owns_and_ticks_its_client_tile_entity() {
        let mut world = WorldClient::new(0);
        let pos = BlockPos::new(4, 64, -2);
        world
            .invalidateRegionAndSetBlock(pos, IBlockState::fromGlobalStateId(116 << 4))
            .unwrap();
        assert_eq!(world.enchantmentTableTileEntities().count(), 1);
        world.tickWithPlayerTarget(Some([4.5, 64.5, -0.5]));
        let table = world.enchantmentTableTileEntities().next().unwrap();
        assert_eq!(table.tickCount, 1);
        assert!((table.bookSpread - 0.1).abs() < 1.0e-6);

        world
            .invalidateRegionAndSetBlock(pos, IBlockState::fromGlobalStateId(0))
            .unwrap();
        assert_eq!(world.enchantmentTableTileEntities().count(), 0);
    }

    #[test]
    fn block_changes_mutate_the_packed_section() {
        let mut world=WorldClient::new(0);
        let pos=BlockPos::new(-1,64,17);
        world.invalidateRegionAndSetBlock(pos,IBlockState::fromGlobalStateId(16)).unwrap();
        assert_eq!(world.getBlockState(pos).getGlobalStateId(),16);
    }
}

impl IBlockAccess for WorldClient {
    fn getBlockState(&self, pos: BlockPos) -> IBlockState {
        WorldClient::getBlockState(self, pos)
    }
}

impl BiomeAccess for WorldClient {
    fn getBiomeId(&self, pos: BlockPos) -> u8 {
        let chunkX = pos.x.div_euclid(16);
        let chunkZ = pos.z.div_euclid(16);
        self.chunks.get(&(chunkX, chunkZ)).map_or(0, |chunk| {
            let localX = pos.x.rem_euclid(16) as usize;
            let localZ = pos.z.rem_euclid(16) as usize;
            chunk.getBiomeArray()[localZ * 16 + localX]
        })
    }

    fn getBlockStateForColor(&self, pos: BlockPos) -> IBlockState {
        WorldClient::getBlockState(self, pos)
    }
}
