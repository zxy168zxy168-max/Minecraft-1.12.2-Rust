use crate::compat::Java::JavaRandom;
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;

const NAME_PARTS: [&str; 61] = [
    "the",
    "elder",
    "scrolls",
    "klaatu",
    "berata",
    "niktu",
    "xyzzy",
    "bless",
    "curse",
    "light",
    "darkness",
    "fire",
    "air",
    "earth",
    "water",
    "hot",
    "dry",
    "cold",
    "wet",
    "ignite",
    "snuff",
    "embiggen",
    "twist",
    "shorten",
    "stretch",
    "fiddle",
    "destroy",
    "imbue",
    "galvanize",
    "enchant",
    "free",
    "limited",
    "range",
    "of",
    "towards",
    "inside",
    "sphere",
    "cube",
    "self",
    "other",
    "ball",
    "mental",
    "physical",
    "grow",
    "shrink",
    "demon",
    "elemental",
    "spirit",
    "animal",
    "creature",
    "beast",
    "humanoid",
    "undead",
    "fresh",
    "stale",
    "phnglui",
    "mglwnafh",
    "cthulhu",
    "rlyeh",
    "wgahnagl",
    "fhtagnbaguette",
];

/// MCP 1.12.2 `EnchantmentNameParts`. The singleton's mutable Java RNG is
/// represented directly; callers reseed it from `ContainerEnchantment.xpSeed`
/// before generating the three option labels.
#[derive(Debug, Clone, PartialEq)]
pub struct EnchantmentNameParts {
    rand: JavaRandom,
}

impl Default for EnchantmentNameParts {
    fn default() -> Self {
        Self {
            rand: JavaRandom::new(0),
        }
    }
}

impl EnchantmentNameParts {
    pub fn reseedRandomGenerator(&mut self, seed: i64) {
        self.rand.set_seed(seed);
    }

    pub fn generateNewRandomName(&mut self, fontRenderer: &FontRenderer, width: i32) -> String {
        let count = self.rand.next_i32_bound(2) + 3;
        let mut words = Vec::with_capacity(count as usize);
        for _ in 0..count {
            words.push(NAME_PARTS[self.rand.next_i32_bound(NAME_PARTS.len() as i32) as usize]);
        }
        let sentence = words.join(" ");
        let lines = fontRenderer.list_formatted_string_to_width(&sentence, width);
        lines.into_iter().take(2).collect::<Vec<_>>().join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_random_sequence_matches_mcp_name_parts() {
        let font = FontRenderer::test_metric_renderer();
        let mut names = EnchantmentNameParts::default();
        names.reseedRandomGenerator(12345);
        assert_eq!(
            names.generateNewRandomName(&font, 1000),
            "wgahnagl fresh xyzzy"
        );
        assert_eq!(
            names.generateNewRandomName(&font, 1000),
            "mental darkness stretch creature"
        );
        assert_eq!(names.generateNewRandomName(&font, 1000), "stale ball water");
    }
}
