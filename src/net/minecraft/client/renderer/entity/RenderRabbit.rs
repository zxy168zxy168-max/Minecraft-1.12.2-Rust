use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
pub struct RenderRabbit;
impl RenderRabbit {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "rabbit"
    }
    pub fn texture(entity: &EntityOtherClient) -> ResourceLocation {
        let toast = entity.customName().map(strip_formatting).as_deref() == Some("Toast");
        let path = if toast {
            "textures/entity/rabbit/toast.png"
        } else {
            match entity.rabbitType() {
                1 => "textures/entity/rabbit/white.png",
                2 => "textures/entity/rabbit/black.png",
                3 => "textures/entity/rabbit/white_splotched.png",
                4 => "textures/entity/rabbit/gold.png",
                5 => "textures/entity/rabbit/salt.png",
                99 => "textures/entity/rabbit/caerbannog.png",
                _ => "textures/entity/rabbit/brown.png",
            }
        };
        ResourceLocation::new("minecraft", path)
    }
    pub fn allTextures() -> [ResourceLocation; 8] {
        [
            ResourceLocation::new("minecraft", "textures/entity/rabbit/brown.png"),
            ResourceLocation::new("minecraft", "textures/entity/rabbit/white.png"),
            ResourceLocation::new("minecraft", "textures/entity/rabbit/black.png"),
            ResourceLocation::new("minecraft", "textures/entity/rabbit/white_splotched.png"),
            ResourceLocation::new("minecraft", "textures/entity/rabbit/gold.png"),
            ResourceLocation::new("minecraft", "textures/entity/rabbit/salt.png"),
            ResourceLocation::new("minecraft", "textures/entity/rabbit/caerbannog.png"),
            ResourceLocation::new("minecraft", "textures/entity/rabbit/toast.png"),
        ]
    }
}
fn strip_formatting(value: &str) -> String {
    let mut out = String::new();
    let mut skip = false;
    for c in value.chars() {
        if skip {
            skip = false;
            continue;
        }
        if c == '§' {
            skip = true;
        } else {
            out.push(c);
        }
    }
    out
}
