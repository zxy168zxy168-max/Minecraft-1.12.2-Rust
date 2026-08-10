use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, Sender},
    Arc,
};
use std::thread;
use std::time::Duration;

use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::multiplayer::ServerAddress::ServerAddress;
use crate::net::minecraft::client::multiplayer::ServerData::ServerData;
use crate::net::minecraft::client::network::NetHandlerLoginClient::{
    LoginHandlerEvent, NetHandlerLoginClient,
};
use crate::net::minecraft::client::network::NetHandlerPlayClient::{
    ClientSettingsSnapshot, NetHandlerPlayClient, PlayHandlerEvent, SharedPlayClientState,
};
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::entity::player::EntityPlayer::EnumChatVisibility;
use crate::net::minecraft::network::EnumConnectionState::ConnectionState;
use crate::net::minecraft::network::NetworkManager::{LocalEndpointAddress, NetworkManager, NetworkManagerError};
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::handshake::client::C00Handshake::C00Handshake;
use crate::net::minecraft::network::login::client::CPacketLoginStart::CPacketLoginStart;
use crate::net::minecraft::network::play::server::SPacketJoinGame::SPacketJoinGame;
use crate::net::minecraft::util::EnumHandSide::EnumHandSide;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::util::SoundCategory::SoundCategory;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::Session::Session;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;
use crate::vulkan::GuiDrawList::GuiDrawList;

