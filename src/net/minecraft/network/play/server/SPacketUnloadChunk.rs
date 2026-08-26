use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_i32_be, write_i32_be, CodecError};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPacketUnloadChunk {
    x: i32,
    z: i32,
}
impl SPacketUnloadChunk {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
    pub fn writePacketData(&self) -> RawPacket {
        let mut payload = Vec::with_capacity(8);
        write_i32_be(self.x, &mut payload);
        write_i32_be(self.z, &mut payload);
        RawPacket::new(0x1D, payload)
    }
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        Ok(Self {
            x: read_i32_be(&mut input)?,
            z: read_i32_be(&mut input)?,
        })
    }
    pub const fn getX(&self) -> i32 {
        self.x
    }
    pub const fn getZ(&self) -> i32 {
        self.z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unload_chunk_round_trips_chunk_coordinates() {
        let packet = SPacketUnloadChunk::new(-193, 271);
        let raw = packet.writePacketData();
        assert_eq!(raw.id, 0x1D);
        assert_eq!(SPacketUnloadChunk::readPacketData(&raw).unwrap(), packet);
    }
}
