/// Rust semantic owner for the MCP 1.12.2 `AbstractHorse` constants and
/// calculation-only portions used by the heterogeneous client entity store.
///
/// Java inheritance is represented by `EntityOtherClient` retaining the
/// concrete synchronized fields while delegating the source formulas here.
pub struct AbstractHorse;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JumpPowerUpdate {
    pub jumpPower: f32,
    pub allowStandSliding: bool,
    pub rear: bool,
}

impl AbstractHorse {
    pub const STATUS_TAME: u8 = 2;
    pub const STATUS_SADDLED: u8 = 4;
    pub const STATUS_EATING_HAYSTACK: u8 = 16;
    pub const STATUS_REARING: u8 = 32;
    pub const STATUS_MOUTH_OPEN: u8 = 64;

    pub const DEFAULT_MAX_HEALTH: f64 = 53.0;
    pub const DEFAULT_MOVEMENT_SPEED: f64 = 0.22499999403953552;
    pub const DEFAULT_JUMP_STRENGTH: f64 = 0.7;

    pub const RIDER_PITCH_SCALE: f32 = 0.5;
    pub const RIDER_STRAFE_SCALE: f32 = 0.5;
    pub const REVERSE_SPEED_SCALE: f32 = 0.25;
    pub const FORWARD_JUMP_IMPULSE: f32 = 0.4;
    pub const REARING_CLEAR_TICK: i32 = 20;

    pub fn canBeSteered(registryName: &str) -> bool {
        matches!(
            registryName,
            "horse" | "donkey" | "mule" | "skeleton_horse" | "zombie_horse"
        )
    }

    /// Calculation half of MCP `AbstractHorse#setJumpPower`.
    pub fn jumpPowerUpdate(saddled: bool, mut jumpPowerIn: i32) -> Option<JumpPowerUpdate> {
        if !saddled {
            return None;
        }
        let rear = jumpPowerIn >= 0;
        if jumpPowerIn < 0 {
            jumpPowerIn = 0;
        }
        let jumpPower = if jumpPowerIn >= 90 {
            1.0
        } else {
            0.4 + 0.4 * jumpPowerIn as f32 / 90.0
        };
        Some(JumpPowerUpdate {
            jumpPower,
            allowStandSliding: rear,
            rear,
        })
    }

    /// Horizontal part of the mounted jump impulse in
    /// `AbstractHorse#func_191986_a`.
    pub fn forwardJumpImpulse(rotationYaw: f32, jumpPower: f32, forward: f32) -> [f64; 2] {
        if forward <= 0.0 {
            return [0.0, 0.0];
        }
        let yaw = rotationYaw * 0.017453292;
        [
            (-Self::FORWARD_JUMP_IMPULSE * yaw.sin() * jumpPower) as f64,
            (Self::FORWARD_JUMP_IMPULSE * yaw.cos() * jumpPower) as f64,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jump_power_piecewise_values_match_mcp() {
        assert_eq!(AbstractHorse::jumpPowerUpdate(false, 90), None);
        let negative = AbstractHorse::jumpPowerUpdate(true, -1).unwrap();
        assert!((negative.jumpPower - 0.4).abs() < 1.0e-6);
        assert!(!negative.rear);
        let half = AbstractHorse::jumpPowerUpdate(true, 45).unwrap();
        assert!((half.jumpPower - 0.6).abs() < 1.0e-6);
        assert!(half.rear);
        assert_eq!(
            AbstractHorse::jumpPowerUpdate(true, 90).unwrap().jumpPower,
            1.0
        );
    }

    #[test]
    fn llama_is_not_steerable_but_other_abstract_horses_are() {
        assert!(AbstractHorse::canBeSteered("horse"));
        assert!(AbstractHorse::canBeSteered("donkey"));
        assert!(!AbstractHorse::canBeSteered("llama"));
    }
}
