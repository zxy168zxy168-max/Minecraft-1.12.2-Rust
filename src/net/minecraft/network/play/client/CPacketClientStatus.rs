use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::write_var_i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    PerformRespawn,
    RequestStats,
}

impl State {
    const fn ordinal(self) -> i32 {
        match self {
            Self::PerformRespawn => 0,
            Self::RequestStats => 1,
        }
    }
}

/// MCP 1.12.2 `CPacketClientStatus` (serverbound play packet 0x03).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPacketClientStatus {
    status: State,
}

impl CPacketClientStatus {
    pub const fn new(status: State) -> Self {
        Self { status }
    }
    pub const fn getStatus(self) -> State {
        self.status
    }

    pub fn writePacketData(self) -> RawPacket {
        let mut payload = Vec::new();
        write_var_i32(self.status.ordinal(), &mut payload);
        RawPacket::new(0x03, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perform_respawn_uses_status_zero() {
        let packet = CPacketClientStatus::new(State::PerformRespawn).writePacketData();
        assert_eq!(packet.id, 0x03);
        assert_eq!(packet.payload, vec![0]);
    }
}
