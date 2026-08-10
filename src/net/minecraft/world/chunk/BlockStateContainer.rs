use crate::net::minecraft::network::PacketBuffer::{read_i64_be, read_u8, read_var_i32, write_i64_be, write_var_i32, CodecError};
use crate::net::minecraft::util::BitArray::BitArray;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Palette { Local(Vec<i32>), Registry }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateContainer {
    storage: BitArray,
    palette: Palette,
    bits: usize,
}

impl Default for BlockStateContainer {
    fn default() -> Self { Self::new() }
}

impl BlockStateContainer {
    pub fn new() -> Self {
        Self {
            storage: BitArray::new(4, 4096).expect("fixed BlockStateContainer dimensions"),
            palette: Palette::Local(vec![0]),
            bits: 4,
        }
    }

    pub fn read(buf: &mut &[u8]) -> Result<Self, CodecError> {
        let packetBits = read_u8(buf)? as usize;
        let bits = if packetBits <= 4 { 4 } else { packetBits };
        let palette = if bits <= 8 {
            let count = read_var_i32(buf)?;
            if count < 0 { return Err(CodecError::NegativeLength(count)); }
            let mut entries = Vec::with_capacity(count as usize);
            for _ in 0..count { entries.push(read_var_i32(buf)?); }
            Palette::Local(entries)
        } else { Palette::Registry };
        let longCount = read_var_i32(buf)?;
        if longCount < 0 { return Err(CodecError::NegativeLength(longCount)); }
        let mut longs = Vec::with_capacity(longCount as usize);
        for _ in 0..longCount { longs.push(read_i64_be(buf)? as u64); }
        let storage = BitArray::fromBacking(bits, 4096, longs)
            .map_err(CodecError::InvalidData)?;
        Ok(Self { storage, palette, bits })
    }

    const fn getIndex(x: usize, y: usize, z: usize) -> usize { y << 8 | z << 4 | x }

    pub fn getGlobalStateId(&self, x: usize, y: usize, z: usize) -> i32 {
        self.getGlobalStateIdAt(Self::getIndex(x, y, z))
    }

    pub fn getGlobalStateIdAt(&self, index: usize) -> i32 {
        let value = self.storage.getAt(index).unwrap_or(0) as usize;
        match &self.palette {
            Palette::Local(entries) => entries.get(value).copied().unwrap_or(0),
            Palette::Registry => value as i32,
        }
    }

    /// Mutable equivalent of MCP `BlockStateContainer.set`. Palette growth and
    /// the transition to the global registry preserve the Java container rules.
    pub fn setGlobalStateId(&mut self, x: usize, y: usize, z: usize, stateId: i32) -> Result<i32, String> {
        self.setGlobalStateIdAt(Self::getIndex(x, y, z), stateId)
    }

    pub fn setGlobalStateIdAt(&mut self, index: usize, stateId: i32) -> Result<i32, String> {
        if index >= 4096 { return Err(format!("index out of bounds: {index}")); }
        let stateId = stateId.max(0);
        let old = self.getGlobalStateIdAt(index);
        let paletteIndex = match &mut self.palette {
            Palette::Registry => Some(stateId as u32),
            Palette::Local(entries) => {
                if let Some(existing) = entries.iter().position(|entry| *entry == stateId) {
                    Some(existing as u32)
                } else if entries.len() < (1_usize << self.bits) {
                    entries.push(stateId);
                    Some((entries.len() - 1) as u32)
                } else {
                    None
                }
            }
        };
        // End the mutable palette borrow before resizing the whole container.
        let paletteIndex = match paletteIndex {
            Some(index) => index,
            None => self.resizeForState(stateId)?,
        };
        self.storage.setAt(index, paletteIndex)?;
        Ok(old)
    }

    fn resizeForState(&mut self, stateId: i32) -> Result<u32, String> {
        let oldValues = (0..4096).map(|index| self.getGlobalStateIdAt(index)).collect::<Vec<_>>();
        if self.bits < 8 {
            let mut entries = match &self.palette {
                Palette::Local(entries) => entries.clone(),
                Palette::Registry => unreachable!(),
            };
            entries.push(stateId);
            self.bits += 1;
            self.palette = Palette::Local(entries.clone());
            self.storage = BitArray::new(self.bits, 4096)?;
            for (index, value) in oldValues.into_iter().enumerate() {
                let paletteIndex = entries.iter().position(|entry| *entry == value).unwrap_or(0) as u32;
                self.storage.setAt(index, paletteIndex)?;
            }
            Ok((entries.len() - 1) as u32)
        } else {
            let maximum = oldValues.iter().copied().chain(std::iter::once(stateId)).max().unwrap_or(0) as u32;
            self.bits = (32 - maximum.leading_zeros()).max(9) as usize;
            self.palette = Palette::Registry;
            self.storage = BitArray::new(self.bits, 4096)?;
            for (index, value) in oldValues.into_iter().enumerate() {
                self.storage.setAt(index, value.max(0) as u32)?;
            }
            Ok(stateId as u32)
        }
    }

    /// MCP `BlockStateContainer#getDataForNBT`.  Anvil 1.12.2 stores the
    /// global state id as block-id low byte (`Blocks`), metadata (`Data`) and
    /// an optional high block-id nibble (`Add`).
    pub fn getDataForNBT(&self, blocks: &mut [u8; 4096], data: &mut crate::net::minecraft::world::chunk::NibbleArray::NibbleArray) -> Option<crate::net::minecraft::world::chunk::NibbleArray::NibbleArray> {
        let mut add: Option<crate::net::minecraft::world::chunk::NibbleArray::NibbleArray> = None;
        for index in 0..4096 {
            let state_id = self.getGlobalStateIdAt(index).max(0);
            let x = index & 15;
            let y = (index >> 8) & 15;
            let z = (index >> 4) & 15;
            let high = ((state_id >> 12) & 15) as u8;
            if high != 0 {
                let add_array = add.get_or_insert_with(crate::net::minecraft::world::chunk::NibbleArray::NibbleArray::new);
                add_array.set(x, y, z, high);
            }
            blocks[index] = ((state_id >> 4) & 255) as u8;
            data.set(x, y, z, (state_id & 15) as u8);
        }
        add
    }

