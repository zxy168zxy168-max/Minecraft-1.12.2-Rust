use crate::net::minecraft::world::DimensionType::DimensionType;

/// MCP 1.12.2 `WorldProviderSurface` specialization.
pub struct WorldProviderSurface;
impl WorldProviderSurface {
    pub const fn getDimensionType() -> DimensionType {
        DimensionType::Overworld
    }
    pub const fn canDropChunk(isSpawnChunk: bool) -> bool {
        !isSpawnChunk
    }
}
