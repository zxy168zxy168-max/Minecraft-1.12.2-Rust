#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PotionMeta {
    pub statusIconIndex: i32,
    /// MCP `Potion#liquidColor`, used by `PotionEffect#compareTo`.
    pub liquidColor: i32,
    pub beneficial: bool,
}

impl PotionMeta {
    pub const fn hasStatusIcon(&self) -> bool { self.statusIconIndex >= 0 }
    pub const fn iconRect(&self) -> (i32, i32) {
        (self.statusIconIndex % 8 * 18, 198 + self.statusIconIndex / 8 * 18)
    }
}

/// MCP 1.12.2 Potion registrations needed by the HUD.
pub const fn potion_meta(potionId: u8) -> Option<PotionMeta> {
    let meta = match potionId {
        1 => PotionMeta { statusIconIndex: 0, liquidColor: 8171462, beneficial: true },
        2 => PotionMeta { statusIconIndex: 1, liquidColor: 5926017, beneficial: false },
        3 => PotionMeta { statusIconIndex: 2, liquidColor: 14270531, beneficial: true },
        4 => PotionMeta { statusIconIndex: 3, liquidColor: 4866583, beneficial: false },
        5 => PotionMeta { statusIconIndex: 4, liquidColor: 9643043, beneficial: true },
        6 => PotionMeta { statusIconIndex: -1, liquidColor: 16262179, beneficial: true },
        7 => PotionMeta { statusIconIndex: -1, liquidColor: 4393481, beneficial: true },
        8 => PotionMeta { statusIconIndex: 10, liquidColor: 2293580, beneficial: true },
        9 => PotionMeta { statusIconIndex: 11, liquidColor: 5578058, beneficial: false },
        10 => PotionMeta { statusIconIndex: 7, liquidColor: 13458603, beneficial: true },
        11 => PotionMeta { statusIconIndex: 14, liquidColor: 10044730, beneficial: true },
        12 => PotionMeta { statusIconIndex: 15, liquidColor: 14981690, beneficial: true },
        13 => PotionMeta { statusIconIndex: 16, liquidColor: 3035801, beneficial: true },
        14 => PotionMeta { statusIconIndex: 8, liquidColor: 8356754, beneficial: true },
        15 => PotionMeta { statusIconIndex: 13, liquidColor: 2039587, beneficial: false },
        16 => PotionMeta { statusIconIndex: 12, liquidColor: 2039713, beneficial: true },
        17 => PotionMeta { statusIconIndex: 9, liquidColor: 5797459, beneficial: false },
        18 => PotionMeta { statusIconIndex: 5, liquidColor: 4738376, beneficial: false },
        19 => PotionMeta { statusIconIndex: 6, liquidColor: 5149489, beneficial: false },
        20 => PotionMeta { statusIconIndex: 17, liquidColor: 3484199, beneficial: false },
        21 => PotionMeta { statusIconIndex: 23, liquidColor: 16284963, beneficial: true },
        22 => PotionMeta { statusIconIndex: 18, liquidColor: 2445989, beneficial: true },
        23 => PotionMeta { statusIconIndex: -1, liquidColor: 16262179, beneficial: true },
        24 => PotionMeta { statusIconIndex: 20, liquidColor: 9740385, beneficial: false },
        25 => PotionMeta { statusIconIndex: 19, liquidColor: 13565951, beneficial: false },
        26 => PotionMeta { statusIconIndex: 21, liquidColor: 3381504, beneficial: true },
        27 => PotionMeta { statusIconIndex: 22, liquidColor: 12624973, beneficial: false },
        _ => return None,
    };
    Some(meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_icon_indices_match_potion_java_registrations() {
        assert_eq!(potion_meta(1).unwrap().statusIconIndex, 0);
        assert_eq!(potion_meta(10).unwrap().statusIconIndex, 7);
        assert_eq!(potion_meta(13).unwrap().statusIconIndex, 16);
        assert_eq!(potion_meta(21).unwrap().statusIconIndex, 23);
        assert_eq!(potion_meta(8).unwrap().statusIconIndex, 10);
    }

    #[test]
    fn iconless_and_beneficial_flags_match_registrations() {
        assert!(!potion_meta(6).unwrap().hasStatusIcon());
        assert!(!potion_meta(23).unwrap().hasStatusIcon());
        assert!(potion_meta(7).unwrap().beneficial);
        assert!(!potion_meta(2).unwrap().beneficial);
        assert!(!potion_meta(24).unwrap().beneficial);
        assert!(potion_meta(26).unwrap().beneficial);
        assert!(potion_meta(255).is_none());
    }
}
