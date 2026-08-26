#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum SoundCategory {
    Master,
    Music,
    Records,
    Weather,
    Blocks,
    Hostile,
    Neutral,
    Players,
    Ambient,
    Voice,
}

impl SoundCategory {
    pub const ALL: [Self; 10] = [
        Self::Master,
        Self::Music,
        Self::Records,
        Self::Weather,
        Self::Blocks,
        Self::Hostile,
        Self::Neutral,
        Self::Players,
        Self::Ambient,
        Self::Voice,
    ];

    pub const fn getName(self) -> &'static str {
        match self {
            Self::Master => "master",
            Self::Music => "music",
            Self::Records => "record",
            Self::Weather => "weather",
            Self::Blocks => "block",
            Self::Hostile => "hostile",
            Self::Neutral => "neutral",
            Self::Players => "player",
            Self::Ambient => "ambient",
            Self::Voice => "voice",
        }
    }

    pub fn getByName(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.getName() == name)
    }

    pub const fn index(self) -> usize {
        match self {
            Self::Master => 0,
            Self::Music => 1,
            Self::Records => 2,
            Self::Weather => 3,
            Self::Blocks => 4,
            Self::Hostile => 5,
            Self::Neutral => 6,
            Self::Players => 7,
            Self::Ambient => 8,
            Self::Voice => 9,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_match_mcp_enum() {
        assert_eq!(SoundCategory::Records.getName(), "record");
        assert_eq!(
            SoundCategory::getByName("player"),
            Some(SoundCategory::Players)
        );
        assert!(SoundCategory::getByName("players").is_none());
    }
}
