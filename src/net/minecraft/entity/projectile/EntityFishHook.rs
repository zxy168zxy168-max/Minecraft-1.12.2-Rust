use crate::net::minecraft::entity::projectile::ProjectileHelper::ProjectileHelper;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FishHookState {
    Flying,
    HookedInEntity,
    Bobbing,
}

/// Client-owned constants and source equations from MCP 1.12.2 `EntityFishHook`.
pub struct EntityFishHook;

impl EntityFishHook {
    pub const WIDTH: f32 = 0.25;
    pub const HEIGHT: f32 = 0.25;
    pub const DATA_HOOKED_ENTITY_INDEX: u8 = 6;
    pub const RENDER_DISTANCE_SQ: f64 = 4096.0;
    pub const MAX_GROUND_TICKS: i32 = 1200;
    pub const GRAVITY: f64 = 0.03;
    pub const DRAG: f64 = 0.92;
    pub const WATER_FLYING_XZ_FACTOR: f64 = 0.3;
    pub const WATER_FLYING_Y_FACTOR: f64 = 0.2;
    pub const BOBBING_XZ_FACTOR: f64 = 0.9;
    pub const ROTATION_INTERPOLATION: f32 = 0.2;

    pub const fn hookedEntityId(dataValue: i32) -> Option<i32> {
        if dataValue > 0 {
            Some(dataValue - 1)
        } else {
            None
        }
    }

    pub const fn isInRangeToRenderDist(distanceSquared: f64) -> bool {
        distanceSquared < Self::RENDER_DISTANCE_SQ
    }

    pub fn rotateTowardsMovement(
        prevYaw: &mut f32,
        prevPitch: &mut f32,
        yaw: &mut f32,
        pitch: &mut f32,
        motion: [f64; 3],
    ) {
        let (nextYaw, nextPitch) = ProjectileHelper::rotateTowardsMovement(
            motion[0],
            motion[1],
            motion[2],
            prevYaw,
            prevPitch,
            Self::ROTATION_INTERPOLATION,
        );
        *yaw = nextYaw;
        *pitch = nextPitch;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_contract() {
        assert_eq!(
            (EntityFishHook::WIDTH, EntityFishHook::HEIGHT),
            (0.25, 0.25)
        );
        assert_eq!(EntityFishHook::hookedEntityId(0), None);
        assert_eq!(EntityFishHook::hookedEntityId(42), Some(41));
        assert!(EntityFishHook::isInRangeToRenderDist(4095.999));
        assert!(!EntityFishHook::isInRangeToRenderDist(4096.0));
    }
}
