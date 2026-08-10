use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc,atomic::{AtomicBool,Ordering},mpsc};
use std::thread::{self,JoinHandle};
use std::time::{Duration,Instant};

use crate::compat::Java::string_hash_code;
use crate::net::minecraft::entity::player::EntityPlayerMP::EntityPlayerMP;
use crate::net::minecraft::network::NetworkManager::{LocalEndpointAddress,NetworkManagerError};
use crate::net::minecraft::server::MinecraftServer::MinecraftServer;
use crate::net::minecraft::server::management::PlayerList::PlayerList;
use crate::net::minecraft::server::network::NetHandlerHandshakeTCP::NetHandlerHandshakeTCP;
use crate::net::minecraft::server::network::NetHandlerLoginServer::{LoginUpdate,NetHandlerLoginServer};
use crate::net::minecraft::network::NetHandlerPlayServer::NetHandlerPlayServer;
use crate::net::minecraft::util::CryptManager::{generateKeyPair,CryptManagerError};
use crate::net::minecraft::world::GameType::GameType;
use crate::net::minecraft::world::WorldServer::WorldServer;
use crate::net::minecraft::world::WorldSettings::WorldSettings;
use crate::net::minecraft::world::WorldType::WorldType;
use crate::net::minecraft::world::storage::WorldInfo::WorldInfo;

#[derive(Debug)]
enum ServerConnection {
    Handshake,
    Login(NetHandlerLoginServer),
    Play { player:EntityPlayerMP, handler:NetHandlerPlayServer },
}

/// MCP 1.12.2 `IntegratedServer` at the first genuinely playable local-server
/// boundary. Only world types whose real IChunkGenerator is available may
/// start; no empty/fake generator is substituted.
#[derive(Debug)]
pub struct IntegratedServer {
    base:MinecraftServer,
    worldSettings:WorldSettings,
    isGamePaused:bool,
    isPublic:bool,
    worldServer:Option<WorldServer>,
    playerList:PlayerList,
    connections:HashMap<u64,ServerConnection>,
}

