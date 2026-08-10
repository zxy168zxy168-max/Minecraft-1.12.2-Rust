/// MCP 1.12.2 `WorldType` registry values used by world creation and join/respawn packets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorldType {
    Default,
    Flat,
    LargeBiomes,
    Amplified,
    Customized,
    DebugWorld,
    Default11,
}

impl Default for WorldType {
    fn default() -> Self { Self::Default }
}

impl WorldType {
    pub const CREATABLE: [Self; 6] = [
        Self::Default,
        Self::Flat,
        Self::LargeBiomes,
        Self::Amplified,
        Self::Customized,
        Self::DebugWorld,
    ];

    pub fn parseWorldType(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "flat" => Self::Flat,
            "largebiomes" => Self::LargeBiomes,
            "amplified" => Self::Amplified,
            "customized" => Self::Customized,
            "debug_all_block_states" => Self::DebugWorld,
            "default_1_1" => Self::Default11,
            _ => Self::Default,
        }
    }

    pub const fn getWorldTypeName(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Flat => "flat",
            Self::LargeBiomes => "largeBiomes",
            Self::Amplified => "amplified",
            Self::Customized => "customized",
            Self::DebugWorld => "debug_all_block_states",
            Self::Default11 => "default_1_1",
        }
    }

    pub const fn getTranslateName(self) -> &'static str {
        match self {
            Self::Default => "generator.default",
            Self::Flat => "generator.flat",
            Self::LargeBiomes => "generator.largeBiomes",
            Self::Amplified => "generator.amplified",
            Self::Customized => "generator.customized",
            Self::DebugWorld => "generator.debug_all_block_states",
            Self::Default11 => "generator.default_1_1",
        }
    }

    pub const fn getTranslatedInfo(self) -> &'static str {
        match self {
            Self::Default => "generator.default.info",
            Self::Flat => "generator.flat.info",
            Self::LargeBiomes => "generator.largeBiomes.info",
            Self::Amplified => "generator.amplified.info",
            Self::Customized => "generator.customized.info",
            Self::DebugWorld => "generator.debug_all_block_states.info",
            Self::Default11 => "generator.default_1_1.info",
        }
    }

    pub const fn getGeneratorVersion(self) -> i32 {
        match self {
            Self::Default => 1,
            Self::Default11 => 0,
            _ => 0,
        }
    }

    pub const fn getWorldTypeID(self) -> i32 {
        match self {
            Self::Default => 0,
            Self::Flat => 1,
            Self::LargeBiomes => 2,
            Self::Amplified => 3,
            Self::Customized => 4,
            Self::DebugWorld => 5,
            Self::Default11 => 8,
        }
    }

    pub const fn getCanBeCreated(self) -> bool { !matches!(self, Self::Default11) }
    pub const fn isVersioned(self) -> bool { matches!(self, Self::Default) }
    pub const fn showWorldInfoNotice(self) -> bool { matches!(self, Self::Amplified) }

    pub const fn getWorldTypeForGeneratorVersion(self, version: i32) -> Self {
        if matches!(self, Self::Default) && version == 0 { Self::Default11 } else { self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_ids_and_names_are_preserved() {
        assert_eq!(WorldType::Default.getWorldTypeID(), 0);
        assert_eq!(WorldType::LargeBiomes.getWorldTypeName(), "largeBiomes");
        assert_eq!(WorldType::DebugWorld.getWorldTypeID(), 5);
        assert!(!WorldType::Default11.getCanBeCreated());
        assert!(WorldType::Amplified.showWorldInfoNotice());
        assert_eq!(WorldType::parseWorldType("unknown"), WorldType::Default);
    }
}
