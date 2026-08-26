use rsa::RsaPublicKey;

use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{write_byte_array, CodecError};
use crate::net::minecraft::util::CryptManager::{encryptData, CryptManagerError, SecretKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CPacketEncryptionResponse {
    secretKeyEncrypted: Vec<u8>,
    verifyTokenEncrypted: Vec<u8>,
}

impl CPacketEncryptionResponse {
    pub fn new(
        secret: &SecretKey,
        key: &RsaPublicKey,
        verifyToken: &[u8],
    ) -> Result<Self, CryptManagerError> {
        Ok(Self {
            secretKeyEncrypted: encryptData(key, secret.getEncoded())?,
            verifyTokenEncrypted: encryptData(key, verifyToken)?,
        })
    }

    pub fn writePacketData(&self) -> Result<RawPacket, CodecError> {
        let mut payload = Vec::new();
        write_byte_array(&self.secretKeyEncrypted, &mut payload)?;
        write_byte_array(&self.verifyTokenEncrypted, &mut payload)?;
        Ok(RawPacket::new(1, payload))
    }
}
