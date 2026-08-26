use std::io;
use std::path::{Path, PathBuf};

use crate::net::minecraft::client::multiplayer::ServerData::ServerData;
use crate::net::minecraft::nbt::CompressedStreamTools;
use crate::net::minecraft::nbt::NBTBase::{NBTBase, TAG_COMPOUND};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::nbt::NBTTagList::NBTTagList;

#[derive(Debug, Clone)]
pub struct ServerList {
    gameDir: PathBuf,
    servers: Vec<ServerData>,
}

impl ServerList {
    pub fn new(gameDir: impl Into<PathBuf>) -> Self {
        let mut value = Self {
            gameDir: gameDir.into(),
            servers: Vec::new(),
        };
        if let Err(error) = value.loadServerList() {
            log::error!("Couldn't load server list: {error}");
        }
        value
    }

    pub fn loadServerList(&mut self) -> io::Result<()> {
        self.servers.clear();
        let Some(root) = CompressedStreamTools::read(self.serverListPath())? else {
            return Ok(());
        };
        let list = root.getTagList("servers", TAG_COMPOUND);
        for index in 0..list.tagCount() {
            self.servers.push(ServerData::getServerDataFromNBTCompound(
                &list.getCompoundTagAt(index),
            ));
        }
        Ok(())
    }

    pub fn saveServerList(&self) -> io::Result<()> {
        let mut list = NBTTagList::new();
        for server in &self.servers {
            list.appendTag(NBTBase::Compound(server.getNBTCompound()));
        }
        let mut root = NBTTagCompound::new();
        root.setTagList("servers", list);
        CompressedStreamTools::safeWrite(&root, self.serverListPath())
    }

    pub fn getServerData(&self, index: usize) -> Option<&ServerData> {
        self.servers.get(index)
    }
    pub fn getServerDataMut(&mut self, index: usize) -> Option<&mut ServerData> {
        self.servers.get_mut(index)
    }
    pub fn removeServerData(&mut self, index: usize) -> Option<ServerData> {
        if index < self.servers.len() {
            Some(self.servers.remove(index))
        } else {
            None
        }
    }
    pub fn addServerData(&mut self, server: ServerData) {
        self.servers.push(server);
    }
    pub fn countServers(&self) -> usize {
        self.servers.len()
    }
    pub fn servers(&self) -> &[ServerData] {
        &self.servers
    }
    pub fn serversMut(&mut self) -> &mut [ServerData] {
        &mut self.servers
    }

    pub fn swapServers(&mut self, pos1: usize, pos2: usize) -> io::Result<()> {
        if pos1 >= self.servers.len() || pos2 >= self.servers.len() {
            return Ok(());
        }
        self.servers.swap(pos1, pos2);
        self.saveServerList()
    }

    pub fn set(&mut self, index: usize, server: ServerData) {
        if index < self.servers.len() {
            self.servers[index] = server;
        }
    }
    pub fn serverListPath(&self) -> PathBuf {
        self.gameDir.join("servers.dat")
    }

    pub fn saveSingleServer(gameDir: impl AsRef<Path>, server: &ServerData) -> io::Result<()> {
        let mut serverList = Self::new(gameDir.as_ref().to_path_buf());
        for current in &mut serverList.servers {
            if current.serverName == server.serverName && current.serverIP == server.serverIP {
                *current = server.clone();
                break;
            }
        }
        serverList.saveServerList()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn servers_dat_roundtrip_uses_mcp_root_and_list_shape() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("mc112-server-list-{unique}"));
        std::fs::create_dir_all(&directory).unwrap();
        let mut list = ServerList::new(&directory);
        list.addServerData(ServerData::new("Local", "127.0.0.1:25565", false));
        list.saveServerList().unwrap();
        let loaded = ServerList::new(&directory);
        assert_eq!(loaded.countServers(), 1);
        assert_eq!(loaded.getServerData(0).unwrap().serverName, "Local");
        let _ = std::fs::remove_dir_all(directory);
    }
}
