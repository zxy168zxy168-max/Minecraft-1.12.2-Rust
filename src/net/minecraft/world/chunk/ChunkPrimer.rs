use crate::net::minecraft::block::state::IBlockState::IBlockState;

/// MCP 1.12.2 `ChunkPrimer` compact generation buffer.
#[derive(Debug, Clone)]
pub struct ChunkPrimer { data: Vec<i32> }

impl ChunkPrimer {
    pub fn new() -> Self { Self { data: vec![0; 65_536] } }
    const fn getBlockIndex(x: usize, y: usize, z: usize) -> usize { x << 12 | z << 8 | y }
    pub fn getBlockState(&self, x: usize, y: usize, z: usize) -> IBlockState {
        if x >= 16 || z >= 16 || y >= 256 { return IBlockState::fromGlobalStateId(0); }
        IBlockState::fromGlobalStateId(self.data[Self::getBlockIndex(x, y, z)])
    }
    pub fn setBlockState(&mut self, x: usize, y: usize, z: usize, state: IBlockState) {
        if x < 16 && z < 16 && y < 256 { self.data[Self::getBlockIndex(x, y, z)] = state.getGlobalStateId(); }
    }
    pub fn findGroundBlockIdx(&self, x: usize, z: usize) -> i32 {
        if x >= 16 || z >= 16 { return 0; }
        for y in (0..256).rev() {
            if !self.getBlockState(x, y, z).isAir() { return y as i32; }
        }
        0
    }
}
impl Default for ChunkPrimer { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_index_layout_and_ground_scan_are_preserved() {
        let mut primer = ChunkPrimer::new();
        primer.setBlockState(3, 70, 9, IBlockState::fromGlobalStateId(1 << 4));
        assert_eq!(primer.getBlockState(3, 70, 9).getBlockId(), 1);
        assert_eq!(primer.findGroundBlockIdx(3, 9), 70);
    }
}
