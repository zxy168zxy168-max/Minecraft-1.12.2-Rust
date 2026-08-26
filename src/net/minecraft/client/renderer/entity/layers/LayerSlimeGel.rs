/// MCP 1.12.2 `LayerSlimeGel` color/alpha owner. Geometry is the outer
/// `ModelSlime(0)` cube and is submitted by the shared Vulkan living pass.
pub struct LayerSlimeGel;

impl LayerSlimeGel {
    pub const fn color() -> [f32; 4] {
        [1.0, 1.0, 1.0, 0.5]
    }
}
