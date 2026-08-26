use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::read_string;
use crate::net::minecraft::network::ServerStatusResponse::ServerStatusResponse;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketServerInfo {
    response: ServerStatusResponse,
}
impl SPacketServerInfo {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, anyhow::Error> {
        let mut input = packet.payload.as_slice();
        let json = read_string(&mut input, 32767).map_err(anyhow::Error::new)?;
        let response = ServerStatusResponse::fromJson(&json)?;
        Ok(Self { response })
    }
    pub fn getResponse(&self) -> &ServerStatusResponse {
        &self.response
    }
}
