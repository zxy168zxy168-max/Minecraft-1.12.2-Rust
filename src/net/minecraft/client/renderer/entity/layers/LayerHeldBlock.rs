/// MCP 1.12.2 `LayerHeldBlock` fixed transforms for the Enderman carried block.
pub struct LayerHeldBlock;
impl LayerHeldBlock {
    pub const TRANSLATION_1: [f32; 3] = [0.0, 0.6875, -0.75];
    pub const ROTATION_X: f32 = 20.0;
    pub const ROTATION_Y: f32 = 45.0;
    pub const TRANSLATION_2: [f32; 3] = [0.25, 0.1875, 0.25];
    pub const SCALE: [f32; 3] = [-0.5, -0.5, 0.5];
}
