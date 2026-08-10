use aes::Aes128;
use cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit};
use num_bigint::BigInt;
use rand::{rngs::OsRng, RngCore};
use rsa::{pkcs8::DecodePublicKey, Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use sha1::{Digest, Sha1};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptManagerError {
    #[error("invalid RSA public key: {0}")]
    InvalidPublicKey(String),
    #[error("RSA encryption failed: {0}")]
    Rsa(String),
    #[error("AES shared key must contain exactly 16 bytes")]
    InvalidSharedKey,
    #[error("RSA key-pair generation failed: {0}")]
    KeyPairGeneration(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretKey([u8; 16]);

impl SecretKey {
    pub const fn fromBytes(bytes: [u8; 16]) -> Self { Self(bytes) }
    pub const fn getEncoded(&self) -> &[u8; 16] { &self.0 }
}


#[derive(Debug, Clone)]
pub struct KeyPair {
    privateKey: RsaPrivateKey,
    publicKey: RsaPublicKey,
}

impl KeyPair {
    pub fn getPrivate(&self) -> &RsaPrivateKey { &self.privateKey }
    pub fn getPublic(&self) -> &RsaPublicKey { &self.publicKey }
}

/// MCP `CryptManager#generateKeyPair`: RSA, 1024-bit.
pub fn generateKeyPair() -> Result<KeyPair, CryptManagerError> {
    let privateKey = RsaPrivateKey::new(&mut OsRng, 1024)
        .map_err(|error| CryptManagerError::KeyPairGeneration(error.to_string()))?;
    let publicKey = RsaPublicKey::from(&privateKey);
    Ok(KeyPair { privateKey, publicKey })
}

pub fn createNewSharedKey() -> SecretKey {
    let mut key = [0_u8; 16];
    OsRng.fill_bytes(&mut key);
    SecretKey::fromBytes(key)
}

pub fn decodePublicKey(encodedKey: &[u8]) -> Result<RsaPublicKey, CryptManagerError> {
    RsaPublicKey::from_public_key_der(encodedKey)
        .map_err(|error| CryptManagerError::InvalidPublicKey(error.to_string()))
}

pub fn encryptData(key: &RsaPublicKey, data: &[u8]) -> Result<Vec<u8>, CryptManagerError> {
    key.encrypt(&mut OsRng, Pkcs1v15Encrypt, data)
        .map_err(|error| CryptManagerError::Rsa(error.to_string()))
}

pub fn getServerIdHash(serverId: &str, publicKey: &[u8], secretKey: &SecretKey) -> [u8; 20] {
    let mut digest = Sha1::new();
    // Java uses ISO-8859-1. Protocol server IDs are ASCII in 1.12.2; mapping
    // each Unicode scalar below 256 preserves the Java byte sequence.
    digest.update(serverId.chars().map(|character| character as u32 as u8).collect::<Vec<_>>());
    digest.update(secretKey.getEncoded());
    digest.update(publicKey);
    digest.finalize().into()
}

pub fn getServerIdHashHex(serverId: &str, publicKey: &[u8], secretKey: &SecretKey) -> String {
    BigInt::from_signed_bytes_be(&getServerIdHash(serverId, publicKey, secretKey)).to_str_radix(16)
}

/// Stateful AES/CFB8/NoPadding stream matching Netty's 1.12.2 cipher setup.
#[derive(Clone)]
pub struct NetCipher {
    cipher: Aes128,
    shiftRegister: [u8; 16],
    encrypting: bool,
}

impl std::fmt::Debug for NetCipher {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("NetCipher").field("encrypting", &self.encrypting).finish_non_exhaustive()
    }
}

impl NetCipher {
    pub fn new(secretKey: &SecretKey, encrypting: bool) -> Self {
        let cipher = Aes128::new(GenericArray::from_slice(secretKey.getEncoded()));
        Self { cipher, shiftRegister: *secretKey.getEncoded(), encrypting }
    }

    pub fn apply(&mut self, data: &mut [u8]) {
        for byte in data {
            let input = *byte;
            let mut block = GenericArray::clone_from_slice(&self.shiftRegister);
            self.cipher.encrypt_block(&mut block);
            let output = input ^ block[0];
            *byte = output;
            self.shiftRegister.copy_within(1.., 0);
            self.shiftRegister[15] = if self.encrypting { output } else { input };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String { bytes.iter().map(|value| format!("{value:02x}")).collect() }

    #[test]
    fn generated_keypair_is_1024_bit_rsa() {
        use rsa::traits::PublicKeyParts;
        let pair = generateKeyPair().unwrap();
        assert_eq!(pair.getPublic().n().bits(), 1024);
        assert_eq!(&pair.getPrivate().to_public_key(), pair.getPublic());
    }

    #[test]
    fn cfb8_encrypt_decrypt_roundtrip_is_stateful() {
        let key = SecretKey::fromBytes(*b"0123456789abcdef");
        let mut encrypted = b"Minecraft protocol encryption".to_vec();
        NetCipher::new(&key, true).apply(&mut encrypted);
        assert_eq!(hex(&encrypted), "3fda07bf434480a92711e2066f941fa508fa5ba78907758aa93087b4a4");
        NetCipher::new(&key, false).apply(&mut encrypted);
        assert_eq!(encrypted, b"Minecraft protocol encryption");
    }

    #[test]
    fn signed_server_hash_matches_java_big_integer_shape() {
        let key = SecretKey::fromBytes([0; 16]);
        let hash = getServerIdHashHex("", &[0], &key);
        assert_eq!(hash, "-12db1ed7df0d06ff51c7c4833b0d4ce3bfd24e42");
    }
}
