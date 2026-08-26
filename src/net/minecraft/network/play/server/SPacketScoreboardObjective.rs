use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i8, read_string, CodecError};
use crate::net::minecraft::scoreboard::IScoreCriteria::EnumRenderType;

/// Protocol 340 clientbound 0x42, matching MCP 1.12.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketScoreboardObjective {
    objectiveName: String,
    objectiveValue: String,
    type_: EnumRenderType,
    action: i32,
}

impl SPacketScoreboardObjective {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let objectiveName = read_string(&mut input, 16)?;
        let action = read_i8(&mut input)? as i32;
        let (objectiveValue, type_) = if action == 0 || action == 2 {
            (
                read_string(&mut input, 32)?,
                EnumRenderType::getByName(&read_string(&mut input, 16)?),
            )
        } else {
            (String::new(), EnumRenderType::Integer)
        };
        Ok(Self {
            objectiveName,
            objectiveValue,
            type_,
            action,
        })
    }

    pub fn getObjectiveName(&self) -> &str {
        &self.objectiveName
    }
    pub fn getObjectiveValue(&self) -> &str {
        &self.objectiveValue
    }
    pub const fn getAction(&self) -> i32 {
        self.action
    }
    pub const fn getRenderType(&self) -> EnumRenderType {
        self.type_
    }
}
