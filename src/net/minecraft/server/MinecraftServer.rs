use std::path::{Path, PathBuf};

use crate::net::minecraft::network::NetworkSystem::NetworkSystem;
use crate::net::minecraft::util::CryptManager::KeyPair;
use crate::net::minecraft::world::chunk::storage::AnvilSaveConverter::AnvilSaveConverter;

/// Source-shaped base state from MCP 1.12.2 `MinecraftServer` needed by the
/// integrated-server bootstrap. World ticking/player management are added only
/// when their corresponding server classes are ported; no client-world stand-in
/// is used here.
#[derive(Debug)]
pub struct MinecraftServer {
    anvilFile: PathBuf,
    anvilConverterForAnvilFile: AnvilSaveConverter,
    networkSystem: NetworkSystem,
    serverPort: i32,
    serverRunning: bool,
    serverStopped: bool,
    tickCounter: i32,
    onlineMode: bool,
    canSpawnAnimals: bool,
    canSpawnNPCs: bool,
    pvpEnabled: bool,
    allowFlight: bool,
    motd: String,
    buildLimit: i32,
    serverKeyPair: Option<KeyPair>,
    serverOwner: Option<String>,
    folderName: String,
    worldName: String,
    isDemo: bool,
    enableBonusChest: bool,
    serverIsRunning: bool,
    userMessage: Option<String>,
}

impl MinecraftServer {
    pub fn new(anvilFileIn: impl AsRef<Path>) -> Self {
        let anvilFile = anvilFileIn.as_ref().to_path_buf();
        Self {
            anvilConverterForAnvilFile: AnvilSaveConverter::new(&anvilFile),
            networkSystem: NetworkSystem::new(),
            anvilFile,
            serverPort: -1,
            serverRunning: true,
            serverStopped: false,
            tickCounter: 0,
            onlineMode: false,
            canSpawnAnimals: false,
            canSpawnNPCs: false,
            pvpEnabled: false,
            allowFlight: false,
            motd: String::new(),
            buildLimit: 0,
            serverKeyPair: None,
            serverOwner: None,
            folderName: String::new(),
            worldName: String::new(),
            isDemo: false,
            enableBonusChest: false,
            serverIsRunning: false,
            userMessage: None,
        }
    }

    /// MCP `MinecraftServer#initiateShutdown` is complete at this tranche:
    /// it only clears the server-running flag. Full `tick`/`stopServer` are
    /// intentionally not exposed until WorldServer and PlayerList exist, so a
    /// partial implementation cannot masquerade as the source lifecycle.
    pub fn initiateShutdown(&mut self) { self.serverRunning = false; }

    pub fn getActiveAnvilConverter(&self) -> &AnvilSaveConverter { &self.anvilConverterForAnvilFile }
    pub fn getNetworkSystem(&self) -> &NetworkSystem { &self.networkSystem }
    pub fn getNetworkSystemMut(&mut self) -> &mut NetworkSystem { &mut self.networkSystem }
    pub fn getAnvilFile(&self) -> &Path { &self.anvilFile }
    pub const fn getServerPort(&self) -> i32 { self.serverPort }
    pub fn setServerPort(&mut self, port: i32) { self.serverPort = port; }
    pub const fn isServerRunning(&self) -> bool { self.serverRunning }
    pub const fn isServerStopped(&self) -> bool { self.serverStopped }
    pub const fn getTickCounter(&self) -> i32 { self.tickCounter }
    pub const fn isServerInOnlineMode(&self) -> bool { self.onlineMode }
    pub fn setOnlineMode(&mut self, online: bool) { self.onlineMode = online; }
    pub const fn getCanSpawnAnimals(&self) -> bool { self.canSpawnAnimals }
    pub fn setCanSpawnAnimals(&mut self, enabled: bool) { self.canSpawnAnimals = enabled; }
    pub const fn getCanSpawnNPCs(&self) -> bool { self.canSpawnNPCs }
    pub fn setCanSpawnNPCs(&mut self, enabled: bool) { self.canSpawnNPCs = enabled; }
    pub const fn isPVPEnabled(&self) -> bool { self.pvpEnabled }
    pub fn setAllowPvp(&mut self, enabled: bool) { self.pvpEnabled = enabled; }
    pub const fn isFlightAllowed(&self) -> bool { self.allowFlight }
    pub fn setAllowFlight(&mut self, enabled: bool) { self.allowFlight = enabled; }
    pub fn getMOTD(&self) -> &str { &self.motd }
    pub fn setMOTD(&mut self, motd: impl Into<String>) { self.motd = motd.into(); }
    pub const fn getBuildLimit(&self) -> i32 { self.buildLimit }
    pub fn setBuildLimit(&mut self, height: i32) { self.buildLimit = height; }
    pub fn getKeyPair(&self) -> Option<&KeyPair> { self.serverKeyPair.as_ref() }
    pub fn setKeyPair(&mut self, keyPair: KeyPair) { self.serverKeyPair = Some(keyPair); }
    pub fn getServerOwner(&self) -> Option<&str> { self.serverOwner.as_deref() }
    pub fn setServerOwner(&mut self, owner: impl Into<String>) { self.serverOwner = Some(owner.into()); }
    pub const fn isSinglePlayer(&self) -> bool { self.serverOwner.is_some() }
    pub fn getFolderName(&self) -> &str { &self.folderName }
    pub fn setFolderName(&mut self, name: impl Into<String>) { self.folderName = name.into(); }
    pub fn getWorldName(&self) -> &str { &self.worldName }
    pub fn setWorldName(&mut self, name: impl Into<String>) { self.worldName = name.into(); }
    pub const fn isDemo(&self) -> bool { self.isDemo }
    pub fn setDemo(&mut self, demo: bool) { self.isDemo = demo; }
    pub const fn getEnableBonusChest(&self) -> bool { self.enableBonusChest }
    pub fn canCreateBonusChest(&mut self, enable: bool) { self.enableBonusChest = enable; }
    pub const fn serverIsInRunLoop(&self) -> bool { self.serverIsRunning }
    pub fn setServerInRunLoop(&mut self, running: bool) { self.serverIsRunning = running; }
    pub fn incrementTickCounter(&mut self) { self.tickCounter = self.tickCounter.wrapping_add(1); }
    pub fn getUserMessage(&self) -> Option<&str> { self.userMessage.as_deref() }
    pub fn setUserMessage(&mut self, message: Option<impl Into<String>>) {
        self.userMessage = message.map(Into::into);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_matches_minecraft_server_bootstrap_defaults() {
        let root = std::env::temp_dir().join("mc1122-server-base-test");
        let server = MinecraftServer::new(&root);
        assert_eq!(server.getServerPort(), -1);
        assert!(server.isServerRunning());
        assert!(!server.isServerStopped());
        assert!(!server.serverIsInRunLoop());
        assert!(!server.isServerInOnlineMode());
        assert_eq!(server.getTickCounter(), 0);
    }

    #[test]
    fn initiate_shutdown_matches_source_flag_only() {
        let root = std::env::temp_dir().join("mc1122-server-shutdown-test");
        let mut server = MinecraftServer::new(&root);
        assert!(server.isServerRunning());
        server.initiateShutdown();
        assert!(!server.isServerRunning());
        assert!(!server.isServerStopped());
    }
}
