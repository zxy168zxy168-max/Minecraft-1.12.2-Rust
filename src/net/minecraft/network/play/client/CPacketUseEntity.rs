use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{write_f32_be, write_var_i32};
use crate::net::minecraft::util::math::Vec3d::Vec3d;
use crate::net::minecraft::util::EnumHand::EnumHand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Interact,
    Attack,
    InteractAt,
}
impl Action {
    const fn id(self) -> i32 {
        match self {
            Self::Interact => 0,
            Self::Attack => 1,
            Self::InteractAt => 2,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CPacketUseEntity {
    entityId: i32,
    action: Action,
    hitVec: Option<Vec3d>,
    hand: Option<EnumHand>,
}
impl CPacketUseEntity {
    pub const fn attack(entityId: i32) -> Self {
        Self {
            entityId,
            action: Action::Attack,
            hitVec: None,
            hand: None,
        }
    }
    pub const fn interact(entityId: i32, hand: EnumHand) -> Self {
        Self {
            entityId,
            action: Action::Interact,
            hitVec: None,
            hand: Some(hand),
        }
    }
    pub const fn interactAt(entityId: i32, hand: EnumHand, hitVec: Vec3d) -> Self {
        Self {
            entityId,
            action: Action::InteractAt,
            hitVec: Some(hitVec),
            hand: Some(hand),
        }
    }
    pub fn writePacketData(self) -> RawPacket {
        let mut payload = Vec::new();
        write_var_i32(self.entityId, &mut payload);
        write_var_i32(self.action.id(), &mut payload);
        if let Some(hit) = self.hitVec {
            write_f32_be(hit.x as f32, &mut payload);
            write_f32_be(hit.y as f32, &mut payload);
            write_f32_be(hit.z as f32, &mut payload);
        }
        if let Some(hand) = self.hand {
            write_var_i32(hand.ordinal(), &mut payload);
        }
        RawPacket::new(0x0A, payload)
    }
}
