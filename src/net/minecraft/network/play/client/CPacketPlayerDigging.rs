use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{write_i64_be, write_var_i32};
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

/// Protocol-340 port of MCP 1.12.2 `CPacketPlayerDigging`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    StartDestroyBlock,
    AbortDestroyBlock,
    StopDestroyBlock,
    DropAllItems,
    DropItem,
    ReleaseUseItem,
    SwapHeldItems,
}

impl Action {
    pub const fn ordinal(self) -> i32 {
        match self {
            Self::StartDestroyBlock => 0,
            Self::AbortDestroyBlock => 1,
            Self::StopDestroyBlock => 2,
            Self::DropAllItems => 3,
            Self::DropItem => 4,
            Self::ReleaseUseItem => 5,
            Self::SwapHeldItems => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPacketPlayerDigging {
    pub action: Action,
    pub position: BlockPos,
    pub facing: EnumFacing,
}

impl CPacketPlayerDigging {
    pub const fn new(actionIn: Action, posIn: BlockPos, facingIn: EnumFacing) -> Self {
        Self {
            action: actionIn,
            position: posIn,
            facing: facingIn,
        }
    }

    pub fn writePacketData(self) -> RawPacket {
        let mut payload = Vec::with_capacity(10);
        write_var_i32(self.action.ordinal(), &mut payload);
        write_i64_be(self.position.to_long(), &mut payload);
        payload.push(self.facing.index() as u8);
        RawPacket::new(0x14, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_340_id_and_mcp_enum_order_match() {
        let packet = CPacketPlayerDigging::new(
            Action::StopDestroyBlock,
            BlockPos::new(1, 2, 3),
            EnumFacing::West,
        )
        .writePacketData();
        assert_eq!(packet.id, 0x14);
        assert_eq!(packet.payload[0], 2);
        assert_eq!(*packet.payload.last().unwrap(), 4);
    }
}
