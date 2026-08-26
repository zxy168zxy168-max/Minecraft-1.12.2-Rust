#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumPlayerModelParts {
    Cape,
    Jacket,
    LeftSleeve,
    RightSleeve,
    LeftPantsLeg,
    RightPantsLeg,
    Hat,
}

impl EnumPlayerModelParts {
    pub const VALUES: [Self; 7] = [
        Self::Cape,
        Self::Jacket,
        Self::LeftSleeve,
        Self::RightSleeve,
        Self::LeftPantsLeg,
        Self::RightPantsLeg,
        Self::Hat,
    ];

    pub const fn getPartId(self) -> i32 {
        match self {
            Self::Cape => 0,
            Self::Jacket => 1,
            Self::LeftSleeve => 2,
            Self::RightSleeve => 3,
            Self::LeftPantsLeg => 4,
            Self::RightPantsLeg => 5,
            Self::Hat => 6,
        }
    }

    pub const fn getPartMask(self) -> u8 {
        1_u8 << self.getPartId()
    }

    pub const fn getPartName(self) -> &'static str {
        match self {
            Self::Cape => "cape",
            Self::Jacket => "jacket",
            Self::LeftSleeve => "left_sleeve",
            Self::RightSleeve => "right_sleeve",
            Self::LeftPantsLeg => "left_pants_leg",
            Self::RightPantsLeg => "right_pants_leg",
            Self::Hat => "hat",
        }
    }
}
