use crate::compat::Java::JavaRandom;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::world::biome::Biome::Biome;
use crate::net::minecraft::world::biome::BiomeProvider::BiomeProvider;
use crate::net::minecraft::world::biome::BiomeProviderSingle::BiomeProviderSingle;

/// Rust dynamic-dispatch equivalent for the MCP `BiomeProvider` base class and
/// its `BiomeProviderSingle` subclass.
#[derive(Debug, Clone)]
pub enum BiomeProviderKind {
    Layered(BiomeProvider),
    Single(BiomeProviderSingle),
}
impl BiomeProviderKind {
    pub fn getBiome(&self, pos: BlockPos) -> Biome {
        match self {
            Self::Layered(p) => p.getBiome(pos),
            Self::Single(p) => p.getBiome(pos),
        }
    }
    pub fn getBiomesToSpawnIn(&self) -> [Biome; 7] {
        match self {
            Self::Layered(p) => p.getBiomesToSpawnIn(),
            Self::Single(p) => p.getBiomesToSpawnIn(),
        }
    }
    pub fn findBiomePosition(
        &self,
        x: i32,
        z: i32,
        range: i32,
        biomes: &[Biome],
        random: &mut JavaRandom,
    ) -> Option<BlockPos> {
        match self {
            Self::Layered(p) => p.findBiomePosition(x, z, range, biomes, random),
            Self::Single(p) => p.findBiomePosition(x, z, range, biomes, random),
        }
    }
    pub fn areBiomesViable(&self, x: i32, z: i32, radius: i32, allowed: &[Biome]) -> bool {
        match self {
            Self::Layered(p) => p.areBiomesViable(x, z, radius, allowed),
            Self::Single(p) => p.areBiomesViable(x, z, radius, allowed),
        }
    }
    pub fn getBiomesForGeneration(
        &self,
        reuse: Option<Vec<Biome>>,
        x: i32,
        z: i32,
        w: i32,
        h: i32,
    ) -> Vec<Biome> {
        match self {
            Self::Layered(p) => p.getBiomesForGeneration(reuse, x, z, w, h),
            Self::Single(p) => p.getBiomesForGeneration(reuse, x, z, w, h),
        }
    }
    pub fn getBiomes(
        &self,
        reuse: Option<Vec<Biome>>,
        x: i32,
        z: i32,
        w: i32,
        h: i32,
    ) -> Vec<Biome> {
        match self {
            Self::Layered(p) => p.getBiomes(reuse, x, z, w, h),
            Self::Single(p) => p.getBiomes(reuse, x, z, w, h),
        }
    }
    pub fn getBiomesCached(
        &self,
        reuse: Option<Vec<Biome>>,
        x: i32,
        z: i32,
        w: i32,
        h: i32,
        cache: bool,
    ) -> Vec<Biome> {
        match self {
            Self::Layered(p) => p.getBiomesCached(reuse, x, z, w, h, cache),
            Self::Single(p) => p.getBiomesCached(reuse, x, z, w, h, cache),
        }
    }
    pub fn cleanupCache(&self) {
        if let Self::Layered(p) = self {
            p.cleanupCache();
        }
    }
    pub fn func_190944_c(&self) -> bool {
        match self {
            Self::Layered(p) => p.func_190944_c(),
            Self::Single(p) => p.func_190944_c(),
        }
    }
    pub fn func_190943_d(&self) -> Option<Biome> {
        match self {
            Self::Layered(p) => p.func_190943_d(),
            Self::Single(p) => Some(p.func_190943_d()),
        }
    }
}
