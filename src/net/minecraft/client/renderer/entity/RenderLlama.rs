use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderLlama;
impl RenderLlama {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "llama"
    }
    pub fn texture(entity: &EntityOtherClient) -> ResourceLocation {
        let name = match entity.llamaVariant() {
            1 => "white",
            2 => "brown",
            3 => "gray",
            _ => "creamy",
        };
        ResourceLocation::new(
            "minecraft",
            format!("textures/entity/llama/llama_{name}.png"),
        )
    }
    pub fn allTextures() -> Vec<ResourceLocation> {
        ["creamy", "white", "brown", "gray"]
            .into_iter()
            .map(|name| {
                ResourceLocation::new(
                    "minecraft",
                    format!("textures/entity/llama/llama_{name}.png"),
                )
            })
            .collect()
    }
}
