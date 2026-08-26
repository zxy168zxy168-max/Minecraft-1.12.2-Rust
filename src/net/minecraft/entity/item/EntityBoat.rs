/// MCP 1.12.2 `EntityBoat.Type` in declaration/metadata order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoatType {
    Oak,
    Spruce,
    Birch,
    Jungle,
    Acacia,
    DarkOak,
}

impl BoatType {
    pub const ALL: [Self; 6] = [
        Self::Oak,
        Self::Spruce,
        Self::Birch,
        Self::Jungle,
        Self::Acacia,
        Self::DarkOak,
    ];

    /// MCP `EntityBoat.Type#byId`: invalid synchronized values fall back to OAK.
    pub const fn byId(id: i32) -> Self {
        match id {
            1 => Self::Spruce,
            2 => Self::Birch,
            3 => Self::Jungle,
            4 => Self::Acacia,
            5 => Self::DarkOak,
            _ => Self::Oak,
        }
    }

    pub const fn ordinal(self) -> usize {
        match self {
            Self::Oak => 0,
            Self::Spruce => 1,
            Self::Birch => 2,
            Self::Jungle => 3,
            Self::Acacia => 4,
            Self::DarkOak => 5,
        }
    }
}

/// MCP 1.12.2 `EntityBoat.Status` declaration order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoatStatus {
    InWater,
    UnderWater,
    UnderFlowingWater,
    OnLand,
    InAir,
}

/// Constants and pure client-side portions of MCP 1.12.2 `EntityBoat`.
pub struct EntityBoat;

impl EntityBoat {
    pub const WIDTH: f32 = 1.375;
    pub const HEIGHT: f32 = 0.5625;
    pub const LERP_STEPS: i32 = 10;
    pub const PADDLE_STEP: f32 = 0.39269909262657166_f64 as f32;

    pub fn rowingTime(active: bool, previous: f32, current: f32, partialTicks: f32) -> f32 {
        if active {
            previous + (current - previous) * partialTicks.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_boat_type_is_oak() {
        assert_eq!(BoatType::byId(-1), BoatType::Oak);
        assert_eq!(BoatType::byId(99), BoatType::Oak);
    }

    #[test]
    fn paddle_step_is_pi_over_eight() {
        assert!((EntityBoat::PADDLE_STEP - std::f32::consts::FRAC_PI_4 * 0.5).abs() < 1.0e-6);
    }
}
