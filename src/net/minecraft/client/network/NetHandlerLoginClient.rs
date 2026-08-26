use crate::com::mojang::authlib::minecraft::MinecraftSessionService::{
    JoinServerError, MinecraftSessionService,
};
use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::net::minecraft::network::login::client::CPacketEncryptionResponse::CPacketEncryptionResponse;
use crate::net::minecraft::network::login::server::SPacketDisconnect::SPacketDisconnect;
use crate::net::minecraft::network::login::server::SPacketEnableCompression::SPacketEnableCompression;
use crate::net::minecraft::network::login::server::SPacketEncryptionRequest::SPacketEncryptionRequest;
use crate::net::minecraft::network::login::server::SPacketLoginSuccess::SPacketLoginSuccess;
use crate::net::minecraft::network::NetworkManager::{NetworkManager, NetworkManagerError};
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;
use crate::net::minecraft::util::CryptManager::{createNewSharedKey, getServerIdHashHex};
use crate::net::minecraft::util::Session::Session;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginHandlerEvent {
    Authorizing,
    CompressionEnabled(i32),
    LoginSuccess(GameProfile),
    Disconnected(ITextComponent),
}

#[derive(Debug, thiserror::Error)]
pub enum NetHandlerLoginClientError {
    #[error(transparent)]
    Network(#[from] NetworkManagerError),
    #[error("invalid encryption request: {0}")]
    EncryptionRequest(String),
    #[error("invalid login success packet: {0}")]
    LoginSuccess(String),
    #[error("invalid compression packet: {0}")]
    Compression(String),
    #[error("invalid disconnect packet: {0}")]
    Disconnect(String),
    #[error("login authentication failed: {0}")]
    Authentication(String),
}

#[derive(Debug, Clone)]
pub struct NetHandlerLoginClient {
    session: Session,
    isLanServer: bool,
    sessionService: MinecraftSessionService,
}

impl NetHandlerLoginClient {
    pub fn new(session: Session, isLanServer: bool) -> Self {
        Self {
            session,
            isLanServer,
            sessionService: MinecraftSessionService::new(),
        }
    }

    pub fn processPacket(
        &self,
        networkManager: &mut NetworkManager,
        packet: &RawPacket,
    ) -> Result<LoginHandlerEvent, NetHandlerLoginClientError> {
        match packet.id {
            0 => {
                let packet = SPacketDisconnect::readPacketData(packet)
                    .map_err(|error| NetHandlerLoginClientError::Disconnect(error.to_string()))?;
                Ok(LoginHandlerEvent::Disconnected(packet.getReason().clone()))
            }
            1 => self.handleEncryptionRequest(networkManager, packet),
            2 => {
                let packet = SPacketLoginSuccess::readPacketData(packet)
                    .map_err(|error| NetHandlerLoginClientError::LoginSuccess(error.to_string()))?;
                Ok(LoginHandlerEvent::LoginSuccess(packet.getProfile().clone()))
            }
            3 => {
                let packet = SPacketEnableCompression::readPacketData(packet)
                    .map_err(|error| NetHandlerLoginClientError::Compression(error.to_string()))?;
                networkManager.setCompressionThreshold(packet.getCompressionThreshold());
                Ok(LoginHandlerEvent::CompressionEnabled(
                    packet.getCompressionThreshold(),
                ))
            }
            id => Err(NetHandlerLoginClientError::LoginSuccess(format!(
                "unexpected login packet id {id}"
            ))),
        }
    }

    fn handleEncryptionRequest(
        &self,
        networkManager: &mut NetworkManager,
        rawPacket: &RawPacket,
    ) -> Result<LoginHandlerEvent, NetHandlerLoginClientError> {
        let packet = SPacketEncryptionRequest::readPacketData(rawPacket)
            .map_err(|error| NetHandlerLoginClientError::EncryptionRequest(error.to_string()))?;
        let secretKey = createNewSharedKey();
        let serverHash = getServerIdHashHex(
            packet.getServerId(),
            packet.getPublicKeyEncoded(),
            &secretKey,
        );
        let profile = self.session.getProfile();
        if let Err(error) =
            self.sessionService
                .joinServer(&profile, self.session.getToken(), &serverHash)
        {
            if self.isLanServer {
                // MCP catches the Authlib base AuthenticationException here,
                // not only AuthenticationUnavailableException.
                log::warn!("Couldn't connect to auth servers but will continue to join LAN");
            } else {
                return Err(NetHandlerLoginClientError::Authentication(
                    authentication_message(error),
                ));
            }
        }
        let response = CPacketEncryptionResponse::new(
            &secretKey,
            packet.getPublicKey(),
            packet.getVerifyToken(),
        )
        .map_err(|error| NetHandlerLoginClientError::EncryptionRequest(error.to_string()))?
        .writePacketData()
        .map_err(NetworkManagerError::Codec)?;
        networkManager.sendPacket(&response)?;
        networkManager.enableEncryption(&secretKey);
        Ok(LoginHandlerEvent::Authorizing)
    }
}

fn authentication_message(error: JoinServerError) -> String {
    match error {
        JoinServerError::AuthenticationUnavailable(_) => {
            "disconnect.loginFailedInfo.serversUnavailable".to_owned()
        }
        JoinServerError::InvalidCredentials(_) => {
            "disconnect.loginFailedInfo.invalidSession".to_owned()
        }
        JoinServerError::Authentication(message) => message,
    }
}
