use crate::net::minecraft::client::renderer::tileentity::TileEntityItemStackRenderer::{
    BuiltInItemMesh, TileEntityItemStackRenderer,
};
use crate::net::minecraft::util::EnumFacing::EnumFacing;

/// Vulkan-backed semantic owner for MCP 1.12.2
/// `TileEntityShulkerBoxRenderer`.
pub struct TileEntityShulkerBoxRenderer;

impl TileEntityShulkerBoxRenderer {
    /// Builds the base and animated lid using the source renderer's facing
    /// transform and interpolated open progress.
    pub fn buildMesh(colorMetadata: i32, facing: EnumFacing, progress: f32) -> BuiltInItemMesh {
        TileEntityItemStackRenderer::buildWorldShulker(colorMetadata, facing, progress)
    }
}
