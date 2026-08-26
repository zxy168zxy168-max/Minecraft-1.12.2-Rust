use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::write_var_i32;

/// Protocol-340 port of MCP `CPacketEntityAction`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    StartSneaking,
    StopSneaking,
    StopSleeping,
    StartSprinting,
    StopSprinting,
    StartRidingJump,
    StopRidingJump,
    OpenInventory,
    StartFallFlying,
}

impl Action {
    pub const fn ordinal(self) -> i32 {
        match self {
            Self::StartSneaking => 0,
            Self::StopSneaking => 1,
            Self::StopSleeping => 2,
            Self::StartSprinting => 3,
            Self::StopSprinting => 4,
            Self::StartRidingJump => 5,
            Self::StopRidingJump => 6,
            Self::OpenInventory => 7,
            Self::StartFallFlying => 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPacketEntityAction {
    entityId: i32,
    action: Action,
    auxData: i32,
}

impl CPacketEntityAction {
    pub const fn new(entityId: i32, action: Action) -> Self {
        Self {
            entityId,
            action,
            auxData: 0,
        }
    }

    pub const fn withAuxData(entityId: i32, action: Action, auxData: i32) -> Self {
        Self {
            entityId,
            action,
            auxData,
        }
    }

    pub fn writePacketData(&self) -> RawPacket {
        let mut payload = Vec::with_capacity(8);
        write_var_i32(self.entityId, &mut payload);
        write_var_i32(self.action.ordinal(), &mut payload);
        write_var_i32(self.auxData, &mut payload);
        RawPacket::new(0x15, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sneaking_action_uses_vanilla_enum_ordinal_and_packet_id() {
        let packet = CPacketEntityAction::new(42, Action::StartSneaking).writePacketData();
        assert_eq!(packet.id, 0x15);
        assert_eq!(packet.payload, vec![42, 0, 0]);
    }

    #[test]
    fn riding_jump_preserves_action_ordinal_and_aux_power() {
        let packet =
            CPacketEntityAction::withAuxData(7, Action::StartRidingJump, 83).writePacketData();
        assert_eq!(packet.id, 0x15);
        assert_eq!(packet.payload, vec![7, 5, 83]);
    }
}
