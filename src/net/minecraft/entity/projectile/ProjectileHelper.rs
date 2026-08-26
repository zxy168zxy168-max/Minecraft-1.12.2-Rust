/// Renderer-facing subset of MCP 1.12.2 `ProjectileHelper`.
pub struct ProjectileHelper;

impl ProjectileHelper {
    /// Exact `ProjectileHelper#rotateTowardsMovement` angle construction,
    /// previous-angle normalization and interpolation.
    pub fn rotateTowardsMovement(
        motionX: f64,
        motionY: f64,
        motionZ: f64,
        previousYaw: &mut f32,
        previousPitch: &mut f32,
        interpolation: f32,
    ) -> (f32, f32) {
        let horizontal = (motionX * motionX + motionZ * motionZ).sqrt();
        let targetYaw = motionZ.atan2(motionX).to_degrees() as f32 + 90.0;
        let targetPitch = horizontal.atan2(motionY).to_degrees() as f32 - 90.0;

        while targetPitch - *previousPitch < -180.0 {
            *previousPitch -= 360.0;
        }
        while targetPitch - *previousPitch >= 180.0 {
            *previousPitch += 360.0;
        }
        while targetYaw - *previousYaw < -180.0 {
            *previousYaw -= 360.0;
        }
        while targetYaw - *previousYaw >= 180.0 {
            *previousYaw += 360.0;
        }

        (
            *previousYaw + (targetYaw - *previousYaw) * interpolation,
            *previousPitch + (targetPitch - *previousPitch) * interpolation,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_x_motion_turns_halfway_to_source_ninety_degree_yaw() {
        let mut previousYaw = 0.0;
        let mut previousPitch = 0.0;
        let (yaw, pitch) = ProjectileHelper::rotateTowardsMovement(
            1.0,
            0.0,
            0.0,
            &mut previousYaw,
            &mut previousPitch,
            0.5,
        );
        assert!((yaw - 45.0).abs() < 1.0e-5);
        assert!(pitch.abs() < 1.0e-5);
    }

    #[test]
    fn previous_angles_are_normalized_across_the_shortest_arc() {
        let mut previousYaw = 350.0;
        let mut previousPitch = 0.0;
        let (yaw, _) = ProjectileHelper::rotateTowardsMovement(
            0.0,
            0.0,
            -1.0,
            &mut previousYaw,
            &mut previousPitch,
            0.5,
        );
        assert!((yaw - 360.0).abs() < 1.0e-5);
    }
}
