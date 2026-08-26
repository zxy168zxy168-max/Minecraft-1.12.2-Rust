use crate::net::minecraft::util::math::MathHelper::{clamp_f32, cos, sin, PI};
use crate::net::minecraft::util::math::Vec3d::Vec3d;
use crate::net::minecraft::world::biome::Biome::Biome;
use crate::net::minecraft::world::biome::BiomeProvider::BiomeProvider;
use crate::net::minecraft::world::biome::BiomeProviderKind::BiomeProviderKind;
use crate::net::minecraft::world::biome::BiomeProviderSingle::BiomeProviderSingle;
use crate::net::minecraft::world::gen::ChunkGeneratorFlat::ChunkGeneratorFlat;
use crate::net::minecraft::world::gen::ChunkGeneratorOverworld::ChunkGeneratorOverworld;
use crate::net::minecraft::world::gen::FlatGeneratorInfo::FlatGeneratorInfo;
use crate::net::minecraft::world::gen::IChunkGenerator::IChunkGenerator;
use crate::net::minecraft::world::storage::WorldInfo::WorldInfo;
use crate::net::minecraft::world::DimensionType::DimensionType;
use crate::net::minecraft::world::WorldProviderEnd::WorldProviderEnd;
use crate::net::minecraft::world::WorldProviderHell::WorldProviderHell;
use crate::net::minecraft::world::WorldProviderSurface::WorldProviderSurface;
use crate::net::minecraft::world::WorldType::WorldType;

/// MCP 1.12.2 `WorldProvider` vanilla-dimension dispatch.
///
/// Rust keeps a compact tagged provider here so existing renderer/server call
/// sites do not need Java reflection. Dimension-specific responsibilities are
/// implemented by `WorldProviderSurface`, `WorldProviderHell`, and
/// `WorldProviderEnd`; world-owned biome/generator objects are connected only
/// as their source dependencies are ported.
#[derive(Debug, Clone)]
pub struct WorldProvider {
    dimension: i32,
    dimensionType: DimensionType,
    lightBrightnessTable: [f32; 16],
    terrainType: WorldType,
    generatorSettings: String,
    biomeProvider: Option<BiomeProviderKind>,
    field_191067_f: bool,
}

impl WorldProvider {
    pub fn new(dimension: i32) -> Self {
        let dimensionType =
            DimensionType::getById(dimension).unwrap_or_else(|error| panic!("{error}"));
        let lightBrightnessTable = if dimensionType == DimensionType::Nether {
            WorldProviderHell::generateLightBrightnessTable()
        } else {
            let mut table = [0.0; 16];
            for (level, brightness) in table.iter_mut().enumerate() {
                let inverse = 1.0 - level as f32 / 15.0;
                *brightness = (1.0 - inverse) / (inverse * 3.0 + 1.0);
            }
            table
        };
        let (biomeProvider, field_191067_f) = match dimensionType {
            DimensionType::Nether => (
                Some(BiomeProviderKind::Single(BiomeProviderSingle::new(
                    Biome::getBiome(8),
                ))),
                false,
            ),
            DimensionType::TheEnd => (
                Some(BiomeProviderKind::Single(BiomeProviderSingle::new(
                    Biome::getBiome(9),
                ))),
                false,
            ),
            DimensionType::Overworld => (None, true),
        };
        Self {
            dimension,
            dimensionType,
            lightBrightnessTable,
            terrainType: WorldType::Default,
            generatorSettings: String::new(),
            biomeProvider,
            field_191067_f,
        }
    }

    /// MCP `WorldProvider#setWorld` storage-visible configuration: terrain
    /// type/options are copied from WorldInfo and `createBiomeProvider` owns
    /// the fixed-biome provider choice for flat/debug/Nether/End worlds.
    pub fn configureFromWorldInfo(&mut self, info: &WorldInfo) {
        self.terrainType = info.getTerrainType();
        self.generatorSettings = info.getGeneratorOptions().to_owned();
        match self.dimensionType {
            DimensionType::Overworld => {
                self.field_191067_f = true;
                self.biomeProvider = match self.terrainType {
                    WorldType::Flat => {
                        let flat = FlatGeneratorInfo::createFlatGeneratorFromString(
                            &self.generatorSettings,
                        );
                        Some(BiomeProviderKind::Single(BiomeProviderSingle::new(
                            Biome::getBiome(flat.getBiome().clamp(0, 255) as u8),
                        )))
                    }
                    WorldType::DebugWorld => Some(BiomeProviderKind::Single(
                        BiomeProviderSingle::new(Biome::getBiome(1)),
                    )),
                    _ => Some(BiomeProviderKind::Layered(BiomeProvider::new(
                        info.getSeed(),
                        self.terrainType,
                        &self.generatorSettings,
                    ))),
                };
            }
            DimensionType::Nether => {
                self.field_191067_f = false;
                self.biomeProvider = Some(BiomeProviderKind::Single(BiomeProviderSingle::new(
                    Biome::getBiome(8),
                )));
            }
            DimensionType::TheEnd => {
                self.field_191067_f = false;
                self.biomeProvider = Some(BiomeProviderKind::Single(BiomeProviderSingle::new(
                    Biome::getBiome(9),
                )));
            }
        }
    }

