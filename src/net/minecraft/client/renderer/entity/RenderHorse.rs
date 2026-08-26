use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HorseLayeredTexture {
    pub generated: ResourceLocation,
    pub layers: Vec<ResourceLocation>,
}

pub struct RenderHorse;

impl RenderHorse {
    const COATS: [&'static str; 7] = [
        "white",
        "creamy",
        "chestnut",
        "brown",
        "black",
        "gray",
        "darkbrown",
    ];
    const MARKINGS: [Option<&'static str>; 5] = [
        None,
        Some("white"),
        Some("whitefield"),
        Some("whitedots"),
        Some("blackdots"),
    ];
    const ARMOR: [Option<&'static str>; 4] = [None, Some("iron"), Some("gold"), Some("diamond")];

    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "horse"
    }

    pub fn coatIndex(entity: &EntityOtherClient) -> usize {
        ((entity.horseVariant() & 255).rem_euclid(7)) as usize
    }

    pub fn markingIndex(entity: &EntityOtherClient) -> usize {
        (((entity.horseVariant() & 65280) >> 8).rem_euclid(5)) as usize
    }

    pub fn texture(entity: &EntityOtherClient) -> ResourceLocation {
        Self::generatedTexture(
            Self::coatIndex(entity),
            Self::markingIndex(entity),
            entity.horseArmorOrdinal() as usize,
        )
    }

    pub fn generatedTexture(coat: usize, marking: usize, armor: usize) -> ResourceLocation {
        ResourceLocation::new(
            "minecraft",
            format!(
                "generated/entity/horse/{}_{}_{}.png",
                Self::COATS[coat.min(6)],
                Self::MARKINGS[marking.min(4)].unwrap_or("none"),
                Self::ARMOR[armor.min(3)].unwrap_or("none"),
            ),
        )
    }

    pub fn layeredTextures() -> Vec<HorseLayeredTexture> {
        let mut result = Vec::with_capacity(7 * 5 * 4);
        for coat in 0..7 {
            for marking in 0..5 {
                for armor in 0..4 {
                    // Vulkan's material compositor is front-to-back. MCP
                    // LayeredTexture draws armor and markings over the opaque
                    // coat, so register topmost layers first and the coat last.
                    let mut layers = Vec::with_capacity(3);
                    if let Some(armor) = Self::ARMOR[armor] {
                        layers.push(ResourceLocation::new(
                            "minecraft",
                            format!("textures/entity/horse/armor/horse_armor_{armor}.png"),
                        ));
                    }
                    if let Some(marking) = Self::MARKINGS[marking] {
                        layers.push(ResourceLocation::new(
                            "minecraft",
                            format!("textures/entity/horse/horse_markings_{marking}.png"),
                        ));
                    }
                    layers.push(ResourceLocation::new(
                        "minecraft",
                        format!("textures/entity/horse/horse_{}.png", Self::COATS[coat]),
                    ));
                    result.push(HorseLayeredTexture {
                        generated: Self::generatedTexture(coat, marking, armor),
                        layers,
                    });
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_vanilla_coat_marking_armor_combinations_are_registered() {
        let textures = RenderHorse::layeredTextures();
        assert_eq!(textures.len(), 140);
        assert!(textures.iter().any(|entry| entry.layers.len() == 3));
    }
}
