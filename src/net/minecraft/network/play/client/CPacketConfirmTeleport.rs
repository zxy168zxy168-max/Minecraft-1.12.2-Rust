use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::write_var_i32;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPacketConfirmTeleport {
    telportId: i32,
}
impl CPacketConfirmTeleport {
    pub const fn new(teleportIdIn: i32) -> Self {
        Self {
            telportId: teleportIdIn,
        }
    }
    pub fn writePacketData(&self) -> RawPacket {
        let mut payload = Vec::new();
        write_var_i32(self.telportId, &mut payload);
        RawPacket::new(0, payload)
    }
    pub const fn getTeleportId(&self) -> i32 {
        self.telportId
    }
}
