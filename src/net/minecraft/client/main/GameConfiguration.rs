use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::net::minecraft::util::Session::Session;

pub type PropertyMap = BTreeMap<String, Vec<String>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Proxy {
    NoProxy,
    Socks {
        host: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct GameConfiguration {
    pub userInfo: UserInformation,
    pub displayInfo: DisplayInformation,
    pub folderInfo: FolderInformation,
    pub gameInfo: GameInformation,
    pub serverInfo: ServerInformation,
}

impl GameConfiguration {
    pub fn new(
        userInfoIn: UserInformation,
        displayInfoIn: DisplayInformation,
        folderInfoIn: FolderInformation,
        gameInfoIn: GameInformation,
        serverInfoIn: ServerInformation,
    ) -> Self {
        Self {
            userInfo: userInfoIn,
            displayInfo: displayInfoIn,
            folderInfo: folderInfoIn,
            gameInfo: gameInfoIn,
            serverInfo: serverInfoIn,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayInformation {
    pub width: i32,
    pub height: i32,
    pub fullscreen: bool,
    pub checkGlErrors: bool,
}

impl DisplayInformation {
    pub const fn new(
        widthIn: i32,
        heightIn: i32,
        fullscreenIn: bool,
        checkGlErrorsIn: bool,
    ) -> Self {
        Self {
            width: widthIn,
            height: heightIn,
            fullscreen: fullscreenIn,
            checkGlErrors: checkGlErrorsIn,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderInformation {
    pub mcDataDir: PathBuf,
    pub resourcePacksDir: PathBuf,
    pub assetsDir: PathBuf,
    pub assetIndex: Option<String>,
}

impl FolderInformation {
    pub fn new(
        mcDataDirIn: impl Into<PathBuf>,
        resourcePacksDirIn: impl Into<PathBuf>,
        assetsDirIn: impl Into<PathBuf>,
        assetIndexIn: Option<String>,
    ) -> Self {
        Self {
            mcDataDir: mcDataDirIn.into(),
            resourcePacksDir: resourcePacksDirIn.into(),
            assetsDir: assetsDirIn.into(),
            assetIndex: assetIndexIn,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameInformation {
    pub isDemo: bool,
    pub version: String,
    pub versionType: String,
}

impl GameInformation {
    pub fn new(demo: bool, versionIn: impl Into<String>, versionTypeIn: impl Into<String>) -> Self {
        Self {
            isDemo: demo,
            version: versionIn.into(),
            versionType: versionTypeIn.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerInformation {
    pub serverName: Option<String>,
    pub serverPort: u16,
}

impl ServerInformation {
    pub fn new(serverNameIn: Option<String>, serverPortIn: u16) -> Self {
        Self {
            serverName: serverNameIn,
            serverPort: serverPortIn,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserInformation {
    pub session: Session,
    pub userProperties: PropertyMap,
    pub profileProperties: PropertyMap,
    pub proxy: Proxy,
}

impl UserInformation {
    pub fn new(
        sessionIn: Session,
        userPropertiesIn: PropertyMap,
        profilePropertiesIn: PropertyMap,
        proxyIn: Proxy,
    ) -> Self {
        Self {
            session: sessionIn,
            userProperties: userPropertiesIn,
            profileProperties: profilePropertiesIn,
            proxy: proxyIn,
        }
    }
}
