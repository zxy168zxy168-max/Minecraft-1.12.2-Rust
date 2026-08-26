use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockRailBase::{
    direction, isAscending, isRailBlock, EnumRailDirection,
};
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// MCP 1.12.2 `EntityMinecart.Type`, which is also SPacketSpawnObject data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MinecartType {
    Rideable,
    Chest,
    Furnace,
    Tnt,
    Spawner,
    Hopper,
    CommandBlock,
}

impl MinecartType {
    pub const fn byId(id: i32) -> Self {
        match id {
            1 => Self::Chest,
            2 => Self::Furnace,
            3 => Self::Tnt,
            4 => Self::Spawner,
            5 => Self::Hopper,
            6 => Self::CommandBlock,
            _ => Self::Rideable,
        }
    }

    pub const fn id(self) -> i32 {
        match self {
            Self::Rideable => 0,
            Self::Chest => 1,
            Self::Furnace => 2,
            Self::Tnt => 3,
            Self::Spawner => 4,
            Self::Hopper => 5,
            Self::CommandBlock => 6,
        }
    }

    /// Internal protocol palette state used by the Rust block renderer for
    /// each subclass' MCP `getDefaultDisplayTile`. This project stores chunk
    /// states as `(block registry id << 4) | metadata`, whereas the minecart
    /// DATAWATCHER VARINT uses `Block#getStateId`'s distinct
    /// `block id | (metadata << 12)` representation. Explicit synchronized
    /// values therefore pass through `fromMcpBlockStateId` below.
    pub const fn defaultDisplayStateId(self, furnacePowered: bool) -> i32 {
        let (block, meta) = match self {
            Self::Rideable => (0, 0),
            Self::Chest => (54, 2),
            Self::Furnace => (if furnacePowered { 62 } else { 61 }, 2),
            Self::Tnt => (46, 0),
            Self::Spawner => (52, 0),
            Self::Hopper => (154, 0),
            Self::CommandBlock => (137, 0),
        };
        (block << 4) | meta
    }

    pub const fn defaultDisplayOffset(self) -> i32 {
        match self {
            Self::Chest => 8,
            Self::Hopper => 1,
            _ => 6,
        }
    }
}

pub struct EntityMinecart;

impl EntityMinecart {
    pub const WIDTH: f32 = 0.98;
    pub const HEIGHT: f32 = 0.7;
    pub const RENDER_SAMPLE_OFFSET: f64 = 0.30000001192092896;

    /// Convert the exact integer serialized by MCP `Block#getStateId`
    /// (`blockId | metadata << 12`) into this port's chunk/model palette key
    /// (`blockId << 4 | metadata`). Keeping this bridge explicit prevents a
    /// chest/furnace metadata value from being misread as a completely
    /// different block ID.
    pub const fn fromMcpBlockStateId(stateId: i32) -> i32 {
        let blockId = stateId & 4095;
        let metadata = (stateId >> 12) & 15;
        (blockId << 4) | metadata
    }

    /// Inverse used by regression tests and future metadata writes.
    pub const fn toMcpBlockStateId(protocolStateId: i32) -> i32 {
        let blockId = protocolStateId >> 4;
        let metadata = protocolStateId & 15;
        blockId | (metadata << 12)
    }

    /// MCP's rail endpoint table indexed by EnumRailDirection metadata.
    pub const MATRIX: [[[i32; 3]; 2]; 10] = [
        [[0, 0, -1], [0, 0, 1]],
        [[-1, 0, 0], [1, 0, 0]],
        [[-1, -1, 0], [1, 0, 0]],
        [[-1, 0, 0], [1, -1, 0]],
        [[0, 0, -1], [0, -1, 1]],
        [[0, -1, -1], [0, 0, 1]],
        [[0, 0, 1], [1, 0, 0]],
        [[0, 0, 1], [-1, 0, 0]],
        [[0, 0, -1], [-1, 0, 0]],
        [[0, 0, -1], [1, 0, 0]],
    ];

    pub const fn directionIndex(value: EnumRailDirection) -> usize {
        match value {
            EnumRailDirection::NorthSouth => 0,
            EnumRailDirection::EastWest => 1,
            EnumRailDirection::AscendingEast => 2,
            EnumRailDirection::AscendingWest => 3,
            EnumRailDirection::AscendingNorth => 4,
            EnumRailDirection::AscendingSouth => 5,
            EnumRailDirection::SouthEast => 6,
            EnumRailDirection::SouthWest => 7,
            EnumRailDirection::NorthWest => 8,
            EnumRailDirection::NorthEast => 9,
        }
    }

