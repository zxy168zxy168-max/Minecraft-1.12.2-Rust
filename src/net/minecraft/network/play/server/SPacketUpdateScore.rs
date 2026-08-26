use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_string, read_var_i32, CodecError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Change,
    Remove,
}

/// Protocol 340 clientbound 0x45, matching MCP 1.12.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketUpdateScore {
    name: String,
    objective: String,
    value: i32,
    action: Action,
}

impl SPacketUpdateScore {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let name = read_string(&mut input, 40)?;
        let action = match read_var_i32(&mut input)? {
            1 => Action::Remove,
            _ => Action::Change,
        };
        let objective = read_string(&mut input, 16)?;
        let value = if action == Action::Change {
            read_var_i32(&mut input)?
        } else {
            0
        };
        Ok(Self {
            name,
            objective,
            value,
            action,
        })
    }
    pub fn getPlayerName(&self) -> &str {
        &self.name
    }
    pub fn getObjectiveName(&self) -> &str {
        &self.objective
    }
    pub const fn getScoreValue(&self) -> i32 {
        self.value
    }
    pub const fn getScoreAction(&self) -> Action {
        self.action
    }
}
