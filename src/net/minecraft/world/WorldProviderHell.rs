use crate::net::minecraft::util::math::Vec3d::Vec3d;
use crate::net::minecraft::world::DimensionType::DimensionType;
use crate::net::minecraft::world::biome::Biome::Biome;
use crate::net::minecraft::world::biome::BiomeProviderSingle::BiomeProviderSingle;

/// MCP 1.12.2 `WorldProviderHell` specialization excluding the still-pending
/// BiomeProviderSingle/ChunkGeneratorHell runtime objects.
pub struct WorldProviderHell;
impl WorldProviderHell {
    /// MCP `createBiomeProvider` fixed-biome portion. World-owned side effects
    /// (Hell flags / End DragonFightManager) remain on the provider wrapper.
    pub fn createBiomeProvider() -> BiomeProviderSingle {
        BiomeProviderSingle::new(Biome::getBiome(8))
    }
    pub const fn getDimensionType() -> DimensionType { DimensionType::Nether }
    pub fn getFogColor() -> Vec3d { Vec3d::new(0.20000000298023224, 0.029999999329447746, 0.029999999329447746) }
    pub const fn calculateCelestialAngle() -> f32 { 0.5 }
    pub const fn isSurfaceWorld() -> bool { false }
    pub const fn canCoordinateBeSpawn() -> bool { false }
    pub const fn canRespawnHere() -> bool { false }
    pub const fn doesXZShowFog() -> bool { true }
    pub const fn hasNoSky() -> bool { true }
    pub const fn doesWaterVaporize() -> bool { true }
    pub fn generateLightBrightnessTable() -> [f32;16] {
        let mut table=[0.0;16];
        let mut i=0usize;
        while i<=15 {
            let inverse=1.0-i as f32/15.0;
            table[i]=(1.0-inverse)/(inverse*3.0+1.0)*0.9+0.1;
            i+=1;
        }
        table
    }
    /// MCP anonymous WorldBorder override uses an 1:8 coordinate projection.
    pub const fn borderCoordinate(value:f64)->f64 { value/8.0 }
}
