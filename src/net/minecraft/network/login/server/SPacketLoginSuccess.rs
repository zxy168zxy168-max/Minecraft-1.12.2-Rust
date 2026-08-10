use uuid::Uuid;

use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_string, write_string, CodecError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketLoginSuccess { profile: GameProfile }
impl SPacketLoginSuccess {
    pub fn new(profile: GameProfile) -> Self { Self { profile } }
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, SPacketLoginSuccessError> {
        let mut input = packet.payload.as_slice();
        let uuidText = read_string(&mut input, 36)?;
        let name = read_string(&mut input, 16)?;
        if !input.is_empty() {
            return Err(SPacketLoginSuccessError::Codec(CodecError::InvalidData(format!("{} unread login-success bytes", input.len()))));
        }
        let id = Uuid::parse_str(&uuidText).map_err(|error| SPacketLoginSuccessError::InvalidUuid(error.to_string()))?;
        Ok(Self { profile: GameProfile::new(Some(id), name) })
    }
    pub fn writePacketData(&self) -> Result<RawPacket, SPacketLoginSuccessError> {
        let id = self.profile.getId().ok_or(SPacketLoginSuccessError::MissingUuid)?;
        let mut payload = Vec::new();
        write_string(&id.to_string(), 36, &mut payload)?;
        write_string(self.profile.getName(), 16, &mut payload)?;
        Ok(RawPacket::new(2, payload))
    }
    pub fn getProfile(&self) -> &GameProfile { &self.profile }
}

#[derive(Debug, thiserror::Error)]
pub enum SPacketLoginSuccessError {
    #[error(transparent)] Codec(#[from] CodecError),
    #[error("invalid login UUID: {0}")] InvalidUuid(String),
    #[error("login success profile has no UUID")] MissingUuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_success_roundtrip_matches_protocol_340_fields() {
        let id = Uuid::parse_str("01234567-89ab-cdef-0123-456789abcdef").unwrap();
        let packet = SPacketLoginSuccess::new(GameProfile::new(Some(id), "Player"))
            .writePacketData().unwrap();
        assert_eq!(packet.id, 2);
        let decoded = SPacketLoginSuccess::readPacketData(&packet).unwrap();
        assert_eq!(decoded.getProfile().getId(), Some(id));
        assert_eq!(decoded.getProfile().getName(), "Player");
    }
}
