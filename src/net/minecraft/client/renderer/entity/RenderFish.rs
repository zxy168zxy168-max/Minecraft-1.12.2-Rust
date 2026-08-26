use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// Exact immutable geometry constants from MCP 1.12.2 `RenderFish`.
pub struct RenderFish;

impl RenderFish {
    pub const SCALE: f32 = 0.5;
    pub const HOOK_Y_OFFSET: f64 = 0.25;
    pub const LINE_SEGMENTS: usize = 16;
    pub const HOOK_UV: [[f32; 2]; 4] = [
        [0.0625, 0.1875],
        [0.125, 0.1875],
        [0.125, 0.125],
        [0.0625, 0.125],
    ];
    pub const HOOK_POSITIONS: [[f32; 3]; 4] = [
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ];

    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/particle/particles.png")
    }

    pub const fn linePoint(start: [f64; 3], delta: [f64; 3], step: usize) -> [f64; 3] {
        let f = step as f64 / Self::LINE_SEGMENTS as f64;
        [
            start[0] + delta[0] * f,
            start[1] + delta[1] * (f * f + f) * 0.5,
            start[2] + delta[2] * f,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hook_tile_and_parabolic_line_match_source() {
        assert_eq!(RenderFish::LINE_SEGMENTS, 16);
        assert_eq!(RenderFish::HOOK_UV[0], [1.0 / 16.0, 3.0 / 16.0]);
        let midpoint = RenderFish::linePoint([0.0; 3], [2.0, 2.0, 2.0], 8);
        assert_eq!(midpoint, [1.0, 0.75, 1.0]);
    }
}