    pub fn getPos<F>(mut x: f64, mut y: f64, mut z: f64, mut stateAt: F) -> Option<[f64; 3]>
    where
        F: FnMut(BlockPos) -> IBlockState,
    {
        let i = x.floor() as i32;
        let mut j = y.floor() as i32;
        let k = z.floor() as i32;
        if isRailBlock(stateAt(BlockPos::new(i, j - 1, k))) {
            j -= 1;
        }
        let state = stateAt(BlockPos::new(i, j, k));
        if !isRailBlock(state) {
            return None;
        }
        let railDirection = direction(state);
        let matrix = Self::MATRIX[Self::directionIndex(railDirection)];
        let d0 = i as f64 + 0.5 + matrix[0][0] as f64 * 0.5;
        let d1 = j as f64 + 0.0625 + matrix[0][1] as f64 * 0.5;
        let d2 = k as f64 + 0.5 + matrix[0][2] as f64 * 0.5;
        let d3 = i as f64 + 0.5 + matrix[1][0] as f64 * 0.5;
        let d4 = j as f64 + 0.0625 + matrix[1][1] as f64 * 0.5;
        let d5 = k as f64 + 0.5 + matrix[1][2] as f64 * 0.5;
        let d6 = d3 - d0;
        let d7 = (d4 - d1) * 2.0;
        let d8 = d5 - d2;
        let d9 = if d6 == 0.0 {
            z - k as f64
        } else if d8 == 0.0 {
            x - i as f64
        } else {
            let d10 = x - d0;
            let d11 = z - d2;
            (d10 * d6 + d11 * d8) * 2.0
        };
        x = d0 + d6 * d9;
        y = d1 + d7 * d9;
        z = d2 + d8 * d9;
        if d7 < 0.0 {
            y += 1.0;
        }
        if d7 > 0.0 {
            y += 0.5;
        }
        Some([x, y, z])
    }

    pub fn getPosOffset<F>(
        mut x: f64,
        mut y: f64,
        mut z: f64,
        offset: f64,
        mut stateAt: F,
    ) -> Option<[f64; 3]>
    where
        F: FnMut(BlockPos) -> IBlockState,
    {
        let i = x.floor() as i32;
        let mut j = y.floor() as i32;
        let k = z.floor() as i32;
        if isRailBlock(stateAt(BlockPos::new(i, j - 1, k))) {
            j -= 1;
        }
        let state = stateAt(BlockPos::new(i, j, k));
        if !isRailBlock(state) {
            return None;
        }
        let railDirection = direction(state);
        y = j as f64;
        if isAscending(railDirection) {
            y = (j + 1) as f64;
        }
        let matrix = Self::MATRIX[Self::directionIndex(railDirection)];
        let mut dx = (matrix[1][0] - matrix[0][0]) as f64;
        let mut dz = (matrix[1][2] - matrix[0][2]) as f64;
        let length = (dx * dx + dz * dz).sqrt();
        dx /= length;
        dz /= length;
        x += dx * offset;
        z += dz * offset;
        if matrix[0][1] != 0
            && x.floor() as i32 - i == matrix[0][0]
            && z.floor() as i32 - k == matrix[0][2]
        {
            y += matrix[0][1] as f64;
        } else if matrix[1][1] != 0
            && x.floor() as i32 - i == matrix[1][0]
            && z.floor() as i32 - k == matrix[1][2]
        {
            y += matrix[1][1] as f64;
        }
        Self::getPos(x, y, z, stateAt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_rail(pos: BlockPos) -> IBlockState {
        if pos == BlockPos::new(0, 0, 0) {
            IBlockState::fromGlobalStateId(66 << 4)
        } else {
            IBlockState::default()
        }
    }

    #[test]
    fn type_fallback_and_default_contents_match_mcp() {
        assert_eq!(MinecartType::byId(99), MinecartType::Rideable);
        assert_eq!(
            MinecartType::Chest.defaultDisplayStateId(false),
            (54 << 4) | 2
        );
        assert_eq!(MinecartType::Hopper.defaultDisplayOffset(), 1);
        let mcpChestNorth = 54 | (2 << 12);
        assert_eq!(
            EntityMinecart::fromMcpBlockStateId(mcpChestNorth),
            (54 << 4) | 2
        );
        assert_eq!(
            EntityMinecart::toMcpBlockStateId((54 << 4) | 2),
            mcpChestNorth
        );
    }

    #[test]
    fn north_south_rail_projection_centers_x() {
        let p = EntityMinecart::getPos(0.2, 0.0, 0.75, flat_rail).unwrap();
        assert!((p[0] - 0.5).abs() < 1.0e-9);
        assert!((p[1] - 0.0625).abs() < 1.0e-9);
        assert!((p[2] - 0.75).abs() < 1.0e-9);
    }
}
