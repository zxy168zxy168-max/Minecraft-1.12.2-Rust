use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::math::Vec3d::Vec3d;
use crate::net::minecraft::world::biome::Biome::Biome;
use crate::net::minecraft::world::biome::BiomeProviderSingle::BiomeProviderSingle;
use crate::net::minecraft::world::DimensionType::DimensionType;

/// MCP 1.12.2 `WorldProviderEnd` specialization. DragonFightManager ownership
/// remains pending with the concrete server entity/world runtime.
pub struct WorldProviderEnd;
impl WorldProviderEnd {
    /// MCP `createBiomeProvider` fixed-biome portion. World-owned side effects
    /// (Hell flags / End DragonFightManager) remain on the provider wrapper.
    pub fn createBiomeProvider() -> BiomeProviderSingle {
        BiomeProviderSingle::new(Biome::getBiome(9))
    }
    pub const fn getDimensionType() -> DimensionType {
        DimensionType::TheEnd
    }
    pub const fn calculateCelestialAngle() -> f32 {
        0.0
    }
    pub fn getFogColor() -> Vec3d {
        // Source computes all channels as float and only widens when
        // constructing Vec3d. Keep that narrowing/widening boundary exact.
        let red = (0.627451_f32 * 0.15_f32) as f64;
        let green = (0.5019608_f32 * 0.15_f32) as f64;
        let blue = (0.627451_f32 * 0.15_f32) as f64;
        Vec3d::new(red, green, blue)
    }
    pub const fn isSkyColored() -> bool {
        false
    }
    pub const fn canRespawnHere() -> bool {
        false
    }
    pub const fn isSurfaceWorld() -> bool {
        false
    }
    pub const fn getCloudHeight() -> f32 {
        8.0
    }
    pub const fn getSpawnCoordinate() -> BlockPos {
        BlockPos::new(100, 50, 0)
    }
    pub const fn getAverageGroundLevel() -> i32 {
        50
    }
    pub const fn doesXZShowFog() -> bool {
        false
    }
    /// Unlike the Nether, MCP WorldProviderEnd never sets `hasNoSky=true`.
    pub const fn hasNoSky() -> bool {
        false
    }
}
