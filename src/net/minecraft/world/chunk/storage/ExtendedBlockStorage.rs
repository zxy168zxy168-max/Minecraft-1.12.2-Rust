use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::world::chunk::BlockStateContainer::BlockStateContainer;
use crate::net::minecraft::world::chunk::NibbleArray::NibbleArray;

/// MCP 1.12.2 `ExtendedBlockStorage`.
///
/// The renderer still receives immutable `Arc` snapshots from `Chunk`; the
/// block/tick reference counters are source state used by the integrated
/// server and by Anvil deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedBlockStorage {
    yBase: i32,
    blockRefCount: i32,
    tickRefCount: i32,
    data: BlockStateContainer,
    blocklightArray: NibbleArray,
    skylightArray: Option<NibbleArray>,
}

impl ExtendedBlockStorage {
    /// MCP constructor `ExtendedBlockStorage(int y, boolean storeSkylight)`.
    pub fn new(y: i32, storeSkylight: bool) -> Self {
        Self {
            yBase: y,
            blockRefCount: 0,
            tickRefCount: 0,
            data: BlockStateContainer::new(),
            blocklightArray: NibbleArray::new(),
            skylightArray: storeSkylight.then(NibbleArray::new),
        }
    }

    /// Network fill equivalent. `Chunk#fillChunk` calls
    /// `removeInvalidBlocks()` for every section after reading packet data;
    /// doing it here gives the same final counters for the current Rust
    /// packet reader, which constructs complete sections in one operation.
    pub fn fromNetwork(
        y: i32,
        data: BlockStateContainer,
        blocklightArray: NibbleArray,
        skylightArray: Option<NibbleArray>,
    ) -> Self {
        let mut result = Self {
            yBase: y,
            blockRefCount: 0,
            tickRefCount: 0,
            data,
            blocklightArray,
            skylightArray,
        };
        result.removeInvalidBlocks();
        result
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> IBlockState {
        IBlockState::fromGlobalStateId(self.data.getGlobalStateId(x, y, z))
    }

    /// MCP `ExtendedBlockStorage#set`: update non-air and random-tick counts
    /// before replacing the state in the palette container.
    pub fn set(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        state: IBlockState,
    ) -> Result<IBlockState, String> {
        let old = self.get(x, y, z);
        if !old.isAir() {
            self.blockRefCount -= 1;
            if old.getBlock().getTickRandomly() {
                self.tickRefCount -= 1;
            }
        }
        if !state.isAir() {
            self.blockRefCount += 1;
            if state.getBlock().getTickRandomly() {
                self.tickRefCount += 1;
            }
        }
        self.data
            .setGlobalStateId(x, y, z, state.getGlobalStateId())?;
        Ok(old)
    }

    pub const fn isEmpty(&self) -> bool {
        self.blockRefCount == 0
    }
    pub const fn getNeedsRandomTick(&self) -> bool {
        self.tickRefCount > 0
    }
    pub const fn getBlockRefCount(&self) -> i32 {
        self.blockRefCount
    }
    pub const fn getTickRefCount(&self) -> i32 {
        self.tickRefCount
    }

    pub fn getGlobalStateId(&self, x: usize, y: usize, z: usize) -> i32 {
        self.data.getGlobalStateId(x, y, z)
    }
    pub fn getExtBlocklightValue(&self, x: usize, y: usize, z: usize) -> u8 {
        self.blocklightArray.get(x, y, z)
    }
    pub fn setExtBlocklightValue(&mut self, x: usize, y: usize, z: usize, value: u8) {
        self.blocklightArray.set(x, y, z, value);
    }
    pub fn getExtSkylightValue(&self, x: usize, y: usize, z: usize) -> u8 {
        self.skylightArray
            .as_ref()
            .map(|a| a.get(x, y, z))
            .unwrap_or(0)
    }
    pub fn setExtSkylightValue(&mut self, x: usize, y: usize, z: usize, value: u8) {
        if let Some(array) = self.skylightArray.as_mut() {
            array.set(x, y, z, value);
        }
    }
    pub const fn getYLocation(&self) -> i32 {
        self.yBase
    }
    pub fn getData(&self) -> &BlockStateContainer {
        &self.data
    }
    pub fn getDataMut(&mut self) -> &mut BlockStateContainer {
        &mut self.data
    }
    pub fn getBlocklightArray(&self) -> &NibbleArray {
        &self.blocklightArray
    }
    pub fn getSkylightArray(&self) -> Option<&NibbleArray> {
        self.skylightArray.as_ref()
    }
    pub fn setBlocklightArray(&mut self, value: NibbleArray) {
        self.blocklightArray = value;
    }
    pub fn setSkylightArray(&mut self, value: NibbleArray) {
        self.skylightArray = Some(value);
    }

    /// MCP `removeInvalidBlocks`: rebuild both counters from the 4096 stored
    /// states. This is required after network/Anvil bulk loading.
    pub fn removeInvalidBlocks(&mut self) {
        let mut block_count = 0;
        let mut tick_count = 0;
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let state = self.get(x, y, z);
                    if !state.isAir() {
                        block_count += 1;
                        if state.getBlock().getTickRandomly() {
                            tick_count += 1;
                        }
                    }
                }
            }
        }
        self.blockRefCount = block_count;
        self.tickRefCount = tick_count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_counts_follow_air_and_random_tick_transitions() {
        let mut storage = ExtendedBlockStorage::new(0, true);
        assert!(storage.isEmpty());
        // grass id 2 is random-ticking in 1.12.2.
        storage
            .set(1, 2, 3, IBlockState::fromGlobalStateId(2 << 4))
            .unwrap();
        assert_eq!(storage.getBlockRefCount(), 1);
        assert_eq!(storage.getTickRefCount(), 1);
        storage
            .set(1, 2, 3, IBlockState::fromGlobalStateId(1 << 4))
            .unwrap();
        assert_eq!(storage.getBlockRefCount(), 1);
        assert_eq!(storage.getTickRefCount(), 0);
        storage
            .set(1, 2, 3, IBlockState::fromGlobalStateId(0))
            .unwrap();
        assert!(storage.isEmpty());
    }

    #[test]
    fn bulk_counter_rebuild_matches_loaded_states() {
        let mut data = BlockStateContainer::new();
        data.setGlobalStateId(0, 0, 0, 2 << 4).unwrap();
        data.setGlobalStateId(1, 0, 0, 1 << 4).unwrap();
        let storage = ExtendedBlockStorage::fromNetwork(
            0,
            data,
            NibbleArray::new(),
            Some(NibbleArray::new()),
        );
        assert_eq!(storage.getBlockRefCount(), 2);
        assert_eq!(storage.getTickRefCount(), 1);
    }
}
