use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::client::renderer::entity::Render::RenderProperties;

pub struct RenderTNTPrimed;

impl RenderTNTPrimed {
    pub const PROPERTIES: RenderProperties = RenderProperties::new(0.5, 1.0);
    pub const TNT_STATE: IBlockState = IBlockState::fromGlobalStateId(46 << 4);

    pub fn scale(fuse: i32, partialTicks: f32) -> f32 {
        if fuse as f32 - partialTicks + 1.0 >= 10.0 {
            return 1.0;
        }
        let mut value = 1.0 - (fuse as f32 - partialTicks + 1.0) / 10.0;
        value = value.clamp(0.0, 1.0);
        value *= value;
        value *= value;
        1.0 + value * 0.3
    }

    pub fn flashAlpha(fuse: i32, partialTicks: f32) -> f32 {
        (1.0 - (fuse as f32 - partialTicks + 1.0) / 100.0) * 0.8
    }

    pub const fn shouldFlash(fuse: i32) -> bool {
        fuse / 5 % 2 == 0
    }
}
