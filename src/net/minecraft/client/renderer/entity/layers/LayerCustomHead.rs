use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::net::minecraft::item::ItemSkull::ItemSkull;
use crate::net::minecraft::item::ItemStack::ItemStack;

/// MCP 1.12.2 `LayerCustomHead` responsibility used by the Vulkan player
/// renderer. Matrix application remains in the renderer backend, while the
/// item/profile decisions stay in the named layer class.
pub struct LayerCustomHead;

impl LayerCustomHead {
    pub const SKULL_SCALE: f32 = 1.1875;

    pub fn isSkull(stack: &ItemStack) -> bool {
        ItemSkull::isItemSkull(stack)
    }

    pub fn playerProfile(stack: &ItemStack) -> Option<GameProfile> {
        ItemSkull::getPlayerProfile(stack)
    }

    pub const fn shouldCombineTextures() -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skull_layer_uses_mcp_scale_and_never_combines_hurt_tint() {
        assert!((LayerCustomHead::SKULL_SCALE - 1.1875).abs() < f32::EPSILON);
        assert!(!LayerCustomHead::shouldCombineTextures());
    }
}
