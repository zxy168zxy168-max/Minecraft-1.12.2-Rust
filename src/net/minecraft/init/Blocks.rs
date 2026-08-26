use crate::net::minecraft::block::Block::{Block, AIR as AIR_BLOCK};
pub const AIR: Block = AIR_BLOCK;
pub fn getRegisteredBlock(id: i32) -> Block {
    Block::getBlockById(id)
}