    pub const fn getTerrainType(&self) -> WorldType {
        self.terrainType
    }
    pub fn getGeneratorSettings(&self) -> &str {
        &self.generatorSettings
    }
    pub fn getBiomeProvider(&self) -> Option<&BiomeProviderKind> {
        self.biomeProvider.as_ref()
    }

    /// MCP `WorldProvider#createChunkGenerator` at the currently migrated
    /// generator boundary. Unsupported vanilla generators return an explicit
    /// error instead of an empty/fabricated Chunk.
    pub fn createChunkGenerator(
        &self,
        seed: i64,
        mapFeaturesEnabled: bool,
    ) -> Result<Box<dyn IChunkGenerator>, String> {
        match (self.dimensionType, self.terrainType) {
            (DimensionType::Overworld, WorldType::Flat) => Ok(Box::new(ChunkGeneratorFlat::new(
                seed,
                mapFeaturesEnabled,
                &self.generatorSettings,
            ))),
            (DimensionType::Overworld, WorldType::DebugWorld) => {
                Err("ChunkGeneratorDebug has not yet been ported".to_owned())
            }
            (
                DimensionType::Overworld,
                WorldType::Default
                | WorldType::Default11
                | WorldType::LargeBiomes
                | WorldType::Amplified
                | WorldType::Customized,
            ) => {
                let biomeProvider = self.biomeProvider.clone().ok_or_else(|| {
                    "Overworld BiomeProvider/GenLayer graph is not configured".to_owned()
                })?;
                Ok(Box::new(ChunkGeneratorOverworld::new(
                    seed,
                    mapFeaturesEnabled,
                    self.terrainType,
                    &self.generatorSettings,
                    biomeProvider,
                )))
            }
            (DimensionType::Nether, _) => {
                Err("ChunkGeneratorHell has not yet been ported".to_owned())
            }
            (DimensionType::TheEnd, _) => {
                Err("ChunkGeneratorEnd has not yet been ported".to_owned())
            }
        }
    }

    /// MCP `WorldProvider#getAverageGroundLevel`.
    pub fn getAverageGroundLevel(&self, seaLevel: i32) -> i32 {
        if self.terrainType == WorldType::Flat {
            4
        } else {
            seaLevel + 1
        }
    }

    pub const fn getDimension(&self) -> i32 {
        self.dimension
    }

    /// MCP subclass `getDimensionType` dispatch.
    pub const fn getDimensionType(&self) -> DimensionType {
        self.dimensionType
    }

    /// Rust equivalent of `DimensionType#createDimension`.
    pub fn forDimensionType(dimensionType: DimensionType) -> Self {
        Self::new(dimensionType.getId())
    }

    /// MCP `WorldProvider#getHasNoSky`. Only `WorldProviderHell` sets this
    /// field in `createBiomeProvider`; the surface and End providers leave it
    /// false.
    pub const fn getHasNoSky(&self) -> bool {
        match self.dimensionType {
            DimensionType::Nether => WorldProviderHell::hasNoSky(),
            DimensionType::TheEnd => WorldProviderEnd::hasNoSky(),
            DimensionType::Overworld => false,
        }
    }

    /// MCP base/Nether/End respawn contract.
    pub const fn canRespawnHere(&self) -> bool {
        match self.dimensionType {
            DimensionType::Overworld => true,
            DimensionType::Nether => WorldProviderHell::canRespawnHere(),
            DimensionType::TheEnd => WorldProviderEnd::canRespawnHere(),
        }
    }

    /// MCP `WorldProvider#getMoonPhase`.
    pub fn getMoonPhase(&self, worldTime: i64) -> i32 {
        ((worldTime / 24_000).rem_euclid(8)) as i32
    }

    /// MCP `WorldProvider.func_191066_m`: true only when the provider's
    /// `createBiomeProvider` enables the skylight storage flag. In vanilla
    /// 1.12.2 this is the surface provider only; both the Nether and the End
    /// leave the flag false, so their chunk sections omit the 2048-byte sky
    /// nibble array.
    pub const fn hasSkyLight(&self) -> bool {
        self.field_191067_f
    }

