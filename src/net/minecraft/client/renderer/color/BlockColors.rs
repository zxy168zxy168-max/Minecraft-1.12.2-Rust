use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::world::biome::BiomeColorHelper::{BiomeAccess, BiomeColorHelper};
use crate::net::minecraft::world::ColorizerFoliage::ColorizerFoliage;
use crate::net::minecraft::world::ColorizerGrass::ColorizerGrass;

/// Source-derived rendering subset of MCP 1.12.2 `BlockColors` registrations.
#[derive(Debug, Clone)]
pub struct BlockColors {
    grass: ColorizerGrass,
    foliage: ColorizerFoliage,
}

impl BlockColors {
    pub const DEFAULT_COLOR: i32 = -1;

    pub fn new(grass: ColorizerGrass, foliage: ColorizerFoliage) -> Self {
        Self { grass, foliage }
    }

    pub fn colorMultiplier<A: BiomeAccess>(
        &self,
        state: IBlockState,
        access: &A,
        pos: BlockPos,
        tintIndex: i32,
    ) -> i32 {
        let block_id = state.getBlockId();
        let meta = state.getMetadata();
        match block_id {
            2 if tintIndex == 0 => BiomeColorHelper::getGrassColorAtPos(access, pos, &self.grass),
            31 | 83 | 106 => BiomeColorHelper::getGrassColorAtPos(access, pos, &self.grass),
            175 => {
                // `BlockDoublePlant#getActualState` copies VARIANT from the
                // lower half. Only double grass and double fern are tinted.
                let lower_pos = if meta & 8 != 0 { pos.down(1) } else { pos };
                let lower_meta = access.getBlockStateForColor(lower_pos).getMetadata() & 7;
                if matches!(lower_meta, 2 | 3) {
                    BiomeColorHelper::getGrassColorAtPos(access, lower_pos, &self.grass)
                } else {
                    Self::DEFAULT_COLOR
                }
            }
            18 | 161 => match meta & 3 {
                1 => ColorizerFoliage::getFoliageColorPine(),
                2 => ColorizerFoliage::getFoliageColorBirch(),
                _ => BiomeColorHelper::getFoliageColorAtPos(access, pos, &self.foliage),
            },
            8 | 9 => BiomeColorHelper::getWaterColorAtPos(access, pos),
            111 => 2_129_968,
            55 => redstone_color(meta),
            104 | 105 => stem_color(meta),
            _ => Self::DEFAULT_COLOR,
        }
    }
}

fn redstone_color(power: i32) -> i32 {
    let f = (power.clamp(0, 15) as f32) / 15.0;
    let red = f * 0.6 + 0.4;
    let green = (f * f * 0.7 - 0.5).max(0.0);
    let blue = (f * f * 0.6 - 0.7).max(0.0);
    ((red * 255.0) as i32) << 16 | ((green * 255.0) as i32) << 8 | (blue * 255.0) as i32
}

fn stem_color(age: i32) -> i32 {
    let age = age.clamp(0, 7);
    let red = age * 32;
    let green = 255 - age * 8;
    let blue = age * 4;
    red << 16 | green << 8 | blue
}
