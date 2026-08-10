use crate::net::minecraft::network::EnumConnectionState::ConnectionState;
use crate::net::minecraft::network::NetworkManager::NetworkManager;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::handshake::client::C00Handshake::C00Handshake;

/// MCP 1.12.2 `NetHandlerHandshakeTCP` login-intention boundary.
#[derive(Debug, Default)]
pub struct NetHandlerHandshakeTCP;
impl NetHandlerHandshakeTCP {
    pub fn processHandshake(network:&mut NetworkManager, raw:&RawPacket)->Result<(),String>{
        let packet=C00Handshake::readPacketData(raw).map_err(|e|e.to_string())?;
        match packet.getRequestedState(){
            ConnectionState::Login => {
                network.setConnectionState(ConnectionState::Login);
                if packet.getProtocolVersion()!=340 {
                    return Err(format!("Outdated protocol {}, expected Minecraft 1.12.2 protocol 340",packet.getProtocolVersion()));
                }
                Ok(())
            }
            other=>Err(format!("unsupported integrated-server handshake intention {other:?}")),
        }
    }
}
