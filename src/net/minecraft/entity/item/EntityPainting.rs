/// Exact MCP 1.12.2 `EntityPainting.EnumArt` motive table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaintingArt {
    Kebab,
    Aztec,
    Alban,
    Aztec2,
    Bomb,
    Plant,
    Wasteland,
    Pool,
    Courbet,
    Sea,
    Sunset,
    Creebet,
    Wanderer,
    Graham,
    Match,
    Bust,
    Stage,
    Void,
    SkullAndRoses,
    Wither,
    Fighters,
    Pointer,
    Pigscene,
    BurningSkull,
    Skeleton,
    DonkeyKong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaintingArtData {
    pub title: &'static str,
    pub sizeX: i32,
    pub sizeY: i32,
    pub offsetX: i32,
    pub offsetY: i32,
}

impl PaintingArt {
    pub const VALUES: [Self; 26] = [
        Self::Kebab,
        Self::Aztec,
        Self::Alban,
        Self::Aztec2,
        Self::Bomb,
        Self::Plant,
        Self::Wasteland,
        Self::Pool,
        Self::Courbet,
        Self::Sea,
        Self::Sunset,
        Self::Creebet,
        Self::Wanderer,
        Self::Graham,
        Self::Match,
        Self::Bust,
        Self::Stage,
        Self::Void,
        Self::SkullAndRoses,
        Self::Wither,
        Self::Fighters,
        Self::Pointer,
        Self::Pigscene,
        Self::BurningSkull,
        Self::Skeleton,
        Self::DonkeyKong,
    ];

    pub const fn data(self) -> PaintingArtData {
        match self {
            Self::Kebab => PaintingArtData {
                title: "Kebab",
                sizeX: 16,
                sizeY: 16,
                offsetX: 0,
                offsetY: 0,
            },
            Self::Aztec => PaintingArtData {
                title: "Aztec",
                sizeX: 16,
                sizeY: 16,
                offsetX: 16,
                offsetY: 0,
            },
            Self::Alban => PaintingArtData {
                title: "Alban",
                sizeX: 16,
                sizeY: 16,
                offsetX: 32,
                offsetY: 0,
            },
            Self::Aztec2 => PaintingArtData {
                title: "Aztec2",
                sizeX: 16,
                sizeY: 16,
                offsetX: 48,
                offsetY: 0,
            },
            Self::Bomb => PaintingArtData {
                title: "Bomb",
                sizeX: 16,
                sizeY: 16,
                offsetX: 64,
                offsetY: 0,
            },
            Self::Plant => PaintingArtData {
                title: "Plant",
                sizeX: 16,
                sizeY: 16,
                offsetX: 80,
                offsetY: 0,
            },
            Self::Wasteland => PaintingArtData {
                title: "Wasteland",
                sizeX: 16,
                sizeY: 16,
                offsetX: 96,
                offsetY: 0,
            },
            Self::Pool => PaintingArtData {
                title: "Pool",
                sizeX: 32,
                sizeY: 16,
                offsetX: 0,
                offsetY: 32,
            },
            Self::Courbet => PaintingArtData {
                title: "Courbet",
                sizeX: 32,
                sizeY: 16,
                offsetX: 32,
                offsetY: 32,
            },
            Self::Sea => PaintingArtData {
                title: "Sea",
                sizeX: 32,
                sizeY: 16,
                offsetX: 64,
                offsetY: 32,
            },
            Self::Sunset => PaintingArtData {
                title: "Sunset",
                sizeX: 32,
                sizeY: 16,
                offsetX: 96,
                offsetY: 32,
            },
            Self::Creebet => PaintingArtData {
                title: "Creebet",
                sizeX: 32,
                sizeY: 16,
                offsetX: 128,
                offsetY: 32,
            },
            Self::Wanderer => PaintingArtData {
                title: "Wanderer",
                sizeX: 16,
                sizeY: 32,
                offsetX: 0,
                offsetY: 64,
            },
            Self::Graham => PaintingArtData {
                title: "Graham",
                sizeX: 16,
                sizeY: 32,
                offsetX: 16,
                offsetY: 64,
            },
            Self::Match => PaintingArtData {
                title: "Match",
                sizeX: 32,
                sizeY: 32,
                offsetX: 0,
                offsetY: 128,
            },
            Self::Bust => PaintingArtData {
                title: "Bust",
                sizeX: 32,
                sizeY: 32,
                offsetX: 32,
                offsetY: 128,
            },
            Self::Stage => PaintingArtData {
                title: "Stage",
                sizeX: 32,
                sizeY: 32,
                offsetX: 64,
                offsetY: 128,
            },
            Self::Void => PaintingArtData {
                title: "Void",
                sizeX: 32,
                sizeY: 32,
                offsetX: 96,
                offsetY: 128,
            },
            Self::SkullAndRoses => PaintingArtData {
                title: "SkullAndRoses",
                sizeX: 32,
                sizeY: 32,
                offsetX: 128,
                offsetY: 128,
            },
            Self::Wither => PaintingArtData {
                title: "Wither",
                sizeX: 32,
                sizeY: 32,
                offsetX: 160,
                offsetY: 128,
            },
            Self::Fighters => PaintingArtData {
                title: "Fighters",
                sizeX: 64,
                sizeY: 32,
                offsetX: 0,
                offsetY: 96,
            },
            Self::Pointer => PaintingArtData {
                title: "Pointer",
                sizeX: 64,
                sizeY: 64,
                offsetX: 0,
                offsetY: 192,
            },
            Self::Pigscene => PaintingArtData {
                title: "Pigscene",
                sizeX: 64,
                sizeY: 64,
                offsetX: 64,
                offsetY: 192,
            },
            Self::BurningSkull => PaintingArtData {
                title: "BurningSkull",
                sizeX: 64,
                sizeY: 64,
                offsetX: 128,
                offsetY: 192,
            },
            Self::Skeleton => PaintingArtData {
                title: "Skeleton",
                sizeX: 64,
                sizeY: 48,
                offsetX: 192,
                offsetY: 64,
            },
            Self::DonkeyKong => PaintingArtData {
                title: "DonkeyKong",
                sizeX: 64,
                sizeY: 48,
                offsetX: 192,
                offsetY: 112,
            },
        }
    }

    pub fn fromTitle(title: &str) -> Self {
        Self::VALUES
            .into_iter()
            .find(|art| art.data().title == title)
            .unwrap_or(Self::Kebab)
    }
}

pub struct EntityPainting;

impl EntityPainting {
    pub fn art(title: &str) -> PaintingArt {
        PaintingArt::fromTitle(title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_motive_table_has_twenty_six_entries_and_falls_back_to_kebab() {
        assert_eq!(PaintingArt::VALUES.len(), 26);
        assert_eq!(
            PaintingArt::fromTitle("SkullAndRoses"),
            PaintingArt::SkullAndRoses
        );
        assert_eq!(PaintingArt::fromTitle("unknown"), PaintingArt::Kebab);
    }
}
