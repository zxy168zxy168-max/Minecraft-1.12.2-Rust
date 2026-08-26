use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_i8, read_string, read_var_i32, CodecError,
};

/// Protocol 340 clientbound 0x44, matching MCP 1.12.2 `SPacketTeams`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketTeams {
    name: String,
    displayName: String,
    prefix: String,
    suffix: String,
    nameTagVisibility: String,
    collisionRule: String,
    color: i32,
    players: Vec<String>,
    action: i32,
    friendlyFlags: i32,
}

impl SPacketTeams {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let name = read_string(&mut input, 16)?;
        let action = read_i8(&mut input)? as i32;
        let mut result = Self {
            name,
            displayName: String::new(),
            prefix: String::new(),
            suffix: String::new(),
            nameTagVisibility: "always".to_owned(),
            collisionRule: "always".to_owned(),
            color: -1,
            players: Vec::new(),
            action,
            friendlyFlags: 0,
        };
        if action == 0 || action == 2 {
            result.displayName = read_string(&mut input, 32)?;
            result.prefix = read_string(&mut input, 16)?;
            result.suffix = read_string(&mut input, 16)?;
            result.friendlyFlags = read_i8(&mut input)? as i32;
            result.nameTagVisibility = read_string(&mut input, 32)?;
            result.collisionRule = read_string(&mut input, 32)?;
            result.color = read_i8(&mut input)? as i32;
        }
        if action == 0 || action == 3 || action == 4 {
            let count = read_var_i32(&mut input)?;
            if count < 0 {
                return Err(CodecError::NegativeLength(count));
            }
            for _ in 0..count {
                result.players.push(read_string(&mut input, 40)?);
            }
        }
        Ok(result)
    }

    pub fn getName(&self) -> &str {
        &self.name
    }
    pub fn getDisplayName(&self) -> &str {
        &self.displayName
    }
    pub fn getPrefix(&self) -> &str {
        &self.prefix
    }
    pub fn getSuffix(&self) -> &str {
        &self.suffix
    }
    pub fn getPlayers(&self) -> &[String] {
        &self.players
    }
    pub const fn getAction(&self) -> i32 {
        self.action
    }
    pub const fn getFriendlyFlags(&self) -> i32 {
        self.friendlyFlags
    }
    pub const fn getColor(&self) -> i32 {
        self.color
    }
    pub fn getNameTagVisibility(&self) -> &str {
        &self.nameTagVisibility
    }
    pub fn getCollisionRule(&self) -> &str {
        &self.collisionRule
    }
}
