use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// MCP 1.12.2 `ChunkPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub chunkXPos: i32,
    pub chunkZPos: i32,
}

impl ChunkPos {
    pub const fn new(x: i32, z: i32) -> Self {
        Self {
            chunkXPos: x,
            chunkZPos: z,
        }
    }
    pub const fn fromBlockPos(pos: BlockPos) -> Self {
        Self::new(pos.x >> 4, pos.z >> 4)
    }

    /// MCP `ChunkPos#asLong`: unsigned low 32 bits X, high 32 bits Z.
    pub const fn asLong(x: i32, z: i32) -> i64 {
        (((x as i64) & 0xFFFF_FFFF) | (((z as i64) & 0xFFFF_FFFF) << 32)) as i64
    }

    /// Java `ChunkPos#hashCode`, including its cached-value formula. Rust's
    /// `Hash` trait remains structural; callers that require Java identity use
    /// this method explicitly.
    pub const fn javaHashCode(&self) -> i32 {
        let i = 1_664_525_i32
            .wrapping_mul(self.chunkXPos)
            .wrapping_add(1_013_904_223);
        let j = 1_664_525_i32
            .wrapping_mul(self.chunkZPos ^ -559_038_737_i32)
            .wrapping_add(1_013_904_223);
        i ^ j
    }

    pub const fn getXStart(&self) -> i32 {
        self.chunkXPos << 4
    }
    pub const fn getZStart(&self) -> i32 {
        self.chunkZPos << 4
    }
    pub const fn getXEnd(&self) -> i32 {
        (self.chunkXPos << 4) + 15
    }
    pub const fn getZEnd(&self) -> i32 {
        (self.chunkZPos << 4) + 15
    }
    pub const fn getBlock(&self, x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos::new((self.chunkXPos << 4) + x, y, (self.chunkZPos << 4) + z)
    }
}

impl std::fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.chunkXPos, self.chunkZPos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_layout_preserves_signed_chunk_coordinates() {
        for (x, z) in [(0, 0), (1, -1), (-30_000, 30_000), (i32::MIN, i32::MAX)] {
            let packed = ChunkPos::asLong(x, z) as u64;
            assert_eq!(packed as u32 as i32, x);
            assert_eq!((packed >> 32) as u32 as i32, z);
        }
    }

    #[test]
    fn world_coordinate_bounds_match_mcp() {
        let pos = ChunkPos::new(-2, 3);
        assert_eq!(pos.getXStart(), -32);
        assert_eq!(pos.getXEnd(), -17);
        assert_eq!(pos.getZStart(), 48);
        assert_eq!(pos.getZEnd(), 63);
        assert_eq!(pos.getBlock(4, 70, 5), BlockPos::new(-28, 70, 53));
    }
}