    pub const fn getLightBrightnessTable(&self) -> &[f32; 16] {
        &self.lightBrightnessTable
    }

    /// MCP `WorldProvider#getCloudHeight`. Vanilla surface clouds use the
    /// fixed Y=128 plane; the client option adds its separate 0..128 offset.
    pub const fn getCloudHeight(&self) -> f32 {
        match self.dimensionType {
            DimensionType::TheEnd => WorldProviderEnd::getCloudHeight(),
            _ => 128.0,
        }
    }

    /// MCP `WorldProvider#calcSunriseSunsetColors`. End overrides this to
    /// return null; surface and Nether inherit the base computation. Rust
    /// returns the four source values by value rather than exposing the Java
    /// provider's reusable private array.
    pub fn calcSunriseSunsetColors(
        &self,
        celestialAngle: f32,
        _partialTicks: f32,
    ) -> Option<[f32; 4]> {
        if self.dimensionType == DimensionType::TheEnd {
            return None;
        }
        let f = 0.4_f32;
        let f1 = cos(celestialAngle * (PI * 2.0_f32));
        if !(-f..=f).contains(&f1) {
            return None;
        }
        let f3 = f1 / f * 0.5_f32 + 0.5_f32;
        let mut f4 = 1.0_f32 - (1.0_f32 - sin(f3 * PI)) * 0.99_f32;
        f4 *= f4;
        Some([
            f3 * 0.3_f32 + 0.7_f32,
            f3 * f3 * 0.7_f32 + 0.2_f32,
            f3 * f3 * 0.0_f32 + 0.2_f32,
            f4,
        ])
    }

    /// MCP `WorldProvider#getFogColor` with the exact vanilla subclass
    /// dispatch. The base provider keeps MathHelper's lookup-table cosine and
    /// float arithmetic before widening the channels into Vec3d.
    pub fn getFogColor(&self, celestialAngle: f32, _partialTicks: f32) -> Vec3d {
        match self.dimensionType {
            DimensionType::Nether => WorldProviderHell::getFogColor(),
            DimensionType::TheEnd => WorldProviderEnd::getFogColor(),
            DimensionType::Overworld => {
                let mut f = cos(celestialAngle * (PI * 2.0_f32)) * 2.0_f32 + 0.5_f32;
                f = clamp_f32(f, 0.0_f32, 1.0_f32);
                let red = 0.7529412_f32 * (f * 0.94_f32 + 0.06_f32);
                let green = 0.84705883_f32 * (f * 0.94_f32 + 0.06_f32);
                let blue = 1.0_f32 * (f * 0.91_f32 + 0.09_f32);
                Vec3d::new(red as f64, green as f64, blue as f64)
            }
        }
    }

    /// MCP `WorldProvider#isSurfaceWorld`. Only the overworld owns the normal
    /// cloud layer; Nether and End providers return false.
    pub const fn isSurfaceWorld(&self) -> bool {
        match self.dimensionType {
            DimensionType::Overworld => true,
            DimensionType::Nether => WorldProviderHell::isSurfaceWorld(),
            DimensionType::TheEnd => WorldProviderEnd::isSurfaceWorld(),
        }
    }

    /// MCP `WorldProvider#doesWaterVaporize`. Vanilla `WorldProviderHell`
    /// sets `isHellWorld=true`; the surface and End providers leave it false.
    pub const fn doesWaterVaporize(&self) -> bool {
        matches!(self.dimensionType, DimensionType::Nether)
            && WorldProviderHell::doesWaterVaporize()
    }

    /// MCP `WorldProvider#canDropChunk` plus `WorldProviderSurface` override.
    /// Nether/End inherit the base `true`; the surface provider protects the
    /// vanilla spawn-chunk square through `World#isSpawnChunk`. Keeping this
    /// dispatch on the provider preserves the original class responsibility
    /// while Rust passes the already-computed spawn predicate explicitly to
    /// avoid a back-reference from the provider into World.
    pub const fn canDropChunk(&self, isSpawnChunk: bool) -> bool {
        match self.dimensionType {
            DimensionType::Overworld => WorldProviderSurface::canDropChunk(isSpawnChunk),
            _ => true,
        }
    }

