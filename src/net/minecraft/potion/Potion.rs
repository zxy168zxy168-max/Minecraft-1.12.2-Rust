/// HUD-facing subset of MCP 1.12.2 `Potion` registrations
/// (`Potion.java` `REGISTRY.register` calls): the status icon index into
/// `container/inventory.png` and whether the effect is beneficial
/// (`Potion#setBeneficial`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PotionMeta {
    /// `Potion#statusIconIndex`; `-1` means `hasStatusIcon()` is false.
    pub statusIconIndex: i32,
    /// `Potion#isBeneficial()` (the `beneficial` flag; instant-damage is
    /// registered beneficial despite being an instant effect).
    pub beneficial: bool,
}

impl PotionMeta {
    pub const fn hasStatusIcon(&self) -> bool { self.statusIconIndex >= 0 }

    /// Icon rect in the inventory texture: `(index % 8 * 18, 198 + index / 8 * 18, 18, 18)`.
    pub const fn iconRect(&self) -> (i32, i32) {
        (self.statusIconIndex % 8 * 18, 198 + self.statusIconIndex / 8 * 18)
    }
}

/// MCP `Potion#getPotionFromID`, restricted to the ids the game registers.
pub const fn potion_meta(potionId: u8) -> Option<PotionMeta> {
    // (statusIconIndex = col * 8 + row from setIconIndex(row, col)).
    let meta = match potionId {
        1 => PotionMeta { statusIconIndex: 0, beneficial: true },   // speed
        2 => PotionMeta { statusIconIndex: 1, beneficial: false },  // slowness
        3 => PotionMeta { statusIconIndex: 2, beneficial: true },   // haste
        4 => PotionMeta { statusIconIndex: 3, beneficial: false },  // mining fatigue
        5 => PotionMeta { statusIconIndex: 4, beneficial: true },   // strength
        6 => PotionMeta { statusIconIndex: -1, beneficial: true },  // instant health
        7 => PotionMeta { statusIconIndex: -1, beneficial: true },  // instant damage
        8 => PotionMeta { statusIconIndex: 10, beneficial: true },  // jump boost
        9 => PotionMeta { statusIconIndex: 11, beneficial: false }, // nausea
        10 => PotionMeta { statusIconIndex: 7, beneficial: true },  // regeneration
        11 => PotionMeta { statusIconIndex: 14, beneficial: true }, // resistance
        12 => PotionMeta { statusIconIndex: 15, beneficial: true }, // fire resistance
        13 => PotionMeta { statusIconIndex: 16, beneficial: true }, // water breathing
        14 => PotionMeta { statusIconIndex: 8, beneficial: true },  // invisibility
        15 => PotionMeta { statusIconIndex: 13, beneficial: false },// blindness
        16 => PotionMeta { statusIconIndex: 12, beneficial: true }, // night vision
        17 => PotionMeta { statusIconIndex: 9, beneficial: false }, // hunger
        18 => PotionMeta { statusIconIndex: 5, beneficial: false }, // weakness
        19 => PotionMeta { statusIconIndex: 6, beneficial: false }, // poison
        20 => PotionMeta { statusIconIndex: 17, beneficial: false },// wither
        21 => PotionMeta { statusIconIndex: 23, beneficial: true }, // health boost
        22 => PotionMeta { statusIconIndex: 18, beneficial: true }, // absorption
        23 => PotionMeta { statusIconIndex: -1, beneficial: true }, // saturation
        24 => PotionMeta { statusIconIndex: 20, beneficial: false },// glowing
        25 => PotionMeta { statusIconIndex: 19, beneficial: false },// levitation
        26 => PotionMeta { statusIconIndex: 21, beneficial: true }, // luck
        27 => PotionMeta { statusIconIndex: 22, beneficial: false },// unluck
        _ => return None,
    };
    Some(meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_icon_indices_match_potion_java_registrations() {
        assert_eq!(potion_meta(1).unwrap().statusIconIndex, 0);   // speed (0,0)
        assert_eq!(potion_meta(10).unwrap().statusIconIndex, 7);  // regen (7,0)
        assert_eq!(potion_meta(13).unwrap().statusIconIndex, 16); // water breathing (0,2)
        assert_eq!(potion_meta(21).unwrap().statusIconIndex, 23); // health boost (7,2)
        assert_eq!(potion_meta(8).unwrap().statusIconIndex, 10);  // jump boost (2,1)
    }

    #[test]
    fn iconless_and_beneficial_flags_match_registrations() {
        assert!(!potion_meta(6).unwrap().hasStatusIcon());  // instant health
        assert!(!potion_meta(23).unwrap().hasStatusIcon()); // saturation
        assert!(potion_meta(7).unwrap().beneficial);        // instant damage is registered beneficial
        assert!(!potion_meta(2).unwrap().beneficial);
        assert!(!potion_meta(24).unwrap().beneficial);      // glowing
        assert!(potion_meta(26).unwrap().beneficial);       // luck
        assert!(potion_meta(255).is_none());
    }
}
