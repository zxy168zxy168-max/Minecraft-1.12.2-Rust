#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitArray {
    longArray: Vec<u64>,
    bitsPerEntry: usize,
    maxEntryValue: u64,
    arraySize: usize,
}

impl BitArray {
    pub fn new(bitsPerEntryIn: usize, arraySizeIn: usize) -> Result<Self, String> {
        if !(1..=32).contains(&bitsPerEntryIn) {
            return Err(format!(
                "bitsPerEntry must be between 1 and 32: {bitsPerEntryIn}"
            ));
        }
        let longCount = arraySizeIn.saturating_mul(bitsPerEntryIn).div_ceil(64);
        Ok(Self {
            longArray: vec![0; longCount],
            bitsPerEntry: bitsPerEntryIn,
            maxEntryValue: (1_u64 << bitsPerEntryIn) - 1,
            arraySize: arraySizeIn,
        })
    }

    pub fn fromBacking(
        bitsPerEntryIn: usize,
        arraySizeIn: usize,
        longArray: Vec<u64>,
    ) -> Result<Self, String> {
        let mut result = Self::new(bitsPerEntryIn, arraySizeIn)?;
        if result.longArray.len() != longArray.len() {
            return Err(format!(
                "invalid backing length: expected {}, got {}",
                result.longArray.len(),
                longArray.len()
            ));
        }
        result.longArray = longArray;
        Ok(result)
    }

    pub fn setAt(&mut self, index: usize, value: u32) -> Result<(), String> {
        if index >= self.arraySize {
            return Err(format!("index out of bounds: {index}"));
        }
        if u64::from(value) > self.maxEntryValue {
            return Err(format!("value exceeds mask: {value}"));
        }
        let bitIndex = index * self.bitsPerEntry;
        let firstLong = bitIndex / 64;
        let lastLong = ((index + 1) * self.bitsPerEntry - 1) / 64;
        let startBit = bitIndex % 64;
        let value = u64::from(value) & self.maxEntryValue;
        self.longArray[firstLong] =
            (self.longArray[firstLong] & !(self.maxEntryValue << startBit)) | value << startBit;
        if firstLong != lastLong {
            let firstBits = 64 - startBit;
            let secondBits = self.bitsPerEntry - firstBits;
            self.longArray[lastLong] =
                (self.longArray[lastLong] >> secondBits << secondBits) | (value >> firstBits);
        }
        Ok(())
    }

    pub fn getAt(&self, index: usize) -> Result<u32, String> {
        if index >= self.arraySize {
            return Err(format!("index out of bounds: {index}"));
        }
        let bitIndex = index * self.bitsPerEntry;
        let firstLong = bitIndex / 64;
        let lastLong = ((index + 1) * self.bitsPerEntry - 1) / 64;
        let startBit = bitIndex % 64;
        let value = if firstLong == lastLong {
            self.longArray[firstLong] >> startBit
        } else {
            let firstBits = 64 - startBit;
            self.longArray[firstLong] >> startBit | self.longArray[lastLong] << firstBits
        };
        Ok((value & self.maxEntryValue) as u32)
    }

    pub fn getBackingLongArray(&self) -> &[u64] {
        &self.longArray
    }
    pub fn size(&self) -> usize {
        self.arraySize
    }
    pub fn bitsPerEntry(&self) -> usize {
        self.bitsPerEntry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn values_cross_long_boundary_like_java_bit_array() {
        let mut array = BitArray::new(5, 4096).unwrap();
        for index in 0..4096 {
            array.setAt(index, (index & 31) as u32).unwrap();
        }
        for index in 0..4096 {
            assert_eq!(array.getAt(index).unwrap(), (index & 31) as u32);
        }
    }
}