impl IntegratedServer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(savesDirectory:impl AsRef<Path>,serverOwner:impl Into<String>,folderNameIn:impl Into<String>,worldNameIn:impl Into<String>,worldSettingsIn:WorldSettings,isDemo:bool)->Self{
        let mut base=MinecraftServer::new(savesDirectory);base.setServerOwner(serverOwner);base.setFolderName(folderNameIn);base.setWorldName(worldNameIn);base.setDemo(isDemo);base.canCreateBonusChest(worldSettingsIn.isBonusChestEnabled());base.setBuildLimit(256);
        let worldSettings=if isDemo{WorldSettings::new(string_hash_code("North Carolina") as i64,GameType::Survival,true,false,WorldType::Default).enableBonusChest()}else{worldSettingsIn};
        Self{base,worldSettings,isGamePaused:false,isPublic:false,worldServer:None,playerList:PlayerList::newIntegrated(),connections:HashMap::new()}
    }

    pub fn prepareStartServer(&mut self)->Result<(),CryptManagerError>{self.base.setOnlineMode(true);self.base.setCanSpawnAnimals(true);self.base.setCanSpawnNPCs(true);self.base.setAllowPvp(true);self.base.setAllowFlight(true);self.base.setKeyPair(generateKeyPair()?);Ok(())}

    /// Source order of `IntegratedServer#startServer` and the overworld portion
    /// of `MinecraftServer#loadAllWorlds`. Nether/End WorldServerMulti remain a
    /// declared gap until their real generators are ported.
    pub fn startServer(&mut self)->Result<(),String>{
        self.prepareStartServer().map_err(|e|e.to_string())?;
        self.base.setUserMessage(Some("menu.loadingLevel"));
        let folder=self.base.getFolderName().to_owned();let worldName=self.base.getWorldName().to_owned();
        let handler=self.base.getActiveAnvilConverter().getSaveLoader(&folder,true).map_err(|e|e.to_string())?;
        let mut info=match handler.loadWorldInfo().map_err(|e|e.to_string())?{Some(mut info)=>{info.setWorldName(worldName.clone());info},None=>WorldInfo::new(&self.worldSettings,worldName.clone())};
        handler.saveWorldInfo(&mut info).map_err(|e|e.to_string())?;
        let mut world=WorldServer::new(handler,info,0).init()?;
        world.initialize(&self.worldSettings)?;
        self.base.setUserMessage(Some("menu.generatingTerrain"));
        self.initialWorldChunkLoad(&mut world)?;
        self.base.setMOTD(format!("{} - {}",self.base.getServerOwner().unwrap_or("Player"),world.worldInfo.getWorldName()));
        self.base.setServerInRunLoop(true);self.base.setUserMessage(None::<String>);self.worldServer=Some(world);Ok(())
    }

    /// MCP `MinecraftServer#initialWorldChunkLoad`: 25x25 = 625 chunks around
    /// spawn, in the same -192..192 / 16 grid.
    fn initialWorldChunkLoad(&self,world:&mut WorldServer)->Result<(),String>{let spawn=world.getSpawnPoint();for dx in (-192..=192).step_by(16){for dz in (-192..=192).step_by(16){world.provideChunkSnapshot((spawn.x+dx)>>4,(spawn.z+dz)>>4)?;}}Ok(())}

    pub fn addLocalEndpoint(&mut self)->Result<LocalEndpointAddress,String>{if !self.base.serverIsInRunLoop(){return Err("IntegratedServer is not in the run loop".to_owned());}Ok(self.base.getNetworkSystemMut().addLocalEndpoint())}

    fn tick(&mut self)->Result<(),String>{
        self.base.incrementTickCounter();
        if !self.isGamePaused {if let Some(world)=self.worldServer.as_mut(){world.setTotalWorldTime(world.getTotalWorldTime().wrapping_add(1));if world.worldInfo.getGameRulesInstance().getBoolean("doDaylightCycle"){world.setWorldTime(world.getWorldTime().wrapping_add(1));}}}
        let serverOwner=self.base.getServerOwner().map(str::to_owned);
        let Self{base,worldServer,playerList,connections,..}=self;
        let world=worldServer.as_mut().ok_or_else(||"IntegratedServer has no WorldServer".to_owned())?;
        let networkSystem=base.getNetworkSystemMut();networkSystem.pollLocalEndpoints();
        for network in networkSystem.networkManagersMut(){
            let id=network.id();let mut state=connections.remove(&id).unwrap_or(ServerConnection::Handshake);
            loop{
                let raw=match network.readPacket(){Ok(p)=>p,Err(NetworkManagerError::Timeout)=>break,Err(NetworkManagerError::Closed)=>{break;},Err(e)=>return Err(e.to_string())};
                state=match state{
                    ServerConnection::Handshake=>{if raw.id!=0{return Err(format!("unexpected handshake packet id {}",raw.id));}NetHandlerHandshakeTCP::processHandshake(network,&raw)?;ServerConnection::Login(NetHandlerLoginServer::new())}
                    ServerConnection::Login(mut login)=>{if raw.id!=0{return Err(format!("unexpected login packet id {}",raw.id));}login.processLoginStart(network,&raw)?;ServerConnection::Login(login)}
                    ServerConnection::Play{mut player,mut handler}=>{let moved=handler.processPacket(network,world,&mut player,&raw)?;if moved{playerList.updateMovingPlayer(network,&mut player)?;}ServerConnection::Play{player,handler}}
                };
            }
            state=match state{
                ServerConnection::Login(mut login)=>match login.update(network)?{
                    LoginUpdate::Waiting=>ServerConnection::Login(login),
                    LoginUpdate::Accepted(profile)=>{let(player,handler)=playerList.initializeConnectionToPlayer(network,world,profile,serverOwner.as_deref())?;ServerConnection::Play{player,handler}}
                },
                other=>other,
            };
            if let ServerConnection::Play{player,handler}= &mut state {handler.update();let _=playerList.tickPlayerChunks(world,network,player)?;}
            if network.isChannelOpen(){connections.insert(id,state);}
            else if let ServerConnection::Play{player,..}=&state {playerList.writePlayerData(world,player,serverOwner.as_deref())?;}
        }
        // MCP MinecraftServer#tick: every 900 ticks save players first so
        // IntegratedPlayerList hostPlayerData is current, then force-save worlds.
        if base.getTickCounter()%900==0{
            for state in connections.values(){if let ServerConnection::Play{player,..}=state{playerList.writePlayerData(world,player,serverOwner.as_deref())?;}}
            let _=world.saveAllChunks(true)?;
        }
        Ok(())
    }

    fn stopServer(&mut self){
        self.base.setServerInRunLoop(false);
        self.base.getNetworkSystemMut().terminateEndpoints();
        let owner=self.base.getServerOwner().map(str::to_owned);
        if let Some(world)=self.worldServer.as_mut(){
            for state in self.connections.values(){if let ServerConnection::Play{player,..}=state{let _=self.playerList.writePlayerData(world,player,owner.as_deref());}}
            let _=world.saveAllChunks(true); world.saveHandler().flush();
        }
    }

    pub fn minecraftServer(&self)->&MinecraftServer{&self.base} pub fn minecraftServerMut(&mut self)->&mut MinecraftServer{&mut self.base}
    pub fn getWorldSettings(&self)->&WorldSettings{&self.worldSettings} pub const fn isGamePaused(&self)->bool{self.isGamePaused} pub fn setGamePaused(&mut self,paused:bool){self.isGamePaused=paused;} pub const fn getPublic(&self)->bool{self.isPublic}
}

