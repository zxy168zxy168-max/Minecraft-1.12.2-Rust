use uuid::Uuid;

use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::net::minecraft::network::login::client::CPacketLoginStart::CPacketLoginStart;
use crate::net::minecraft::network::login::server::SPacketLoginSuccess::SPacketLoginSuccess;
use crate::net::minecraft::network::EnumConnectionState::ConnectionState;
use crate::net::minecraft::network::NetworkManager::NetworkManager;
use crate::net::minecraft::network::Packet::RawPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginState {
    Hello,
    Key,
    Authenticating,
    ReadyToAccept,
    DelayAccept,
    Accepted,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginUpdate {
    Waiting,
    Accepted(GameProfile),
}

/// Local-channel part of MCP 1.12.2 `NetHandlerLoginServer`.
/// Online TCP authentication deliberately remains in the existing remote-login
/// path; integrated LocalChannel skips encryption exactly as MCP does.
#[derive(Debug)]
pub struct NetHandlerLoginServer {
    currentLoginState: LoginState,
    loginGameProfile: Option<GameProfile>,
    connectionTimer: i32,
}
impl Default for NetHandlerLoginServer {
    fn default() -> Self {
        Self::new()
    }
}
impl NetHandlerLoginServer {
    pub fn new() -> Self {
        Self {
            currentLoginState: LoginState::Hello,
            loginGameProfile: None,
            connectionTimer: 0,
        }
    }
    pub const fn state(&self) -> LoginState {
        self.currentLoginState
    }
    pub fn processLoginStart(
        &mut self,
        network: &NetworkManager,
        raw: &RawPacket,
    ) -> Result<(), String> {
        if self.currentLoginState != LoginState::Hello {
            return Err("Unexpected hello packet".to_owned());
        }
        let packet = CPacketLoginStart::readPacketData(raw).map_err(|e| e.to_string())?;
        self.loginGameProfile = Some(packet.getProfile().clone());
        if network.isLocalChannel() {
            self.currentLoginState = LoginState::ReadyToAccept;
            Ok(())
        } else {
            Err("NetHandlerLoginServer remote online-authentication branch is not owned by the integrated LocalChannel runtime".to_owned())
        }
    }
    pub fn update(&mut self, network: &mut NetworkManager) -> Result<LoginUpdate, String> {
        // MCP uses `if (connectionTimer++ == 600)`: compare the old value,
        // then increment even on the disconnect tick.
        let oldTimer = self.connectionTimer;
        self.connectionTimer = self.connectionTimer.wrapping_add(1);
        if oldTimer == 600 {
            return Err("multiplayer.disconnect.slow_login".to_owned());
        }
        if self.currentLoginState != LoginState::ReadyToAccept {
            return Ok(LoginUpdate::Waiting);
        }
        let mut profile = self
            .loginGameProfile
            .clone()
            .ok_or_else(|| "missing login GameProfile".to_owned())?;
        if !profile.isComplete() {
            profile = Self::getOfflineProfile(&profile);
        }
        self.currentLoginState = LoginState::Accepted;
        let packet = SPacketLoginSuccess::new(profile.clone())
            .writePacketData()
            .map_err(|e| e.to_string())?;
        network.sendPacket(&packet).map_err(|e| e.to_string())?;
        network.setConnectionState(ConnectionState::Play);
        self.loginGameProfile = Some(profile.clone());
        Ok(LoginUpdate::Accepted(profile))
    }

    /// Java `UUID.nameUUIDFromBytes(("OfflinePlayer:"+name).UTF_8)`.
    pub fn getOfflineProfile(original: &GameProfile) -> GameProfile {
        let digest = md5(format!("OfflinePlayer:{}", original.getName()).as_bytes());
        let mut bytes = digest;
        bytes[6] = (bytes[6] & 0x0f) | 0x30;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        GameProfile::new(Some(Uuid::from_bytes(bytes)), original.getName())
    }
}

// Compact RFC 1321 implementation used only for Java-compatible offline UUIDs.
fn md5(input: &[u8]) -> [u8; 16] {
    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    const K: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
        0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
        0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
        0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122,
        0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
        0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
        0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
        0xeb86d391,
    ];
    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut data = input.to_vec();
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_le_bytes());
    let (mut a0, mut b0, mut c0, mut d0) =
        (0x67452301u32, 0xefcdab89u32, 0x98badcfeu32, 0x10325476u32);
    for chunk in data.chunks_exact(64) {
        let mut m = [0u32; 16];
        for (i, w) in m.iter_mut().enumerate() {
            *w = u32::from_le_bytes(chunk[i * 4..i * 4 + 4].try_into().unwrap());
        }
        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);
        for i in 0..64 {
            let (f, g) = if i < 16 {
                ((b & c) | (!b & d), i)
            } else if i < 32 {
                ((d & b) | (!d & c), (5 * i + 1) % 16)
            } else if i < 48 {
                (b ^ c ^ d, (3 * i + 5) % 16)
            } else {
                (c ^ (b | !d), (7 * i) % 16)
            };
            let next = d;
            d = c;
            c = b;
            b = b.wrapping_add(
                a.wrapping_add(f)
                    .wrapping_add(K[i])
                    .wrapping_add(m[g])
                    .rotate_left(S[i]),
            );
            a = next;
        }
        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }
    let mut out = [0u8; 16];
    for (i, v) in [a0, b0, c0, d0].into_iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn offline_uuid_matches_java() {
        let p = GameProfile::new(None, "Player");
        assert_eq!(
            NetHandlerLoginServer::getOfflineProfile(&p)
                .getId()
                .unwrap()
                .to_string(),
            "a01e3843-e521-3998-958a-f459800e4d11"
        );
    }
}
