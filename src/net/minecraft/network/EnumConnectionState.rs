#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ConnectionState {
    Handshaking = -1,
    Play = 0,
    Status = 1,
    Login = 2,
}

impl ConnectionState {
    pub fn fromProtocolId(id: i32) -> Option<Self> {
        match id {
            -1 => Some(Self::Handshaking),
            0 => Some(Self::Play),
            1 => Some(Self::Status),
            2 => Some(Self::Login),
            _ => None,
        }
    }
}