#[derive(Debug, Clone, PartialEq)]
pub enum GuiConnectingEvent {
    Authorizing,
    CompressionEnabled(i32),
    LoginSuccess(GameProfile),
    JoinGame(SPacketJoinGame),
    Respawn { dimension: i32, dimensionChanged: bool },
    TerrainReady,
    PlayerDied(ITextComponent),
    Sound {
        sound: ResourceLocation,
        category: SoundCategory,
        x: f64,
        y: f64,
        z: f64,
        volume: f32,
        pitch: f32,
    },
    WorldEffect {
        effectType: i32,
        position: BlockPos,
        data: i32,
        serverWide: bool,
    },
    Disconnected(String),
    Failed { reasonKey: &'static str, message: String },
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiConnectingAction {
    Cancel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiConnectingInteraction {
    pub action: GuiConnectingAction,
    pub sound: GuiSoundCommand,
}

pub struct GuiConnecting {
    pub GuiScreen: GuiScreen,
    cancel: Arc<AtomicBool>,
    receiver: Receiver<GuiConnectingEvent>,
    /// Events already removed from the network channel but not yet consumed by
    /// the current GUI state. Vanilla handles packets sequentially on the main
    /// thread; retaining the tail across Connecting -> DownloadTerrain avoids
    /// dropping an immediately following first PlayerPosLook/TerrainReady.
    pendingEvents: VecDeque<GuiConnectingEvent>,
    sharedPlayState: SharedPlayClientState,
    playPacketSender: Sender<RawPacket>,
    authorizing: bool,
    terminal: bool,
    connectingText: String,
    authorizingText: String,
}

impl std::fmt::Debug for GuiConnecting {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GuiConnecting")
            .field("authorizing", &self.authorizing)
            .field("terminal", &self.terminal)
            .finish_non_exhaustive()
    }
}

impl GuiConnecting {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        serverDataIn: ServerData,
        session: Session,
        language: String,
        renderDistanceChunks: i32,
        chatVisibility: EnumChatVisibility,
        chatColours: bool,
        modelPartFlags: u8,
        mainHand: EnumHandSide,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();
        let (playPacketSender, playPacketReceiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let sharedPlayState = SharedPlayClientState::new();
        let settings = ClientSettingsSnapshot {
            language,
            renderDistanceChunks,
            chatVisibility,
            chatColours,
            modelPartFlags,
            mainHand,
        };
        spawn_connector(
            serverDataIn,
            session,
            settings,
            Arc::clone(&cancel),
            sender,
            sharedPlayState.clone(),
            playPacketReceiver,
        );
        Self {
            GuiScreen: GuiScreen::default(),
            cancel,
            receiver,
            pendingEvents: VecDeque::new(),
            sharedPlayState,
            playPacketSender,
            authorizing: false,
            terminal: false,
            connectingText: String::new(),
            authorizingText: String::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn newLocal(
        address: LocalEndpointAddress,
        session: Session,
        language: String,
        renderDistanceChunks: i32,
        chatVisibility: EnumChatVisibility,
        chatColours: bool,
        modelPartFlags: u8,
        mainHand: EnumHandSide,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();
        let (playPacketSender, playPacketReceiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let sharedPlayState = SharedPlayClientState::new();
        let settings = ClientSettingsSnapshot { language, renderDistanceChunks, chatVisibility, chatColours, modelPartFlags, mainHand };
        spawn_local_connector(address, session, settings, Arc::clone(&cancel), sender, sharedPlayState.clone(), playPacketReceiver);
        Self { GuiScreen: GuiScreen::default(), cancel, receiver, pendingEvents: VecDeque::new(), sharedPlayState, playPacketSender, authorizing:false, terminal:false, connectingText:String::new(), authorizingText:String::new() }
    }

    pub fn initGui(&mut self, width: i32, height: i32, locale: &Locale) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.GuiScreen.buttonList.clear();
        self.GuiScreen.buttonList.push(GuiButton::new(
            0,
            width / 2 - 100,
            height / 4 + 120 + 12,
            locale.translate_key("gui.cancel"),
        ));
        self.connectingText = locale.translate_key("connect.connecting").to_owned();
        self.authorizingText = locale.translate_key("connect.authorizing").to_owned();
    }

    pub fn updateScreen(&mut self) -> Vec<GuiConnectingEvent> {
        while let Ok(event) = self.receiver.try_recv() {
            self.pendingEvents.push_back(event);
        }

        let mut events = Vec::new();
        while let Some(event) = self.pendingEvents.pop_front() {
            let changesScreen = match &event {
                GuiConnectingEvent::JoinGame(_)
                | GuiConnectingEvent::TerrainReady
                | GuiConnectingEvent::PlayerDied(_)
                | GuiConnectingEvent::Disconnected(_)
                | GuiConnectingEvent::Failed { .. }
                | GuiConnectingEvent::Cancelled => true,
                GuiConnectingEvent::Respawn { dimensionChanged, .. } => *dimensionChanged,
                _ => false,
            };
            match &event {
                GuiConnectingEvent::Authorizing
                | GuiConnectingEvent::CompressionEnabled(_)
                | GuiConnectingEvent::LoginSuccess(_)
                | GuiConnectingEvent::TerrainReady => self.authorizing = true,
                GuiConnectingEvent::Respawn { .. }
                | GuiConnectingEvent::PlayerDied(_)
                | GuiConnectingEvent::Sound { .. }
                | GuiConnectingEvent::WorldEffect { .. } => {},
                GuiConnectingEvent::JoinGame(_)
                | GuiConnectingEvent::Disconnected(_)
                | GuiConnectingEvent::Failed { .. }
                | GuiConnectingEvent::Cancelled => self.terminal = true,
            }
            events.push(event);
            if changesScreen {
                // The same GuiConnecting object is moved into the next runtime
                // state, so any already-received tail remains available there.
                break;
            }
        }
        events
    }

    pub fn drawScreen(
        &mut self,
        drawList: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.GuiScreen.drawDefaultBackground(drawList);
        let text = if self.authorizing {
            &self.authorizingText
        } else {
            &self.connectingText
        };
        self.GuiScreen.Gui.drawCenteredString(
            font,
            drawList,
            text,
            self.GuiScreen.width / 2,
            self.GuiScreen.height / 2 - 50,
            0x00FF_FFFF,
        );
        self.GuiScreen
            .drawScreen(drawList, font, mouseX, mouseY, partialTicks);
    }

    pub fn mouseClicked(
        &self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
    ) -> Option<GuiConnectingInteraction> {
        if mouseButton != 0 {
            return None;
        }
        let button = self
            .GuiScreen
            .buttonList
            .iter()
            .find(|button| button.mousePressed(mouseX, mouseY))?;
        if button.id != 0 {
            return None;
        }
        Some(GuiConnectingInteraction {
            action: GuiConnectingAction::Cancel,
            sound: button.playPressSound(),
        })
    }

    pub fn cancel(&mut self) {
        self.cancel.store(true, Ordering::Release);
        self.terminal = true;
    }

    pub const fn isTerminal(&self) -> bool { self.terminal }
    pub fn getSharedPlayState(&self) -> SharedPlayClientState { self.sharedPlayState.clone() }

    pub fn sendPlayPackets(&self, packets: Vec<RawPacket>) -> Result<(), String> {
        for packet in packets {
            self.playPacketSender
                .send(packet)
                .map_err(|_| "play connection is no longer available".to_owned())?;
        }
        Ok(())
    }
}

impl Drop for GuiConnecting {
    fn drop(&mut self) { self.cancel.store(true, Ordering::Release); }
}

fn spawn_connector(
    serverData: ServerData,
    session: Session,
    settings: ClientSettingsSnapshot,
    cancel: Arc<AtomicBool>,
    sender: Sender<GuiConnectingEvent>,
    sharedPlayState: SharedPlayClientState,
    playPacketReceiver: Receiver<RawPacket>,
) {
    thread::Builder::new()
        .name(next_connector_name())
        .spawn(move || {
            if cancel.load(Ordering::Acquire) {
                let _ = sender.send(GuiConnectingEvent::Cancelled);
                return;
            }

            let address = ServerAddress::fromString(&serverData.serverIP);
            let host = address.getIP();
            if host.is_empty() {
                let _ = sender.send(GuiConnectingEvent::Failed {
                    reasonKey: "connect.failed",
                    message: "Unknown host".to_owned(),
                });
                return;
            }

            log::info!("Connecting to {}, {}", host, address.getPort());
            let mut network = match NetworkManager::createNetworkManagerAndConnect(
                &host,
                address.getPort(),
            ) {
                Ok(network) => network,
                Err(NetworkManagerError::UnknownHost) => {
                    let _ = sender.send(GuiConnectingEvent::Failed {
                        reasonKey: "connect.failed",
                        message: "Unknown host".to_owned(),
                    });
                    return;
                }
                Err(error) => {
                    let _ = sender.send(GuiConnectingEvent::Failed {
                        reasonKey: "connect.failed",
                        message: error.to_string(),
                    });
                    return;
                }
            };

            if cancel.load(Ordering::Acquire) {
                network.closeChannel();
                let _ = sender.send(GuiConnectingEvent::Cancelled);
                return;
            }

            let _ = sender.send(GuiConnectingEvent::Authorizing);
            let handshake = match C00Handshake::new(
                &host,
                address.getPort(),
                ConnectionState::Login,
            )
            .writePacketData()
            {
                Ok(packet) => packet,
                Err(error) => {
                    let _ = sender.send(GuiConnectingEvent::Failed {
                        reasonKey: "connect.failed",
                        message: error.to_string(),
                    });
                    return;
                }
            };
            if let Err(error) = network.sendPacket(&handshake) {
                let _ = sender.send(GuiConnectingEvent::Failed {
                    reasonKey: "connect.failed",
                    message: error.to_string(),
                });
                return;
            }
            network.setConnectionState(ConnectionState::Login);

            let loginStart = match CPacketLoginStart::new(session.getProfile()).writePacketData() {
                Ok(packet) => packet,
                Err(error) => {
                    let _ = sender.send(GuiConnectingEvent::Failed {
                        reasonKey: "connect.failed",
                        message: error.to_string(),
                    });
                    return;
                }
            };
            if let Err(error) = network.sendPacket(&loginStart) {
                let _ = sender.send(GuiConnectingEvent::Failed {
                    reasonKey: "connect.failed",
                    message: error.to_string(),
                });
                return;
            }

            let loginHandler = NetHandlerLoginClient::new(session, serverData.isOnLAN());
            let mut playHandler: Option<NetHandlerPlayClient> = None;

            loop {
                if cancel.load(Ordering::Acquire) {
                    network.closeChannel();
                    let _ = sender.send(GuiConnectingEvent::Cancelled);
                    return;
                }

                if playHandler.is_some() {
                    while let Ok(outbound) = playPacketReceiver.try_recv() {
                        if let Err(error) = network.sendPacket(&outbound) {
                            let _ = sender.send(GuiConnectingEvent::Failed {
                                reasonKey: "connect.failed",
                                message: error.to_string(),
                            });
                            return;
                        }
                    }
                }

                let packet = match network.readPacket() {
                    Ok(packet) => packet,
                    Err(NetworkManagerError::Timeout) => continue,
                    Err(NetworkManagerError::Closed) => {
                        let _ = sender.send(GuiConnectingEvent::Disconnected(
                            "Connection closed".to_owned(),
                        ));
                        return;
                    }
                    Err(error) => {
                        let _ = sender.send(GuiConnectingEvent::Failed {
                            reasonKey: "connect.failed",
                            message: error.to_string(),
                        });
                        return;
                    }
                };

                if let Some(handler) = playHandler.as_mut() {
                    match handler.processPacket(&mut network, &packet) {
                        Ok(PlayHandlerEvent::None | PlayHandlerEvent::IgnoredPacket(_)
                            | PlayHandlerEvent::ChunkLoaded { .. }
                            | PlayHandlerEvent::ChunkUnloaded { .. }
                            | PlayHandlerEvent::BlockChanged
                            | PlayHandlerEvent::TimeUpdated { .. }
                            | PlayHandlerEvent::SignEditorOpened { .. }
                            | PlayHandlerEvent::MapUpdated { .. }) => {}
                        Ok(PlayHandlerEvent::TileEntityUpdated { .. }) => {}
                        Ok(PlayHandlerEvent::JoinGame(joinGame)) => {
                            let _ = sender.send(GuiConnectingEvent::JoinGame(joinGame));
                        }
                        Ok(PlayHandlerEvent::Respawn { dimension, dimensionChanged }) => {
                            let _ = sender.send(GuiConnectingEvent::Respawn { dimension, dimensionChanged });
                        }
                        Ok(PlayHandlerEvent::TerrainReady) => {
                            let _ = sender.send(GuiConnectingEvent::TerrainReady);
                        }
                        Ok(PlayHandlerEvent::PlayerDied { message }) => {
                            let _ = sender.send(GuiConnectingEvent::PlayerDied(message));
                        }
                        Ok(PlayHandlerEvent::Sound { sound, category, x, y, z, volume, pitch }) => {
                            let _ = sender.send(GuiConnectingEvent::Sound {
                                sound, category, x, y, z, volume, pitch,
                            });
                        }
                        Ok(PlayHandlerEvent::WorldEffect { effectType, position, data, serverWide }) => {
                            let _ = sender.send(GuiConnectingEvent::WorldEffect {
                                effectType, position, data, serverWide,
                            });
                        }
                        Ok(PlayHandlerEvent::Disconnected(reason)) => {
                            let _ = sender.send(GuiConnectingEvent::Disconnected(
                                reason.getFormattedText().to_owned(),
                            ));
                            return;
                        }
                        Err(error) => {
                            let _ = sender.send(GuiConnectingEvent::Failed {
                                reasonKey: "connect.failed",
                                message: error.to_string(),
                            });
                            return;
                        }
                    }
                    continue;
                }

                match loginHandler.processPacket(&mut network, &packet) {
                    Ok(LoginHandlerEvent::Authorizing) => {
                        let _ = sender.send(GuiConnectingEvent::Authorizing);
                    }
                    Ok(LoginHandlerEvent::CompressionEnabled(threshold)) => {
                        let _ = sender.send(GuiConnectingEvent::CompressionEnabled(threshold));
                    }
                    Ok(LoginHandlerEvent::LoginSuccess(profile)) => {
                        network.setConnectionState(ConnectionState::Play);
                        let encrypted = network.isEncrypted();
                        sharedPlayState.withWrite(|state| state.networkEncrypted = encrypted);
                        if let Err(error) = network.setReadTimeout(Duration::from_millis(25)) {
                            let _ = sender.send(GuiConnectingEvent::Failed {
                                reasonKey: "connect.failed",
                                message: error.to_string(),
                            });
                            return;
                        }
                        playHandler = Some(NetHandlerPlayClient::new(
                            profile.clone(),
                            settings.clone(),
                            sharedPlayState.clone(),
                        ));
                        let _ = sender.send(GuiConnectingEvent::LoginSuccess(profile));
                    }
                    Ok(LoginHandlerEvent::Disconnected(reason)) => {
                        let _ = sender.send(GuiConnectingEvent::Disconnected(
                            reason.getFormattedText().to_owned(),
                        ));
                        return;
                    }
                    Err(error) => {
                        let _ = sender.send(GuiConnectingEvent::Failed {
                            reasonKey: "connect.failed",
                            message: error.to_string(),
                        });
                        return;
                    }
                }
            }
        })
        .expect("failed spawning Server Connector thread");
}

fn next_connector_name() -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static CONNECTION_ID: AtomicU32 = AtomicU32::new(0);
    format!(
        "Server Connector #{}",
        CONNECTION_ID.fetch_add(1, Ordering::Relaxed) + 1
    )
}

fn spawn_local_connector(
    address: LocalEndpointAddress,
    session: Session,
    settings: ClientSettingsSnapshot,
    cancel: Arc<AtomicBool>,
    sender: Sender<GuiConnectingEvent>,
    sharedPlayState: SharedPlayClientState,
    playPacketReceiver: Receiver<RawPacket>,
) {
    thread::Builder::new().name(next_connector_name()).spawn(move || {
        let mut network = match NetworkManager::provideLocalClient(&address) {
            Ok(network) => network,
            Err(error) => { let _=sender.send(GuiConnectingEvent::Failed{reasonKey:"connect.failed",message:error.to_string()}); return; }
        };
        if cancel.load(Ordering::Acquire) { network.closeChannel(); let _=sender.send(GuiConnectingEvent::Cancelled); return; }
        let _=sender.send(GuiConnectingEvent::Authorizing);
        let handshake=match C00Handshake::new(address.to_string(),0,ConnectionState::Login).writePacketData(){Ok(p)=>p,Err(e)=>{let _=sender.send(GuiConnectingEvent::Failed{reasonKey:"connect.failed",message:e.to_string()});return;}};
        if let Err(e)=network.sendPacket(&handshake){let _=sender.send(GuiConnectingEvent::Failed{reasonKey:"connect.failed",message:e.to_string()});return;}
        network.setConnectionState(ConnectionState::Login);
        let login=match CPacketLoginStart::new(session.getProfile()).writePacketData(){Ok(p)=>p,Err(e)=>{let _=sender.send(GuiConnectingEvent::Failed{reasonKey:"connect.failed",message:e.to_string()});return;}};
        if let Err(e)=network.sendPacket(&login){let _=sender.send(GuiConnectingEvent::Failed{reasonKey:"connect.failed",message:e.to_string()});return;}
        let loginHandler=NetHandlerLoginClient::new(session,false);
        let mut playHandler:Option<NetHandlerPlayClient>=None;
        loop {
            if cancel.load(Ordering::Acquire){network.closeChannel();let _=sender.send(GuiConnectingEvent::Cancelled);return;}
            if playHandler.is_some(){while let Ok(outbound)=playPacketReceiver.try_recv(){if let Err(e)=network.sendPacket(&outbound){let _=sender.send(GuiConnectingEvent::Failed{reasonKey:"connect.failed",message:e.to_string()});return;}}}
            let packet=match network.readPacket(){Ok(p)=>p,Err(NetworkManagerError::Timeout)=>continue,Err(NetworkManagerError::Closed)=>{let _=sender.send(GuiConnectingEvent::Disconnected("Connection closed".to_owned()));return;},Err(e)=>{let _=sender.send(GuiConnectingEvent::Failed{reasonKey:"connect.failed",message:e.to_string()});return;}};
            if let Some(handler)=playHandler.as_mut(){
                match handler.processPacket(&mut network,&packet){
                    Ok(PlayHandlerEvent::None|PlayHandlerEvent::IgnoredPacket(_)|PlayHandlerEvent::ChunkLoaded{..}|PlayHandlerEvent::ChunkUnloaded{..}|PlayHandlerEvent::BlockChanged|PlayHandlerEvent::TimeUpdated{..}|PlayHandlerEvent::SignEditorOpened{..}|PlayHandlerEvent::MapUpdated{..}|PlayHandlerEvent::TileEntityUpdated{..})=>{},
                    Ok(PlayHandlerEvent::JoinGame(v))=>{let _=sender.send(GuiConnectingEvent::JoinGame(v));},
                    Ok(PlayHandlerEvent::Respawn{dimension,dimensionChanged})=>{let _=sender.send(GuiConnectingEvent::Respawn{dimension,dimensionChanged});},
                    Ok(PlayHandlerEvent::TerrainReady)=>{let _=sender.send(GuiConnectingEvent::TerrainReady);},
                    Ok(PlayHandlerEvent::PlayerDied{message})=>{let _=sender.send(GuiConnectingEvent::PlayerDied(message));},
                    Ok(PlayHandlerEvent::Sound{sound,category,x,y,z,volume,pitch})=>{let _=sender.send(GuiConnectingEvent::Sound{sound,category,x,y,z,volume,pitch});},
                    Ok(PlayHandlerEvent::WorldEffect{effectType,position,data,serverWide})=>{let _=sender.send(GuiConnectingEvent::WorldEffect{effectType,position,data,serverWide});},
                    Ok(PlayHandlerEvent::Disconnected(reason))=>{let _=sender.send(GuiConnectingEvent::Disconnected(reason.getFormattedText().to_owned()));return;},
                    Err(e)=>{let _=sender.send(GuiConnectingEvent::Failed{reasonKey:"connect.failed",message:e.to_string()});return;}
                }
                continue;
            }
            match loginHandler.processPacket(&mut network,&packet){
                Ok(LoginHandlerEvent::Authorizing)=>{let _=sender.send(GuiConnectingEvent::Authorizing);},
                Ok(LoginHandlerEvent::CompressionEnabled(t))=>{let _=sender.send(GuiConnectingEvent::CompressionEnabled(t));},
                Ok(LoginHandlerEvent::LoginSuccess(profile))=>{network.setConnectionState(ConnectionState::Play);sharedPlayState.withWrite(|state|state.networkEncrypted=network.isEncrypted());let _=network.setReadTimeout(Duration::from_millis(25));playHandler=Some(NetHandlerPlayClient::new(profile.clone(),settings.clone(),sharedPlayState.clone()));let _=sender.send(GuiConnectingEvent::LoginSuccess(profile));},
                Ok(LoginHandlerEvent::Disconnected(reason))=>{let _=sender.send(GuiConnectingEvent::Disconnected(reason.getFormattedText().to_owned()));return;},
                Err(e)=>{let _=sender.send(GuiConnectingEvent::Failed{reasonKey:"connect.failed",message:e.to_string()});return;}
            }
        }
    }).expect("failed spawning Local Server Connector thread");
}
