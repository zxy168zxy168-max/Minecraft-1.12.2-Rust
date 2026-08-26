use rsa::RsaPublicKey;

use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_byte_array, read_string, CodecError};
use crate::net::minecraft::util::CryptManager::{decodePublicKey, CryptManagerError};

#[derive(Debug, Clone)]
pub struct SPacketEncryptionRequest {
    hashedServerId: String,
    publicKey: RsaPublicKey,
    publicKeyEncoded: Vec<u8>,
    verifyToken: Vec<u8>,
}

impl SPacketEncryptionRequest {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, SPacketEncryptionRequestError> {
        let mut input = packet.payload.as_slice();
        let hashedServerId = read_string(&mut input, 20)?;
        let publicKeyMaximum = input.len();
        let publicKeyEncoded = read_byte_array(&mut input, publicKeyMaximum)?;
        let publicKey = decodePublicKey(&publicKeyEncoded)?;
        let verifyTokenMaximum = input.len();
        let verifyToken = read_byte_array(&mut input, verifyTokenMaximum)?;
        Ok(Self {
            hashedServerId,
            publicKey,
            publicKeyEncoded,
            verifyToken,
        })
    }
    pub fn getServerId(&self) -> &str {
        &self.hashedServerId
    }
    pub fn getPublicKey(&self) -> &RsaPublicKey {
        &self.publicKey
    }
    pub fn getPublicKeyEncoded(&self) -> &[u8] {
        &self.publicKeyEncoded
    }
    pub fn getVerifyToken(&self) -> &[u8] {
        &self.verifyToken
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SPacketEncryptionRequestError {
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error(transparent)]
    Crypt(#[from] CryptManagerError),
}
