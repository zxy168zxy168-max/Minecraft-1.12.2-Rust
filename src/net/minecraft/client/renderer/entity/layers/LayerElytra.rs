use crate::net::minecraft::client::network::NetworkPlayerInfo::NetworkPlayerInfo;
use crate::net::minecraft::entity::player::EnumPlayerModelParts::EnumPlayerModelParts;
use crate::net::minecraft::item::ItemArmor::ItemArmor;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct LayerElytra;

impl LayerElytra {
    pub fn defaultTexture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/elytra.png")
    }

    /// Texture precedence from MCP `LayerElytra#doRenderLayer`: custom ELYTRA,
    /// cape texture when the profile marks it usable for elytra and the cape
    /// model part is enabled, then the vanilla entity texture.
    pub fn texture(
        chestStack: &ItemStack,
        playerInfo: Option<&NetworkPlayerInfo>,
        skinParts: u8,
    ) -> Option<ResourceLocation> {
        if !ItemArmor::isElytra(chestStack) {
            return None;
        }
        if let Some(info) = playerInfo {
            if let Some(elytra) = info.getLocationElytra() {
                return Some(elytra);
            }
            // Vanilla `AbstractClientPlayer#hasElytraCape` rejects only an
            // OptiFine-local cape override. This port has no OptiFine cape
            // location, so a profile cape is eligible whenever the CAPE model
            // part is enabled.
            if (skinParts & EnumPlayerModelParts::Cape.getPartMask()) != 0 {
                if let Some(cape) = info.getLocationCape() {
                    return Some(cape);
                }
            }
        }
        Some(Self::defaultTexture())
    }

    pub const fn shouldCombineTextures() -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_elytra_skips_the_layer() {
        assert!(LayerElytra::texture(&ItemStack::EMPTY, None, 0xff).is_none());
    }

    #[test]
    fn equipped_elytra_falls_back_to_vanilla_texture() {
        let stack = ItemStack {
            itemId: 443,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        };
        assert_eq!(
            LayerElytra::texture(&stack, None, 0xff).unwrap().getPath(),
            "textures/entity/elytra.png"
        );
    }
}
