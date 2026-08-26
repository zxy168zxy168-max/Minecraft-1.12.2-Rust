use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockBed::BlockBed;
use crate::net::minecraft::client::renderer::tileentity::TileEntityItemStackRenderer::{
    BuiltInItemMesh, TileEntityItemStackRenderer,
};
use crate::net::minecraft::tileentity::TileEntityBed::TileEntityBed;

/// CPU/Vulkan equivalent of MCP 1.12.2 `TileEntityBedRenderer`.
pub struct TileEntityBedRenderer;

impl TileEntityBedRenderer {
    pub fn buildMesh(tile: &TileEntityBed, state: IBlockState) -> Option<BuiltInItemMesh> {
        if !BlockBed::isBlockBed(state) {
            return None;
        }
        Some(TileEntityItemStackRenderer::buildWorldBedHalf(
            tile.colorMetadata() as i16,
            BlockBed::isHead(state),
            BlockBed::getFacing(state).horizontalIndex().unwrap_or(2) as i32,
        ))
    }
}
