use crate::compat::Java::JavaRandom;

/// MCP 1.12.2 `MapGenBase` shared seed/range substrate.
#[derive(Debug,Clone)]
pub struct MapGenBase{pub range:i32,pub rand:JavaRandom}
impl Default for MapGenBase{fn default()->Self{Self{range:8,rand:JavaRandom::new(0)}}}
impl MapGenBase{
    pub fn new()->Self{Self::default()}
    pub fn func_191068_a(seed:i64,rand:&mut JavaRandom,chunk_x:i32,chunk_z:i32){
        rand.set_seed(seed);let i=rand.next_i64();let j=rand.next_i64();
        let k=(chunk_x as i64).wrapping_mul(i);let l=(chunk_z as i64).wrapping_mul(j);
        rand.set_seed(k^l^seed);
    }
}