    /// MCP `WorldProvider.calculateCelestialAngle`, including fixed Nether/End
    /// overrides from `WorldProviderHell` and `WorldProviderEnd`.
    pub fn calculateCelestialAngle(&self, worldTime: i64, partialTicks: f32) -> f32 {
        match self.dimensionType {
            DimensionType::Nether => WorldProviderHell::calculateCelestialAngle(),
            DimensionType::TheEnd => WorldProviderEnd::calculateCelestialAngle(),
            DimensionType::Overworld => {
                let mut angle =
                    (worldTime.rem_euclid(24_000) as f32 + partialTicks) / 24_000.0 - 0.25;
                if angle < 0.0 {
                    angle += 1.0;
                }
                if angle > 1.0 {
                    angle -= 1.0;
                }
                // MCP performs Math.cos in double precision after promoting
                // the float angle, then narrows the result back to float.
                let eased = 1.0_f32
                    - (((angle as f64 * std::f64::consts::PI).cos() + 1.0_f64) / 2.0_f64) as f32;
                angle + (eased - angle) / 3.0
            }
        }
    }
    /// MCP `WorldProvider#isSkyColored`, with End override.
    pub fn isSkyColored(&self) -> bool {
        self.dimensionType != DimensionType::TheEnd
    }

    /// MCP `WorldProvider#doesXZShowFog`.
    pub const fn doesXZShowFog(&self, _x: i32, _z: i32) -> bool {
        matches!(self.dimensionType, DimensionType::Nether)
    }

    /// MCP End fixed spawn coordinate; other providers return null.
    pub const fn getSpawnCoordinate(
        &self,
    ) -> Option<crate::net::minecraft::util::math::BlockPos::BlockPos> {
        match self.dimensionType {
            DimensionType::TheEnd => Some(WorldProviderEnd::getSpawnCoordinate()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brightness_table_matches_vanilla_endpoints() {
        let overworld = WorldProvider::new(0);
        assert!((overworld.getLightBrightnessTable()[0] - 0.0).abs() < 1.0e-6);
        assert!((overworld.getLightBrightnessTable()[15] - 1.0).abs() < 1.0e-6);
        let nether = WorldProvider::new(-1);
        assert!((nether.getLightBrightnessTable()[0] - 0.1).abs() < 1.0e-6);
        assert!((nether.getLightBrightnessTable()[15] - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn fixed_dimension_angles_match_provider_overrides() {
        assert_eq!(
            WorldProvider::new(-1).calculateCelestialAngle(12_000, 0.0),
            0.5
        );
        assert_eq!(
            WorldProvider::new(1).calculateCelestialAngle(12_000, 0.0),
            0.0
        );
    }

    #[test]
    fn only_surface_provider_has_chunk_skylight_arrays() {
        assert!(WorldProvider::new(0).hasSkyLight());
        assert!(!WorldProvider::new(-1).hasSkyLight());
        assert!(!WorldProvider::new(1).hasSkyLight());
    }

    #[test]
    fn vanilla_cloud_provider_contract_is_surface_only_at_y_128() {
        assert_eq!(WorldProvider::new(0).getCloudHeight(), 128.0);
        assert!(WorldProvider::new(0).isSurfaceWorld());
        assert!(!WorldProvider::new(-1).isSurfaceWorld());
        assert!(!WorldProvider::new(1).isSurfaceWorld());
        assert!(WorldProvider::new(-1).doesWaterVaporize());
        assert!(!WorldProvider::new(0).doesWaterVaporize());
        assert!(!WorldProvider::new(1).doesWaterVaporize());
    }
    #[test]
    fn fog_and_sunrise_dispatch_match_dimension_overrides() {
        let nether = WorldProvider::new(-1);
        assert_eq!(
            nether.getFogColor(0.1, 0.5),
            WorldProviderHell::getFogColor()
        );
        let end = WorldProvider::new(1);
        assert_eq!(end.getFogColor(0.1, 0.5), WorldProviderEnd::getFogColor());
        assert!(end.calcSunriseSunsetColors(0.0, 0.0).is_none());
        assert!(WorldProvider::new(0)
            .calcSunriseSunsetColors(0.25, 0.0)
            .is_some());
    }

    #[test]
    fn dimension_type_dispatch_matches_vanilla_provider_subclasses() {
        let surface = WorldProvider::forDimensionType(DimensionType::Overworld);
        let nether = WorldProvider::forDimensionType(DimensionType::Nether);
        let end = WorldProvider::forDimensionType(DimensionType::TheEnd);
        assert_eq!(surface.getDimensionType(), DimensionType::Overworld);
        assert!(surface.canRespawnHere());
        assert!(nether.getHasNoSky());
        assert!(!nether.canRespawnHere());
        assert!(!end.getHasNoSky());
        assert_eq!(end.getCloudHeight(), 8.0);
        assert_eq!(
            end.getSpawnCoordinate(),
            Some(crate::net::minecraft::util::math::BlockPos::BlockPos::new(
                100, 50, 0
            ))
        );
        assert_eq!(surface.getMoonPhase(24_000 * 9), 1);
    }
}
