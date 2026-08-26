use crate::net::minecraft::network::PacketBuffer::CodecError;

/// MCP 1.12.2 `EntityEquipmentSlot` in declaration/network ordinal order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityEquipmentSlot {
    Mainhand,
    Offhand,
    Feet,
    Legs,
    Chest,
    Head,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Hand,
    Armor,
}

impl EntityEquipmentSlot {
    pub const ALL: [Self; 6] = [
        Self::Mainhand,
        Self::Offhand,
        Self::Feet,
        Self::Legs,
        Self::Chest,
        Self::Head,
    ];

    /// `PacketBuffer.readEnumValue` writes/reads the Java enum ordinal as a VarInt.
    pub fn fromNetworkOrdinal(ordinal: i32) -> Result<Self, CodecError> {
        match ordinal {
            0 => Ok(Self::Mainhand),
            1 => Ok(Self::Offhand),
            2 => Ok(Self::Feet),
            3 => Ok(Self::Legs),
            4 => Ok(Self::Chest),
            5 => Ok(Self::Head),
            value => Err(CodecError::InvalidData(format!(
                "invalid EntityEquipmentSlot ordinal {value}"
            ))),
        }
    }

    pub const fn networkOrdinal(self) -> i32 {
        match self {
            Self::Mainhand => 0,
            Self::Offhand => 1,
            Self::Feet => 2,
            Self::Legs => 3,
            Self::Chest => 4,
            Self::Head => 5,
        }
    }

    pub const fn getSlotType(self) -> Type {
        match self {
            Self::Mainhand | Self::Offhand => Type::Hand,
            Self::Feet | Self::Legs | Self::Chest | Self::Head => Type::Armor,
        }
    }

    pub const fn getIndex(self) -> usize {
        match self {
            Self::Mainhand | Self::Feet => 0,
            Self::Offhand | Self::Legs => 1,
            Self::Chest => 2,
            Self::Head => 3,
        }
    }

    /// MCP actual equipment slot index: main hand 0, armor 1-4, offhand 5.
    pub const fn getSlotIndex(self) -> usize {
        match self {
            Self::Mainhand => 0,
            Self::Feet => 1,
            Self::Legs => 2,
            Self::Chest => 3,
            Self::Head => 4,
            Self::Offhand => 5,
        }
    }

    pub const fn getName(self) -> &'static str {
        match self {
            Self::Mainhand => "mainhand",
            Self::Offhand => "offhand",
            Self::Feet => "feet",
            Self::Legs => "legs",
            Self::Chest => "chest",
            Self::Head => "head",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinals_match_mcp_declaration_order() {
        for (ordinal, slot) in EntityEquipmentSlot::ALL.into_iter().enumerate() {
            assert_eq!(slot.networkOrdinal(), ordinal as i32);
            assert_eq!(
                EntityEquipmentSlot::fromNetworkOrdinal(ordinal as i32).unwrap(),
                slot
            );
        }
    }
}
