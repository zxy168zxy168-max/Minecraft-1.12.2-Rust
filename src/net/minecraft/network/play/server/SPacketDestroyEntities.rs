use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_var_i32, CodecError};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketDestroyEntities {
    entityIDs: Vec<i32>,
}
impl SPacketDestroyEntities {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let count = read_var_i32(&mut input)?;
        if count < 0 || count > 1_000_000 {
            return Err(CodecError::InvalidData(format!(
                "invalid destroy-entity count {count}"
            )));
        }
        let mut ids = Vec::with_capacity(count as usize);
        for _ in 0..count {
            ids.push(read_var_i32(&mut input)?);
        }
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread destroy-entity bytes",
                input.len()
            )));
        }
        Ok(Self { entityIDs: ids })
    }
    pub fn getEntityIDs(&self) -> &[i32] {
        &self.entityIDs
    }
}
