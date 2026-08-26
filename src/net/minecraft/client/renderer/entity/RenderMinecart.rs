use crate::net::minecraft::client::entity::EntityOtherClient::EntityOtherClient;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `RenderMinecart` constants and deterministic render jitter.
pub struct RenderMinecart;

impl RenderMinecart {
    pub const SHADOW_SIZE: f32 = 0.5;
    pub const Y_TRANSLATION: f32 = 0.375;
    pub const CONTENT_SCALE: f32 = 0.75;

    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/minecart.png")
    }

    /// EntityMinecart#getRenderBoundingBox followed by Render#shouldRender's
    /// ordinary half-block expansion.
    pub fn renderBoundingBox(entity: &EntityOtherClient) -> AxisAlignedBB {
        let content = if entity.minecartHasDisplayTile() {
            entity.minecartDisplayOffset().abs() as f64 / 16.0
        } else {
            0.0
        };
        entity
            .entity
            .boundingBox
            .expand_xyz(content)
            .expand_xyz(0.5)
    }

    pub fn deterministicOffset(entityId: i32) -> [f32; 3] {
        let mut value = (entityId as i64).wrapping_mul(493_286_711);
        value = value
            .wrapping_mul(value)
            .wrapping_mul(4_392_167_121)
            .wrapping_add(value.wrapping_mul(98_761));
        [16, 20, 24].map(|shift| ((((value >> shift) & 7) as f32 + 0.5) / 8.0 - 0.5) * 0.004)
    }

    pub fn damageRotation(
        rollingAmplitude: i32,
        damage: f32,
        rollingDirection: i32,
        partialTicks: f32,
    ) -> f32 {
        let f = rollingAmplitude as f32 - partialTicks;
        let f1 = (damage - partialTicks).max(0.0);
        if f > 0.0 {
            f.sin() * f * f1 / 10.0 * rollingDirection as f32
        } else {
            0.0
        }
    }

    /// `RenderTntMinecart#renderCartContents` pre-explosion fourth-power pulse.
    pub fn tntContentScale(fuseTicks: i32, partialTicks: f32) -> f32 {
        let remaining = fuseTicks as f32 - partialTicks + 1.0;
        if fuseTicks > -1 && remaining < 10.0 {
            let mut progress = (1.0 - remaining / 10.0).clamp(0.0, 1.0);
            progress *= progress;
            progress *= progress;
            1.0 + progress * 0.3
        } else {
            1.0
        }
    }

    /// Alpha of the texture-disabled white flash pass, or `None` on the five-
    /// tick intervals where MCP does not submit the second TNT model.
    pub fn tntFlashAlpha(fuseTicks: i32, partialTicks: f32) -> Option<f32> {
        if fuseTicks > -1 && fuseTicks / 5 % 2 == 0 {
            Some((1.0 - (fuseTicks as f32 - partialTicks + 1.0) / 100.0) * 0.8)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jitter_is_bounded_by_vanilla_range() {
        for v in RenderMinecart::deterministicOffset(42) {
            assert!((-0.002..=0.002).contains(&v));
        }
    }

    #[test]
    fn tnt_pulse_and_flash_follow_render_tnt_minecart() {
        assert_eq!(RenderMinecart::tntContentScale(10, 0.0), 1.0);
        assert!(RenderMinecart::tntContentScale(0, 0.0) > 1.19);
        assert!(RenderMinecart::tntFlashAlpha(10, 0.0).is_some());
        assert!(RenderMinecart::tntFlashAlpha(5, 0.0).is_none());
    }
}
