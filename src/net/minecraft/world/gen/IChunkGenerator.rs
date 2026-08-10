use crate::net::minecraft::world::chunk::Chunk::Chunk;

/// Rust trait boundary for MCP 1.12.2 `IChunkGenerator`.
///
/// The complete interface is migrated as the corresponding creature/structure
/// types arrive. Terrain provision, population and structure recreation remain
/// generator-owned from the first server implementation rather than being
/// embedded in `ChunkProviderServer`.
pub trait IChunkGenerator: Send {
    fn provideChunk(&mut self, x: i32, z: i32) -> Result<Chunk, String>;
    fn populate(&mut self, _x: i32, _z: i32) -> Result<(), String> { Ok(()) }
    fn generateStructures(&mut self, _chunk: &mut Chunk, _x: i32, _z: i32) -> bool { false }
    fn recreateStructures(&mut self, _chunk: &mut Chunk, _x: i32, _z: i32) {}
    fn generatorName(&self) -> &'static str;
    fn seaLevelOverride(&self) -> Option<i32> { None }
}
