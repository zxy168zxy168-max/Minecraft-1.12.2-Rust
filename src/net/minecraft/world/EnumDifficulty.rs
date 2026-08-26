#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumDifficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}
impl EnumDifficulty {
    pub const fn getDifficultyEnum(id: u8) -> Self {
        match id % 4 {
            0 => Self::Peaceful,
            1 => Self::Easy,
            2 => Self::Normal,
            _ => Self::Hard,
        }
    }
    pub const fn getDifficultyId(self) -> u8 {
        match self {
            Self::Peaceful => 0,
            Self::Easy => 1,
            Self::Normal => 2,
            Self::Hard => 3,
        }
    }
}