/// Owns the real integrated-server 20 TPS thread. Dropping the handle requests
/// source-shaped shutdown and joins the thread before world/save ownership dies.
pub struct IntegratedServerHandle { shutdown:Arc<AtomicBool>, thread:Option<JoinHandle<()>> }
impl std::fmt::Debug for IntegratedServerHandle{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.debug_struct("IntegratedServerHandle").finish_non_exhaustive()}}
impl IntegratedServerHandle {
    pub fn launch(mut server:IntegratedServer)->Result<(Self,LocalEndpointAddress),String>{
        let shutdown=Arc::new(AtomicBool::new(false));
        let flag=Arc::clone(&shutdown);
        let (startupSender,startupReceiver)=mpsc::sync_channel::<Result<LocalEndpointAddress,String>>(1);
        let thread=thread::Builder::new().name("Server thread".to_owned()).spawn(move||{
            // MCP `MinecraftServer#run`: `startServer`, world loading and the
            // run loop all belong to the dedicated Server thread.
            let endpoint=match server.startServer().and_then(|_|server.addLocalEndpoint()){
                Ok(endpoint)=>endpoint,
                Err(error)=>{let _=startupSender.send(Err(error));server.stopServer();return;}
            };
            if startupSender.send(Ok(endpoint)).is_err(){server.stopServer();return;}
            let tick=Duration::from_millis(50);
            while !flag.load(Ordering::Acquire)&&server.minecraftServer().isServerRunning(){
                let start=Instant::now();
                if let Err(error)=server.tick(){log::error!("IntegratedServer tick failed: {error}");break;}
                if let Some(rest)=tick.checked_sub(start.elapsed()){thread::sleep(rest);}
            }
            server.stopServer();
        }).map_err(|e|e.to_string())?;
        match startupReceiver.recv(){
            Ok(Ok(endpoint))=>Ok((Self{shutdown,thread:Some(thread)},endpoint)),
            Ok(Err(error))=>{let _=thread.join();Err(error)},
            Err(error)=>{let _=thread.join();Err(format!("IntegratedServer startup channel closed: {error}"))},
        }
    }
    pub fn shutdown(&self){self.shutdown.store(true,Ordering::Release);}
}
impl Drop for IntegratedServerHandle{fn drop(&mut self){self.shutdown();if let Some(thread)=self.thread.take(){let _=thread.join();}}}

#[cfg(test)]mod tests{use super::*;#[test]fn constructor_preserves_integrated_server_source_fields(){let settings=WorldSettings::new(123,GameType::Creative,true,false,WorldType::Flat).enableCommands().enableBonusChest();let server=IntegratedServer::new(std::env::temp_dir().join("mc1122-integrated-server-test"),"Player","World1","My World",settings.clone(),false);let base=server.minecraftServer();assert_eq!(base.getServerOwner(),Some("Player"));assert_eq!(base.getFolderName(),"World1");assert_eq!(base.getWorldName(),"My World");assert_eq!(base.getBuildLimit(),256);assert!(base.getEnableBonusChest());assert_eq!(server.getWorldSettings(),&settings);assert!(!base.serverIsInRunLoop());}}
