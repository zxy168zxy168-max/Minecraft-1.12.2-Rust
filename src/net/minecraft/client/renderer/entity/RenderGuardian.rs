use crate::net::minecraft::client::entity::EntityOtherClient::{EntityOtherClient, MobEntityType};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardianRenderVariant {
    Guardian,
    ElderGuardian,
}

/// MCP 1.12.2 `RenderGuardian` and `RenderElderGuardian` constants and dispatch.
pub struct RenderGuardian;

impl RenderGuardian {
    pub const ELDER_SCALE: f32 = 2.35;
    pub const PACKED_FULL_BRIGHT: u32 = 15_728_880;

    pub fn variant(entityType: MobEntityType) -> Option<GuardianRenderVariant> {
        match entityType.registryName {
            "guardian" => Some(GuardianRenderVariant::Guardian),
            "elder_guardian" => Some(GuardianRenderVariant::ElderGuardian),
            _ => None,
        }
    }

    pub fn texture(variant: GuardianRenderVariant) -> ResourceLocation {
        ResourceLocation::new(
            "minecraft",
            match variant {
                GuardianRenderVariant::Guardian => "textures/entity/guardian.png",
                GuardianRenderVariant::ElderGuardian => "textures/entity/guardian_elder.png",
            },
        )
    }

    pub fn beamTexture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/guardian_beam.png")
    }

    pub fn allTextures() -> Vec<ResourceLocation> {
        vec![
            Self::texture(GuardianRenderVariant::Guardian),
            Self::texture(GuardianRenderVariant::ElderGuardian),
            Self::beamTexture(),
        ]
    }

    pub const fn preScale(variant: GuardianRenderVariant) -> f32 {
        match variant {
            GuardianRenderVariant::Guardian => 1.0,
            GuardianRenderVariant::ElderGuardian => Self::ELDER_SCALE,
        }
    }

    pub const fn attackDuration(variant: GuardianRenderVariant) -> i32 {
        match variant {
            GuardianRenderVariant::Guardian => 80,
            GuardianRenderVariant::ElderGuardian => 60,
        }
    }

    pub fn attackAnimationScale(entity: &EntityOtherClient, partialTicks: f32) -> f32 {
        let variant = match &entity.kind {
            crate::net::minecraft::client::entity::EntityOtherClient::ClientEntityKind::Mob {
                entityType,
            } => Self::variant(*entityType),
            _ => None,
        };
        let Some(variant) = variant else {
            return 0.0;
        };
        (entity.guardianAttackTime as f32 + partialTicks) / Self::attackDuration(variant) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elder_uses_original_texture_scale_and_shorter_charge() {
        let elder = GuardianRenderVariant::ElderGuardian;
        assert_eq!(RenderGuardian::preScale(elder), 2.35);
        assert_eq!(RenderGuardian::attackDuration(elder), 60);
        assert_eq!(
            RenderGuardian::texture(elder).getPath(),
            "textures/entity/guardian_elder.png"
        );
    }
}
