#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NibbleArray {
    data: Vec<u8>,
}
impl Default for NibbleArray {
    fn default() -> Self {
        Self::new()
    }
}
impl NibbleArray {
    pub fn new() -> Self {
        Self {
            data: vec![0; 2048],
        }
    }
    pub fn fromStorage(storageArray: Vec<u8>) -> Result<Self, String> {
        if storageArray.len() != 2048 {
            return Err(format!(
                "ChunkNibbleArrays should be 2048 bytes not: {}",
                storageArray.len()
            ));
        }
        Ok(Self { data: storageArray })
    }
    const fn getCoordinateIndex(x: usize, y: usize, z: usize) -> usize {
        y << 8 | z << 4 | x
    }
    pub fn get(&self, x: usize, y: usize, z: usize) -> u8 {
        self.getFromIndex(Self::getCoordinateIndex(x, y, z))
    }
    pub fn set(&mut self, x: usize, y: usize, z: usize, value: u8) {
        self.setIndex(Self::getCoordinateIndex(x, y, z), value);
    }
    pub fn getFromIndex(&self, index: usize) -> u8 {
        let byte = self.data[index >> 1];
        if index & 1 == 0 {
            byte & 15
        } else {
            byte >> 4 & 15
        }
    }
    pub fn setIndex(&mut self, index: usize, value: u8) {
        let i = index >> 1;
        self.data[i] = if index & 1 == 0 {
            (self.data[i] & 0xF0) | (value & 15)
        } else {
            (self.data[i] & 0x0F) | ((value & 15) << 4)
        };
    }
    pub fn getData(&self) -> &[u8] {
        &self.data
    }
}
