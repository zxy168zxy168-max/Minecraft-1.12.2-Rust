use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::net::minecraft::entity::player::EntityPlayerMP::EntityPlayerMP;
use crate::net::minecraft::network::NetworkManager::NetworkManager;
use crate::net::minecraft::network::PacketBuffer::write_string;
use crate::net::minecraft::network::play::server::SPacketCustomPayload::SPacketCustomPayload;
use crate::net::minecraft::network::play::server::SPacketHeldItemChange::SPacketHeldItemChange;
use crate::net::minecraft::network::play::server::SPacketJoinGame::SPacketJoinGame;
use crate::net::minecraft::network::play::server::SPacketPlayerAbilities::SPacketPlayerAbilities;
use crate::net::minecraft::network::play::server::SPacketServerDifficulty::SPacketServerDifficulty;
use crate::net::minecraft::network::play::server::SPacketSpawnPosition::SPacketSpawnPosition;
use crate::net::minecraft::network::play::server::SPacketTimeUpdate::SPacketTimeUpdate;
use crate::net::minecraft::network::play::server::SPacketUnloadChunk::SPacketUnloadChunk;
use crate::net::minecraft::server::management::PlayerChunkMap::PlayerChunkMap;
use crate::net::minecraft::network::NetHandlerPlayServer::NetHandlerPlayServer;
use crate::net::minecraft::world::WorldServer::WorldServer;
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;

/// Integrated-server subset of MCP `PlayerList` / `IntegratedPlayerList`.
#[derive(Debug)]
pub struct PlayerList {
    maxPlayers:u8,
    viewDistance:i32,
    playerChunkMap:PlayerChunkMap,
    /// MCP `IntegratedPlayerList#hostPlayerData`.
    hostPlayerData:Option<NBTTagCompound>,
}
impl Default for PlayerList { fn default()->Self{Self::newIntegrated()} }
impl PlayerList {
    /// `IntegratedPlayerList` keeps the base maxPlayers=8; vanilla server view
    /// distance defaults to 10 unless settings override it.
    pub fn newIntegrated()->Self{Self{maxPlayers:8,viewDistance:10,playerChunkMap:PlayerChunkMap::new(10),hostPlayerData:None}}
    pub const fn getMaxPlayers(&self)->u8{self.maxPlayers}
    pub const fn getViewDistance(&self)->i32{self.viewDistance}
    pub fn createPlayerForUser(&self,world:&mut WorldServer,profile:GameProfile)->Result<EntityPlayerMP,String>{
        let gameType=world.worldInfo.getGameType();
        EntityPlayerMP::new(world,profile,gameType)
    }
    pub fn initializeConnectionToPlayer(&mut self,network:&mut NetworkManager,world:&mut WorldServer,profile:GameProfile,serverOwner:Option<&str>)->Result<(EntityPlayerMP,NetHandlerPlayServer),String>{
        let mut player=self.createPlayerForUser(world,profile)?;
        // MCP PlayerList#readPlayerDataFromFile: integrated host data embedded
        // in level.dat takes precedence, otherwise use playerdata/<uuid>.dat.
        let host=serverOwner.is_some_and(|owner|player.getName()==owner);
        if host {
            if let Some(tag)=world.worldInfo.getPlayerNBTTagCompound().cloned(){player.readFromNBT(tag);}
            else {world.saveHandler().base().readPlayerData(&mut player).map_err(|e|e.to_string())?;}
        } else {world.saveHandler().base().readPlayerData(&mut player).map_err(|e|e.to_string())?;}
        if player.dimension!=world.provider.getDimension(){return Err(format!("saved player dimension {} is not loaded by this IntegratedServer tranche",player.dimension));}
        player.setGameType(world.worldInfo.getGameType());
        let info=&world.worldInfo;
        network.sendPacket(&SPacketJoinGame::new(
            player.getEntityId(),player.getGameType(),info.isHardcoreModeEnabled(),world.provider.getDimension(),
            info.getDifficulty(),self.maxPlayers,info.getTerrainType(),info.getGameRulesInstance().getBoolean("reducedDebugInfo")
        ).writePacketData().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;

        let mut brand=Vec::new();write_string("vanilla",32767,&mut brand).map_err(|e|e.to_string())?;
        network.sendPacket(&SPacketCustomPayload::new("MC|Brand",brand).writePacketData().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        network.sendPacket(&SPacketServerDifficulty::new(info.getDifficulty(),info.isDifficultyLocked()).writePacketData()).map_err(|e|e.to_string())?;
        network.sendPacket(&SPacketPlayerAbilities::new(&player.capabilities).writePacketData()).map_err(|e|e.to_string())?;
        network.sendPacket(&SPacketHeldItemChange::new(player.inventory.currentItem).writePacketData()).map_err(|e|e.to_string())?;

        self.playerChunkMap.addPlayer(&mut player);
        let mut play=NetHandlerPlayServer::new();
        let (x,y,z,yaw,pitch)=(player.entity.posX,player.entity.posY,player.entity.posZ,player.entity.rotationYaw,player.entity.rotationPitch);
        play.setPlayerLocation(network,&mut player,x,y,z,yaw,pitch)?;
        network.sendPacket(&SPacketTimeUpdate::new(info.getWorldTotalTime(),info.getWorldTime(),info.getGameRulesInstance().getBoolean("doDaylightCycle")).writePacketData()).map_err(|e|e.to_string())?;
        network.sendPacket(&SPacketSpawnPosition::new(world.getSpawnPoint()).writePacketData()).map_err(|e|e.to_string())?;
        Ok((player,play))
    }
    /// MCP PlayerList / IntegratedPlayerList player-data write ownership.
    pub fn writePlayerData(&mut self,world:&mut WorldServer,player:&EntityPlayerMP,serverOwner:Option<&str>)->Result<(),String>{
        let tag=player.writeToNBT();
        world.saveHandler().base().writePlayerData(player).map_err(|e|e.to_string())?;
        if serverOwner.is_some_and(|owner|player.getName()==owner){self.hostPlayerData=Some(tag.clone());world.worldInfo.setPlayerNBTTagCompound(Some(tag));}
        Ok(())
    }
    pub fn getHostPlayerData(&self)->Option<&NBTTagCompound>{self.hostPlayerData.as_ref()}

    pub fn tickPlayerChunks(&mut self,world:&mut WorldServer,network:&mut NetworkManager,player:&EntityPlayerMP)->Result<usize,String>{self.playerChunkMap.tickPlayer(world,network,player)}
    pub fn updateMovingPlayer(&mut self,network:&mut NetworkManager,player:&mut EntityPlayerMP)->Result<(),String>{
        for (x,z) in self.playerChunkMap.updateMovingPlayer(player){network.sendPacket(&SPacketUnloadChunk::new(x,z).writePacketData()).map_err(|e|e.to_string())?;}
        Ok(())
    }
}
