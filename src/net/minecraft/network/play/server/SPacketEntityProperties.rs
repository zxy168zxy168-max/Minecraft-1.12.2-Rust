use crate::net::minecraft::entity::ai::attributes::AttributeModifier::AttributeModifier;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_f64_be, read_i32_be, read_i8, read_string, read_uuid, read_var_i32, CodecError,
};

/// Protocol-340 port of MCP `SPacketEntityProperties` (clientbound 0x4E).
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    name: String,
    baseValue: f64,
    modifiers: Vec<AttributeModifier>,
}

impl Snapshot {
    pub fn getName(&self) -> &str {
        &self.name
    }
    pub const fn getBaseValue(&self) -> f64 {
        self.baseValue
    }
    pub fn getModifiers(&self) -> &[AttributeModifier] {
        &self.modifiers
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SPacketEntityProperties {
    entityId: i32,
    snapshots: Vec<Snapshot>,
}

impl SPacketEntityProperties {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let entityId = read_var_i32(&mut input)?;
        let snapshotCount = read_i32_be(&mut input)?;
        if snapshotCount < 0 {
            return Err(CodecError::NegativeLength(snapshotCount));
        }
        if snapshotCount > 1024 {
            return Err(CodecError::InvalidData(format!(
                "entity attribute snapshot count {} exceeds client safety limit",
                snapshotCount
            )));
        }

        let mut snapshots = Vec::with_capacity(snapshotCount as usize);
        for _ in 0..snapshotCount {
            let name = read_string(&mut input, 64)?;
            let baseValue = read_f64_be(&mut input)?;
            let modifierCount = read_var_i32(&mut input)?;
            if modifierCount < 0 {
                return Err(CodecError::NegativeLength(modifierCount));
            }
            if modifierCount > 1024 {
                return Err(CodecError::InvalidData(format!(
                    "attribute modifier count {} exceeds client safety limit",
                    modifierCount
                )));
            }
            let mut modifiers = Vec::with_capacity(modifierCount as usize);
            for _ in 0..modifierCount {
                let id = read_uuid(&mut input)?;
                let amount = read_f64_be(&mut input)?;
                let operation = read_i8(&mut input)?;
                if !(0..=2).contains(&operation) {
                    return Err(CodecError::InvalidData(format!(
                        "invalid attribute modifier operation {}",
                        operation
                    )));
                }
                modifiers.push(AttributeModifier::new(id, amount, operation));
            }
            snapshots.push(Snapshot {
                name,
                baseValue,
                modifiers,
            });
        }

        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread entity-properties bytes",
                input.len()
            )));
        }
        Ok(Self {
            entityId,
            snapshots,
        })
    }

    pub const fn getEntityId(&self) -> i32 {
        self.entityId
    }
    pub fn getSnapshots(&self) -> &[Snapshot] {
        &self.snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{
        write_f64_be, write_i32_be, write_string, write_uuid, write_var_i32,
    };
    use uuid::Uuid;

    #[test]
    fn reads_protocol_340_attribute_snapshot() {
        let mut payload = Vec::new();
        write_var_i32(7, &mut payload);
        write_i32_be(1, &mut payload);
        write_string("horse.jumpStrength", 64, &mut payload).unwrap();
        write_f64_be(0.7, &mut payload);
        write_var_i32(1, &mut payload);
        let id = Uuid::from_u128(0x1234);
        write_uuid(id, &mut payload);
        write_f64_be(0.1, &mut payload);
        payload.push(2);
        let packet =
            SPacketEntityProperties::readPacketData(&RawPacket::new(0x4E, payload)).unwrap();
        assert_eq!(packet.getEntityId(), 7);
        assert_eq!(packet.getSnapshots()[0].getName(), "horse.jumpStrength");
        assert_eq!(packet.getSnapshots()[0].getModifiers()[0].getID(), id);
    }
}
