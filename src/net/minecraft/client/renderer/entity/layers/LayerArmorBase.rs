use crate::net::minecraft::item::ItemArmor::{ArmorMaterial, ItemArmor};
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub const ENCHANTED_ITEM_GLINT_RES: &str = "textures/misc/enchanted_item_glint.png";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmorTintPass {
    pub color: [f32; 4],
    pub overlay: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlintPass {
    pub textureScale: f32,
    pub translation: f32,
    pub rotationDegrees: f32,
    pub color: [f32; 4],
}

/// Backend-neutral projection of MCP 1.12.2 `LayerArmorBase`.
pub struct LayerArmorBase;

impl LayerArmorBase {
    pub fn texture(stack: &ItemStack, overlay: bool) -> Option<ResourceLocation> {
        ItemArmor::texture(stack, overlay)
    }

    pub fn tintPasses(stack: &ItemStack) -> Vec<ArmorTintPass> {
        let Some(definition) = ItemArmor::definition(stack.itemId) else {
            return Vec::new();
        };
        if definition.material == ArmorMaterial::Leather {
            let color = ItemArmor::getColor(stack);
            vec![
                ArmorTintPass {
                    color: [
                        ((color >> 16) & 255) as f32 / 255.0,
                        ((color >> 8) & 255) as f32 / 255.0,
                        (color & 255) as f32 / 255.0,
                        1.0,
                    ],
                    overlay: false,
                },
                ArmorTintPass {
                    color: [1.0; 4],
                    overlay: true,
                },
            ]
        } else {
            vec![ArmorTintPass {
                color: [1.0; 4],
                overlay: false,
            }]
        }
    }

    /// Exact two texture-matrix passes from
    /// `LayerArmorBase.renderEnchantedGlint` at the supplied entity age.
    pub fn glintPasses(ageInTicks: f32) -> [GlintPass; 2] {
        // MCP: f * (0.001F + i * 0.003F) * 20.0F. The texture
        // sampler repeats, so retaining the unwrapped translation mirrors the
        // OpenGL texture matrix exactly.
        let firstTranslation = ageInTicks * 0.02;
        let secondTranslation = ageInTicks * 0.08;
        [
            GlintPass {
                textureScale: 0.33333334,
                translation: firstTranslation,
                rotationDegrees: 30.0,
                color: [0.38, 0.19, 0.608, 1.0],
            },
            GlintPass {
                textureScale: 0.33333334,
                translation: secondTranslation,
                rotationDegrees: -30.0,
                color: [0.38, 0.19, 0.608, 1.0],
            },
        ]
    }

    pub fn glintTexture() -> ResourceLocation {
        ResourceLocation::new("minecraft", ENCHANTED_ITEM_GLINT_RES)
    }

    pub const fn shouldCombineTextures() -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leather_uses_dyed_base_and_white_overlay() {
        let stack = ItemStack {
            itemId: 299,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        };
        let passes = LayerArmorBase::tintPasses(&stack);
        assert_eq!(passes.len(), 2);
        assert!(!passes[0].overlay);
        assert!(passes[1].overlay);
    }

    #[test]
    fn enchanted_glint_has_opposed_thirty_degree_passes() {
        let passes = LayerArmorBase::glintPasses(20.0);
        assert_eq!(passes[0].rotationDegrees, 30.0);
        assert_eq!(passes[1].rotationDegrees, -30.0);
        assert_eq!(passes[0].color, [0.38, 0.19, 0.608, 1.0]);
        assert!((passes[0].translation - 0.4).abs() < 1.0e-6);
        assert!((passes[1].translation - 1.6).abs() < 1.0e-6);
    }
}