    /// MCP `BlockStateContainer#setDataFromNBT`.
    pub fn setDataFromNBT(
        &mut self,
        blocks: &[u8],
        data: &crate::net::minecraft::world::chunk::NibbleArray::NibbleArray,
        add: Option<&crate::net::minecraft::world::chunk::NibbleArray::NibbleArray>,
    ) -> Result<(), String> {
        if blocks.len() != 4096 {
            return Err(format!("BlockStateContainer Blocks should be 4096 bytes not: {}", blocks.len()));
        }
        // Start from the vanilla minimum local palette just like a freshly
        // constructed container, then let `setGlobalStateIdAt` grow it using
        // the same palette transition rules as network/block updates.
        *self = Self::new();
        for index in 0..4096 {
            let x = index & 15;
            let y = (index >> 8) & 15;
            let z = (index >> 4) & 15;
            let high = add.map(|array| array.get(x, y, z) as i32).unwrap_or(0);
            let state_id = (high << 12) | ((blocks[index] as i32) << 4) | data.get(x, y, z) as i32;
            self.setGlobalStateIdAt(index, state_id)?;
        }
        Ok(())
    }

    /// MCP 1.12.2 `BlockStateContainer#write`.  The local palette is
    /// serialized before the BitArray; the registry palette deliberately has
    /// no palette payload.
    pub fn write(&self, output: &mut Vec<u8>) {
        output.push(self.bits as u8);
        match &self.palette {
            Palette::Local(entries) => {
                write_var_i32(entries.len() as i32, output);
                for entry in entries { write_var_i32(*entry, output); }
            }
            Palette::Registry => {}
        }
        write_var_i32(self.storage.getBackingLongArray().len() as i32, output);
        for value in self.storage.getBackingLongArray() { write_i64_be(*value as i64, output); }
    }

    /// MCP `BlockStateContainer#getSerializedSize`.
    pub fn getSerializedSize(&self) -> usize {
        let palette_size = match &self.palette {
            Palette::Local(entries) => {
                crate::net::minecraft::network::PacketBuffer::var_i32_size(entries.len() as i32)
                    + entries.iter().map(|entry| crate::net::minecraft::network::PacketBuffer::var_i32_size(*entry)).sum::<usize>()
            }
            Palette::Registry => 0,
        };
        1 + palette_size
            + crate::net::minecraft::network::PacketBuffer::var_i32_size(self.storage.getBackingLongArray().len() as i32)
            + self.storage.getBackingLongArray().len() * 8
    }

    pub fn bits(&self) -> usize { self.bits }
    pub fn palette(&self) -> &Palette { &self.palette }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_palette_grows_and_preserves_existing_states() {
        let mut container = BlockStateContainer::new();
        for index in 0..20 {
            container.setGlobalStateIdAt(index, index as i32 + 1).unwrap();
        }
        assert!(container.bits() >= 5);
        for index in 0..20 { assert_eq!(container.getGlobalStateIdAt(index), index as i32 + 1); }
    }

    #[test]
    fn palette_switches_to_registry_after_eight_bits() {
        let mut container = BlockStateContainer::new();
        for index in 0..300 {
            container.setGlobalStateIdAt(index, index as i32 + 1).unwrap();
        }
        assert!(matches!(container.palette(), Palette::Registry));
        assert_eq!(container.getGlobalStateIdAt(299), 300);
    }
    #[test]
    fn anvil_nbt_arrays_roundtrip_global_state_ids() {
        let mut container = BlockStateContainer::new();
        for (index, state) in [(0, 1 << 4), (1, (2 << 4) | 3), (257, (255 << 4) | 15)] {
            container.setGlobalStateIdAt(index, state).unwrap();
        }
        let mut blocks = [0_u8; 4096];
        let mut data = crate::net::minecraft::world::chunk::NibbleArray::NibbleArray::new();
        let add = container.getDataForNBT(&mut blocks, &mut data);
        assert!(add.is_none());
        let mut decoded = BlockStateContainer::new();
        decoded.setDataFromNBT(&blocks, &data, None).unwrap();
        for index in [0, 1, 257, 4095] {
            assert_eq!(decoded.getGlobalStateIdAt(index), container.getGlobalStateIdAt(index));
        }
    }

    #[test]
    fn anvil_add_nibble_is_emitted_for_high_block_id_bits() {
        let mut container = BlockStateContainer::new();
        container.setGlobalStateIdAt(42, 0x1234).unwrap();
        let mut blocks = [0_u8; 4096];
        let mut data = crate::net::minecraft::world::chunk::NibbleArray::NibbleArray::new();
        let add = container.getDataForNBT(&mut blocks, &mut data).expect("high block id needs Add nibble");
        let x = 42 & 15;
        let y = (42 >> 8) & 15;
        let z = (42 >> 4) & 15;
        assert_eq!(blocks[42], 0x23);
        assert_eq!(data.get(x, y, z), 0x4);
        assert_eq!(add.get(x, y, z), 0x1);
        let mut decoded = BlockStateContainer::new();
        decoded.setDataFromNBT(&blocks, &data, Some(&add)).unwrap();
        assert_eq!(decoded.getGlobalStateIdAt(42), 0x1234);
    }

}
