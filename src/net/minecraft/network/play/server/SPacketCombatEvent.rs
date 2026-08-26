use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_i32_be, read_text_component, read_var_i32, CodecError,
};
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    EnterCombat,
    EndCombat,
    EntityDied,
}

impl Event {
    fn fromOrdinal(value: i32) -> Result<Self, CodecError> {
        match value {
            0 => Ok(Self::EnterCombat),
            1 => Ok(Self::EndCombat),
            2 => Ok(Self::EntityDied),
            _ => Err(CodecError::InvalidData(format!(
                "invalid SPacketCombatEvent event ordinal {value}",
            ))),
        }
    }
}

/// MCP 1.12.2 `SPacketCombatEvent` (clientbound play packet 0x2D).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketCombatEvent {
    eventType: Event,
    playerId: i32,
    entityId: i32,
    duration: i32,
    deathMessage: Option<ITextComponent>,
}

impl SPacketCombatEvent {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let eventType = Event::fromOrdinal(read_var_i32(&mut input)?)?;
        let mut result = Self {
            eventType,
            playerId: 0,
            entityId: -1,
            duration: 0,
            deathMessage: None,
        };
        match eventType {
            Event::EnterCombat => {}
            Event::EndCombat => {
                result.duration = read_var_i32(&mut input)?;
                result.entityId = read_i32_be(&mut input)?;
            }
            Event::EntityDied => {
                result.playerId = read_var_i32(&mut input)?;
                result.entityId = read_i32_be(&mut input)?;
                result.deathMessage = Some(read_text_component(&mut input)?);
            }
        }
        Ok(result)
    }

    pub const fn getEventType(&self) -> Event {
        self.eventType
    }
    pub const fn getPlayerId(&self) -> i32 {
        self.playerId
    }
    pub const fn getEntityId(&self) -> i32 {
        self.entityId
    }
    pub const fn getDuration(&self) -> i32 {
        self.duration
    }
    pub fn getDeathMessage(&self) -> Option<&ITextComponent> {
        self.deathMessage.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{write_i32_be, write_string, write_var_i32};

    #[test]
    fn entity_died_reads_player_attacker_and_component() {
        let mut payload = Vec::new();
        write_var_i32(2, &mut payload);
        write_var_i32(17, &mut payload);
        write_i32_be(29, &mut payload);
        write_string(
            r#"{"translate":"death.attack.generic"}"#,
            32_767,
            &mut payload,
        )
        .unwrap();
        let packet = SPacketCombatEvent::readPacketData(&RawPacket::new(0x2D, payload)).unwrap();
        assert_eq!(packet.getEventType(), Event::EntityDied);
        assert_eq!(packet.getPlayerId(), 17);
        assert_eq!(packet.getEntityId(), 29);
        assert!(packet.getDeathMessage().is_some());
    }
}
