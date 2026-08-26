use crate::net::minecraft::client::entity::EntityOtherClient::EntityOtherClient;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct LayerLlamaDecor;
impl LayerLlamaDecor {
    const NAMES: [&'static str; 16] = [
        "white",
        "orange",
        "magenta",
        "light_blue",
        "yellow",
        "lime",
        "pink",
        "gray",
        "silver",
        "cyan",
        "purple",
        "blue",
        "brown",
        "green",
        "red",
        "black",
    ];
    pub fn texture(entity: &EntityOtherClient) -> Option<ResourceLocation> {
        entity.llamaDecorColor().map(|color| {
            ResourceLocation::new(
                "minecraft",
                format!(
                    "textures/entity/llama/decor/decor_{}.png",
                    Self::NAMES[color as usize]
                ),
            )
        })
    }
    pub fn allTextures() -> Vec<ResourceLocation> {
        Self::NAMES
            .into_iter()
            .map(|name| {
                ResourceLocation::new(
                    "minecraft",
                    format!("textures/entity/llama/decor/decor_{name}.png"),
                )
            })
            .collect()
    }
    pub const fn modelDelta() -> f32 {
        0.5
    }
}
