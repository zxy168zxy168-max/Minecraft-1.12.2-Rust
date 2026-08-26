/// MCP 1.12.2 `MapColor` palette and shade calculation used by filled maps.
pub struct MapColor;

impl MapColor {
    /// `MapColor.COLORS[0..=51]` in registration order. Entries 52..63 are
    /// null in vanilla 1.12.2 and are not emitted by an unmodified server.
    pub const COLOR_VALUES: [u32; 52] = [
        0, 8_368_696, 16_247_203, 13_092_807, 16_711_680, 10_526_975, 10_987_431, 31_744,
        16_777_215, 10_791_096, 9_923_917, 7_368_816, 4_210_943, 9_402_184, 16_776_437, 14_188_339,
        11_685_080, 6_724_056, 15_066_419, 8_375_321, 15_892_389, 5_000_268, 10_066_329, 5_013_401,
        8_339_378, 3_361_970, 6_704_179, 6_717_235, 10_040_115, 1_644_825, 16_445_005, 6_085_589,
        4_882_687, 55_610, 8_476_209, 7_340_544, 13_742_497, 10_441_252, 9_787_244, 7_367_818,
        12_223_780, 6_780_213, 10_505_550, 3_746_083, 8_874_850, 5_725_276, 8_014_168, 4_996_700,
        4_993_571, 5_001_770, 9_321_518, 2_430_480,
    ];

    /// Exact `MapColor#getMapColor` ARGB result.
    pub fn getMapColor(colorIndex: usize, shade: u8) -> u32 {
        let colorValue = Self::COLOR_VALUES.get(colorIndex).copied().unwrap_or(0);
        let brightness = match shade & 3 {
            0 => 180_u32,
            1 => 220_u32,
            2 => 255_u32,
            _ => 135_u32,
        };
        let red = ((colorValue >> 16) & 255) * brightness / 255;
        let green = ((colorValue >> 8) & 255) * brightness / 255;
        let blue = (colorValue & 255) * brightness / 255;
        0xFF00_0000 | red << 16 | green << 8 | blue
    }

    pub fn argbToRgba(color: u32) -> [f32; 4] {
        [
            ((color >> 16) & 255) as f32 / 255.0,
            ((color >> 8) & 255) as f32 / 255.0,
            (color & 255) as f32 / 255.0,
            ((color >> 24) & 255) as f32 / 255.0,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_shade_multipliers_match_map_color() {
        assert_eq!(MapColor::getMapColor(8, 2), 0xFFFF_FFFF);
        assert_eq!(MapColor::getMapColor(8, 0), 0xFFB4_B4B4);
        assert_eq!(MapColor::getMapColor(8, 3), 0xFF87_8787);
    }
}
