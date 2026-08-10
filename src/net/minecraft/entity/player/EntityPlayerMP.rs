use std::sync::atomic::{AtomicI32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::net::minecraft::entity::Entity::Entity;
use crate::net::minecraft::entity::player::PlayerCapabilities::PlayerCapabilities;
use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::util::EnumHand::EnumHand;
use crate::net::minecraft::world::GameType::GameType;
use crate::net::minecraft::server::management::PlayerInteractionManager::PlayerInteractionManager;
use crate::net::minecraft::world::WorldServer::WorldServer;
use crate::compat::Java::JavaRandom;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::nbt::NBTBase::NBTBase;
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::nbt::NBTTagList::NBTTagList;

static NEXT_ENTITY_ID: AtomicI32 = AtomicI32::new(1);

/// Server-side player state owned by MCP 1.12.2 `EntityPlayerMP`.
///
/// This tranche contains only fields required by integrated-login, spawn,
/// movement acknowledgement and the initial PlayerChunkMap stream.  It is not
/// a client-player stand-in and intentionally does not claim inventory/stat/
/// advancement coverage which still belongs to later `EntityPlayerMP` work.
#[derive(Debug, Clone)]
pub struct EntityPlayerMP {
    pub entity: Entity,
    entityId: i32,
    profile: GameProfile,
    pub capabilities: PlayerCapabilities,
    pub interactionManager: PlayerInteractionManager,
    pub dimension: i32,
    pub inventory: InventoryPlayer,
    pub managedPosX: f64,
    pub managedPosZ: f64,
    /// Fixed player NBT retained as an opaque base so fields whose runtime
    /// owners are not yet ported are not silently discarded on save.
    persistedNbt: Option<NBTTagCompound>,
}

impl EntityPlayerMP {
    /// MCP `EntityPlayerMP` construction over the currently owned WorldServer
    /// services. `EntityPlayer` first places the player at spawn+1; the MP
    /// constructor then applies spawn-radius/top-solid placement and stepHeight.
    /// The default WorldBorder is effectively unbounded for the supported fresh
    /// flat world; custom border clipping and collision-box lift remain pending
    /// with the full server WorldBorder/collision runtime.
    pub fn new(world: &mut WorldServer, profile: GameProfile, gameType: GameType) -> Result<Self,String> {
        let spawn = world.getSpawnPoint();
        let mut entity = Entity::default();
        // EntityPlayer(World,GameProfile) source constructor.
        entity.setPositionAndRotation(
            spawn.x as f64 + 0.5,
            spawn.y as f64 + 1.0,
            spawn.z as f64 + 0.5,
            0.0,
            0.0,
        );

        // EntityPlayerMP source constructor. Local `Entity#rand` is seeded in
        // Java from process-time entropy; exact coordinates are intentionally
        // nondeterministic there too, while the RNG algorithm remains Java's.
        let mut blockpos=spawn;
        if world.provider.hasSkyLight() && gameType != GameType::Adventure {
            let radius=world.worldInfo.getGameRulesInstance().getInt("spawnRadius").max(0);
            let entropy=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as i64;
            let mut random=JavaRandom::new(entropy);
            let bound=radius.wrapping_mul(2).wrapping_add(1);
            let dx=random.next_i32_bound(bound).wrapping_sub(radius);
            let dz=random.next_i32_bound(bound).wrapping_sub(radius);
            blockpos=world.getTopSolidOrLiquidBlock(BlockPos::new(spawn.x.wrapping_add(dx),spawn.y,spawn.z.wrapping_add(dz)))?;
        }
        entity.stepHeight=1.0;
        entity.setPositionAndRotation(blockpos.x as f64+0.5,blockpos.y as f64,blockpos.z as f64+0.5,0.0,0.0);

        let mut capabilities = PlayerCapabilities::default();
        gameType.configurePlayerCapabilities(&mut capabilities);
        let managedPosX = entity.posX;
        let managedPosZ = entity.posZ;
        Ok(Self {
            entity,
            entityId: NEXT_ENTITY_ID.fetch_add(1, Ordering::Relaxed),
            profile,
            capabilities,
            interactionManager: PlayerInteractionManager::new(gameType),
            dimension: world.provider.getDimension(),
            inventory: InventoryPlayer::default(),
            managedPosX,
            managedPosZ,
            persistedNbt: None,
        })
    }

    /// Current source-backed subset of `Entity#readFromNBT` +
    /// `EntityPlayer#readEntityFromNBT`. The fixed compound is retained so
    /// Food/EnderItems/shoulder/root-vehicle and other not-yet-owned fields
    /// survive a read-modify-write cycle.
    pub fn readFromNBT(&mut self, compound: NBTTagCompound) {
        let pos=compound.getTagList("Pos", 6); let motion=compound.getTagList("Motion", 6); let rotation=compound.getTagList("Rotation", 5);
        if pos.tagCount() >= 3 {
            self.entity.posX=pos.getDoubleAt(0); self.entity.posY=pos.getDoubleAt(1); self.entity.posZ=pos.getDoubleAt(2);
            self.entity.prevPosX=self.entity.posX; self.entity.prevPosY=self.entity.posY; self.entity.prevPosZ=self.entity.posZ;
        }
        if motion.tagCount() >= 3 {
            self.entity.motionX=motion.getDoubleAt(0); self.entity.motionY=motion.getDoubleAt(1); self.entity.motionZ=motion.getDoubleAt(2);
            if self.entity.motionX.abs()>10.0{self.entity.motionX=0.0;} if self.entity.motionY.abs()>10.0{self.entity.motionY=0.0;} if self.entity.motionZ.abs()>10.0{self.entity.motionZ=0.0;}
        }
        if rotation.tagCount() >= 2 { self.entity.rotationYaw=rotation.getFloatAt(0); self.entity.rotationPitch=rotation.getFloatAt(1); self.entity.prevRotationYaw=self.entity.rotationYaw; self.entity.prevRotationPitch=self.entity.rotationPitch; }
        self.entity.fallDistance=compound.getFloat("FallDistance"); self.entity.fire=compound.getShort("Fire") as i32; self.entity.onGround=compound.getBoolean("OnGround");
        if compound.hasKey("Dimension") { self.dimension=compound.getInteger("Dimension"); }
        self.entity.setPosition(self.entity.posX,self.entity.posY,self.entity.posZ);
        self.inventory.readFromNBT(&compound.getTagList("Inventory", 10));
        self.inventory.currentItem=compound.getInteger("SelectedItemSlot");
        self.capabilities.readCapabilitiesFromNBT(&compound);
        self.managedPosX=self.entity.posX; self.managedPosZ=self.entity.posZ;
        self.persistedNbt=Some(compound);
    }

    /// Current source-backed subset of `Entity#writeToNBT` +
    /// `EntityPlayer#writeEntityToNBT`, updating an existing fixed compound
    /// rather than manufacturing a lossy replacement.
    pub fn writeToNBT(&self) -> NBTTagCompound {
        let mut compound=self.persistedNbt.clone().unwrap_or_else(NBTTagCompound::new);
        let mut pos=NBTTagList::new(); pos.appendTag(NBTBase::Double(self.entity.posX)); pos.appendTag(NBTBase::Double(self.entity.posY)); pos.appendTag(NBTBase::Double(self.entity.posZ)); compound.setTagList("Pos",pos);
        let mut motion=NBTTagList::new(); motion.appendTag(NBTBase::Double(self.entity.motionX)); motion.appendTag(NBTBase::Double(self.entity.motionY)); motion.appendTag(NBTBase::Double(self.entity.motionZ)); compound.setTagList("Motion",motion);
        let mut rotation=NBTTagList::new(); rotation.appendTag(NBTBase::Float(self.entity.rotationYaw)); rotation.appendTag(NBTBase::Float(self.entity.rotationPitch)); compound.setTagList("Rotation",rotation);
        compound.setFloat("FallDistance",self.entity.fallDistance); compound.setShort("Fire",self.entity.fire as i16); if !compound.hasKey("Air"){compound.setShort("Air",300);} compound.setBoolean("OnGround",self.entity.onGround);
        compound.setInteger("Dimension",self.dimension); if !compound.hasKey("Invulnerable"){compound.setBoolean("Invulnerable",false);} if !compound.hasKey("PortalCooldown"){compound.setInteger("PortalCooldown",0);}
        if let Some(uuid)=self.profile.getId(){compound.setUniqueId("UUID",uuid);}
        compound.setInteger("DataVersion",1343);
        compound.setTagList("Inventory",self.inventory.writeToNBT(NBTTagList::new())); compound.setInteger("SelectedItemSlot",self.inventory.currentItem);
        if !compound.hasKey("Sleeping"){compound.setBoolean("Sleeping",false);} if !compound.hasKey("SleepTimer"){compound.setShort("SleepTimer",0);}
        if !compound.hasKey("XpP"){compound.setFloat("XpP",0.0);} if !compound.hasKey("XpLevel"){compound.setInteger("XpLevel",0);} if !compound.hasKey("XpTotal"){compound.setInteger("XpTotal",0);} if !compound.hasKey("XpSeed"){compound.setInteger("XpSeed",0);} if !compound.hasKey("Score"){compound.setInteger("Score",0);}
        self.capabilities.writeCapabilitiesToNBT(&mut compound);
        compound
    }

    pub const fn getEntityId(&self) -> i32 { self.entityId }
    pub fn getGameProfile(&self) -> &GameProfile { &self.profile }
    pub fn getName(&self) -> &str { self.profile.getName() }
    pub const fn getGameType(&self) -> GameType { self.interactionManager.getGameType() }
    pub fn setGameType(&mut self, gameType: GameType) {
        self.interactionManager.setGameType(gameType);
        gameType.configurePlayerCapabilities(&mut self.capabilities);
    }
    pub fn setPlayerLocation(&mut self, x:f64,y:f64,z:f64,yaw:f32,pitch:f32) {
        self.entity.setPositionAndRotation(x,y,z,yaw,pitch);
    }
    pub fn getHeldItem(&self, hand: EnumHand) -> &ItemStack {
        match hand {
            EnumHand::MainHand => self.inventory.getCurrentItem(),
            EnumHand::OffHand => &self.inventory.offHandInventory[0],
        }
    }
    pub fn getHeldItemMut(&mut self, hand: EnumHand) -> &mut ItemStack {
        match hand {
            EnumHand::MainHand => {
                let slot = self.inventory.currentItem.clamp(0, 8) as usize;
                &mut self.inventory.mainInventory[slot]
            }
            EnumHand::OffHand => &mut self.inventory.offHandInventory[0],
        }
    }
    pub fn setHeldItem(&mut self, hand: EnumHand, stack: ItemStack) {
        match hand {
            EnumHand::MainHand => {
                let slot = self.inventory.currentItem.clamp(0, 8) as usize;
                self.inventory.mainInventory[slot] = stack;
            }
            EnumHand::OffHand => self.inventory.offHandInventory[0] = stack,
        }
    }
}
