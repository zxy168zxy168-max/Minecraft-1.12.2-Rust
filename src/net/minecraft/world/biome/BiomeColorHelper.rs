use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::world::biome::Biome::Biome;
use crate::net::minecraft::world::ColorizerFoliage::ColorizerFoliage;
use crate::net::minecraft::world::ColorizerGrass::ColorizerGrass;

/// Minimal biome lookup contract corresponding to `IBlockAccess.getBiome`.
pub trait BiomeAccess {
    fn getBiomeId(&self, pos: BlockPos) -> u8;

    /// Colour handlers such as `BlockDoublePlant` need the actual lower-half
    /// state while retaining the narrow biome-access abstraction.
    fn getBlockStateForColor(
        &self,
        pos: BlockPos,
    ) -> crate::net::minecraft::block::state::IBlockState::IBlockState;
}

/// MCP 1.12.2 `BiomeColorHelper`: average a 3x3 horizontal neighbourhood.
pub struct BiomeColorHelper;

impl BiomeColorHelper {
    pub fn getGrassColorAtPos<A: BiomeAccess>(
        access: &A,
        pos: BlockPos,
        grass: &ColorizerGrass,
    ) -> i32 {
        average_color(access, pos, |biome, sample| {
            biome.getGrassColorAtPos(sample, grass)
        })
    }

    pub fn getFoliageColorAtPos<A: BiomeAccess>(
        access: &A,
        pos: BlockPos,
        foliage: &ColorizerFoliage,
    ) -> i32 {
        average_color(access, pos, |biome, sample| {
            biome.getFoliageColorAtPos(sample, foliage)
        })
    }

    pub fn getWaterColorAtPos<A: BiomeAccess>(access: &A, pos: BlockPos) -> i32 {
        average_color(access, pos, |biome, _| biome.getWaterColor())
    }
}

fn average_color<A: BiomeAccess>(
    access: &A,
    pos: BlockPos,
    resolver: impl Fn(Biome, BlockPos) -> i32,
) -> i32 {
    let mut red = 0;
    let mut green = 0;
    let mut blue = 0;
    for z in (pos.z - 1)..=(pos.z + 1) {
        for x in (pos.x - 1)..=(pos.x + 1) {
            let sample = BlockPos::new(x, pos.y, z);
            let color = resolver(Biome::getBiome(access.getBiomeId(sample)), sample);
            red += (color >> 16) & 255;
            green += (color >> 8) & 255;
            blue += color & 255;
        }
    }
    ((red / 9) & 255) << 16 | ((green / 9) & 255) << 8 | (blue / 9) & 255
}
