use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    write_bool, write_i64_be, write_string, CodecError,
};
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// Protocol 340 serverbound 0x01, matching MCP 1.12.2
/// `CPacketTabComplete` field order and optional target block encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CPacketTabComplete {
    message: String,
    hasTargetBlock: bool,
    targetBlock: Option<BlockPos>,
}

impl CPacketTabComplete {
    pub fn new(
        messageIn: impl Into<String>,
        targetBlockIn: Option<BlockPos>,
        hasTargetBlockIn: bool,
    ) -> Self {
        Self {
            message: truncate_utf16(&messageIn.into(), 32_767),
            targetBlock: targetBlockIn,
            hasTargetBlock: hasTargetBlockIn,
        }
    }

    pub fn writePacketData(&self) -> Result<RawPacket, CodecError> {
        let mut payload = Vec::new();
        write_string(&self.message, 32_767, &mut payload)?;
        write_bool(self.hasTargetBlock, &mut payload);
        write_bool(self.targetBlock.is_some(), &mut payload);
        if let Some(position) = self.targetBlock {
            write_i64_be(position.to_long(), &mut payload);
        }
        Ok(RawPacket::new(0x01, payload))
    }

    pub fn getMessage(&self) -> &str {
        &self.message
    }
    pub const fn getTargetBlock(&self) -> Option<BlockPos> {
        self.targetBlock
    }
    pub const fn hasTargetBlock(&self) -> bool {
        self.hasTargetBlock
    }
}

fn truncate_utf16(value: &str, maximum: usize) -> String {
    String::from_utf16_lossy(&value.encode_utf16().take(maximum).collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{read_bool, read_i64_be, read_string};

    #[test]
    fn writes_protocol_340_order_and_optional_block() {
        let position = BlockPos::new(-12, 64, 33);
        let raw = CPacketTabComplete::new("/give Pla", Some(position), true)
            .writePacketData()
            .unwrap();
        assert_eq!(raw.id, 0x01);
        let mut input = raw.payload.as_slice();
        assert_eq!(read_string(&mut input, 32_767).unwrap(), "/give Pla");
        assert!(read_bool(&mut input).unwrap());
        assert!(read_bool(&mut input).unwrap());
        assert_eq!(
            BlockPos::from_long(read_i64_be(&mut input).unwrap()),
            position
        );
        assert!(input.is_empty());
    }
}
