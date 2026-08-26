use std::io;

use crate::net::minecraft::world::chunk::Chunk::Chunk;
use crate::net::minecraft::world::WorldServer::WorldServer;

/// MCP 1.12.2 `IChunkLoader`.
///
/// Java's mutable `World` object is expressed as `&mut WorldServer` at the
/// server boundary currently available in Rust. Method responsibilities and
/// call timing are unchanged.
pub trait IChunkLoader {
    fn loadChunk(&self, worldIn: &mut WorldServer, x: i32, z: i32) -> io::Result<Option<Chunk>>;
    fn saveChunk(&self, worldIn: &mut WorldServer, chunkIn: &mut Chunk) -> io::Result<()>;
    fn saveExtraChunkData(&self, worldIn: &mut WorldServer, chunkIn: &Chunk) -> io::Result<()>;
    fn chunkTick(&self);
    fn saveExtraData(&self);
    fn func_191063_a(&self, x: i32, z: i32) -> io::Result<bool>;
}
