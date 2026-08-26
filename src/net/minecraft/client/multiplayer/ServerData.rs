use crate::net::minecraft::nbt::NBTBase::{TAG_BYTE, TAG_STRING};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerResourceMode {
    Enabled,
    Disabled,
    Prompt,
}

impl ServerResourceMode {
    pub const fn next(self) -> Self {
        match self {
            Self::Enabled => Self::Disabled,
            Self::Disabled => Self::Prompt,
            Self::Prompt => Self::Enabled,
        }
    }
    pub const fn translationKey(self) -> &'static str {
        match self {
            Self::Enabled => "addServer.resourcePack.enabled",
            Self::Disabled => "addServer.resourcePack.disabled",
            Self::Prompt => "addServer.resourcePack.prompt",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerData {
    pub serverName: String,
    pub serverIP: String,
    pub populationInfo: String,
    pub serverMOTD: String,
    pub pingToServer: i64,
    pub version: i32,
    pub gameVersion: String,
    pub pinged: bool,
    pub playerList: Option<String>,
    resourceMode: ServerResourceMode,
    serverIcon: Option<String>,
    lanServer: bool,
}

impl ServerData {
    pub fn new(name: impl Into<String>, ip: impl Into<String>, isLan: bool) -> Self {
        Self {
            serverName: name.into(),
            serverIP: ip.into(),
            populationInfo: String::new(),
            serverMOTD: String::new(),
            pingToServer: -2,
            version: 340,
            gameVersion: "1.12.2".to_owned(),
            pinged: false,
            playerList: None,
            resourceMode: ServerResourceMode::Prompt,
            serverIcon: None,
            lanServer: isLan,
        }
    }

    pub fn getNBTCompound(&self) -> NBTTagCompound {
        let mut result = NBTTagCompound::new();
        result.setString("name", self.serverName.clone());
        result.setString("ip", self.serverIP.clone());
        if let Some(icon) = &self.serverIcon {
            result.setString("icon", icon.clone());
        }
        match self.resourceMode {
            ServerResourceMode::Enabled => result.setBoolean("acceptTextures", true),
            ServerResourceMode::Disabled => result.setBoolean("acceptTextures", false),
            ServerResourceMode::Prompt => {}
        }
        result
    }

    pub fn getResourceMode(&self) -> ServerResourceMode {
        self.resourceMode
    }
    pub fn setResourceMode(&mut self, mode: ServerResourceMode) {
        self.resourceMode = mode;
    }

    pub fn getServerDataFromNBTCompound(nbtCompound: &NBTTagCompound) -> Self {
        let mut result = Self::new(
            nbtCompound.getString("name"),
            nbtCompound.getString("ip"),
            false,
        );
        if nbtCompound.hasKeyWithType("icon", TAG_STRING) {
            result.serverIcon = Some(nbtCompound.getString("icon"));
        }
        result.resourceMode = if nbtCompound.hasKeyWithType("acceptTextures", TAG_BYTE) {
            if nbtCompound.getBoolean("acceptTextures") {
                ServerResourceMode::Enabled
            } else {
                ServerResourceMode::Disabled
            }
        } else {
            ServerResourceMode::Prompt
        };
        result
    }

    pub fn getBase64EncodedIconData(&self) -> Option<&str> {
        self.serverIcon.as_deref()
    }
    pub fn setBase64EncodedIconData(&mut self, icon: Option<String>) {
        self.serverIcon = icon;
    }
    pub const fn isOnLAN(&self) -> bool {
        self.lanServer
    }

    pub fn copyFrom(&mut self, serverDataIn: &ServerData) {
        self.serverIP.clone_from(&serverDataIn.serverIP);
        self.serverName.clone_from(&serverDataIn.serverName);
        self.resourceMode = serverDataIn.resourceMode;
        self.serverIcon.clone_from(&serverDataIn.serverIcon);
        self.lanServer = serverDataIn.lanServer;
    }

    pub fn resetPingState(&mut self, pingingText: &str) {
        self.serverMOTD = pingingText.to_owned();
        self.populationInfo.clear();
        self.pingToServer = -2;
        self.pinged = false;
        self.playerList = None;
        self.version = 340;
        self.gameVersion = "1.12.2".to_owned();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nbt_omits_prompt_and_preserves_explicit_resource_mode() {
        let prompt = ServerData::new("A", "example.org", false).getNBTCompound();
        assert!(!prompt.hasKey("acceptTextures"));
        let mut enabled = ServerData::new("A", "example.org", false);
        enabled.setResourceMode(ServerResourceMode::Enabled);
        assert!(enabled.getNBTCompound().getBoolean("acceptTextures"));
        assert_eq!(
            ServerData::getServerDataFromNBTCompound(&enabled.getNBTCompound()).getResourceMode(),
            ServerResourceMode::Enabled
        );
    }
}
