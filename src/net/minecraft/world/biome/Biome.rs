use crate::compat::Java::JavaRandom;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::world::chunk::ChunkPrimer::ChunkPrimer;
use crate::net::minecraft::world::gen::NoiseGeneratorPerlin::NoiseGeneratorPerlin;
use crate::net::minecraft::world::ColorizerFoliage::ColorizerFoliage;
use crate::net::minecraft::world::ColorizerGrass::ColorizerGrass;
use std::sync::OnceLock;

/// Rendering subset of MCP 1.12.2 `Biome`.
///
/// The table below preserves the registered vanilla biome temperature,
/// rainfall, water colour and the source-confirmed colour overrides used by
/// the block colour pipeline. Terrain generation and spawning remain outside
/// this client-side port.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Biome {
    id: u8,
    temperature: f32,
    rainfall: f32,
    waterColor: i32,
    colorKind: BiomeColorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BiomeColorKind {
    Default,
    Swamp,
    RoofedForest,
    Mesa,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TempCategory {
    Cold,
    Medium,
    Warm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiomeClass {
    Ocean,
    Plains,
    Desert,
    Hills,
    Forest,
    Taiga,
    Swamp,
    River,
    Hell,
    End,
    Snow,
    MushroomIsland,
    Beach,
    Jungle,
    StoneBeach,
    Savanna,
    Mesa,
    Void,
    ForestMutated,
    SavannaMutated,
}

impl Biome {
    const fn new(
        id: u8,
        temperature: f32,
        rainfall: f32,
        waterColor: i32,
        colorKind: BiomeColorKind,
    ) -> Self {
        Self {
            id,
            temperature,
            rainfall,
            waterColor,
            colorKind,
        }
    }

    pub fn getBiome(id: u8) -> Self {
        // Values mirror `Biome.registerBiomes`. Mutated biomes inherit the
        // source biome's colour parameters unless their registration supplies
        // replacements.
        let (temperature, rainfall, waterColor, colorKind) = match id {
            1 | 16 | 129 => (0.8, 0.4, 0xFFFFFF, BiomeColorKind::Default),
            2 | 17 | 130 => (2.0, 0.0, 0xFFFFFF, BiomeColorKind::Default),
            3 | 20 | 25 | 34 | 131 | 162 => (0.2, 0.3, 0xFFFFFF, BiomeColorKind::Default),
            4 | 18 | 132 => (0.7, 0.8, 0xFFFFFF, BiomeColorKind::Default),
            5 | 19 | 133 => (0.25, 0.8, 0xFFFFFF, BiomeColorKind::Default),
            6 | 134 => (0.8, 0.9, 14_745_518, BiomeColorKind::Swamp),
            8 => (2.0, 0.0, 0xFFFFFF, BiomeColorKind::Default),
            10 | 11 | 12 | 13 | 140 => (0.0, 0.5, 0xFFFFFF, BiomeColorKind::Default),
            14 | 15 => (0.9, 1.0, 0xFFFFFF, BiomeColorKind::Default),
            21 | 22 | 149 => (0.95, 0.9, 0xFFFFFF, BiomeColorKind::Default),
            23 | 151 => (0.95, 0.8, 0xFFFFFF, BiomeColorKind::Default),
            26 => (0.05, 0.3, 0xFFFFFF, BiomeColorKind::Default),
            27 | 28 | 155 | 156 => (0.6, 0.6, 0xFFFFFF, BiomeColorKind::Default),
            29 | 157 => (0.7, 0.8, 0xFFFFFF, BiomeColorKind::RoofedForest),
            30 | 31 | 158 => (-0.5, 0.4, 0xFFFFFF, BiomeColorKind::Default),
            32 | 33 => (0.3, 0.8, 0xFFFFFF, BiomeColorKind::Default),
            160 | 161 => (0.25, 0.8, 0xFFFFFF, BiomeColorKind::Default),
            35 => (1.2, 0.0, 0xFFFFFF, BiomeColorKind::Default),
            36 | 164 => (1.0, 0.0, 0xFFFFFF, BiomeColorKind::Default),
            163 => (1.1, 0.0, 0xFFFFFF, BiomeColorKind::Default),
            37..=39 | 165..=167 => (2.0, 0.0, 0xFFFFFF, BiomeColorKind::Mesa),
            // Ocean, river, sky, void and unspecified registry holes use
            // BiomeProperties' vanilla defaults.
            _ => (0.5, 0.5, 0xFFFFFF, BiomeColorKind::Default),
        };
        Self::new(id, temperature, rainfall, waterColor, colorKind)
    }

    /// MCP `BiomeProperties#biomeName` from the vanilla 1.12.2 registry.
    pub const fn getBiomeName(self) -> &'static str {
        match self.id {
            0 => "Ocean",
            1 => "Plains",
            2 => "Desert",
            3 => "Extreme Hills",
            4 => "Forest",
            5 => "Taiga",
            6 => "Swampland",
            7 => "River",
            8 => "Hell",
            9 => "The End",
            10 => "FrozenOcean",
            11 => "FrozenRiver",
            12 => "Ice Plains",
            13 => "Ice Mountains",
            14 => "MushroomIsland",
            15 => "MushroomIslandShore",
            16 => "Beach",
            17 => "DesertHills",
            18 => "ForestHills",
            19 => "TaigaHills",
            20 => "Extreme Hills Edge",
            21 => "Jungle",
            22 => "JungleHills",
            23 => "JungleEdge",
            24 => "Deep Ocean",
            25 => "Stone Beach",
            26 => "Cold Beach",
            27 => "Birch Forest",
            28 => "Birch Forest Hills",
            29 => "Roofed Forest",
            30 => "Cold Taiga",
            31 => "Cold Taiga Hills",
            32 => "Mega Taiga",
            33 => "Mega Taiga Hills",
            34 => "Extreme Hills+",
            35 => "Savanna",
            36 => "Savanna Plateau",
            37 => "Mesa",
            38 => "Mesa Plateau F",
            39 => "Mesa Plateau",
            127 => "The Void",
            129 => "Sunflower Plains",
            130 => "Desert M",
            131 => "Extreme Hills M",
            132 => "Flower Forest",
            133 => "Taiga M",
            134 => "Swampland M",
            140 => "Ice Plains Spikes",
            149 => "Jungle M",
            151 => "JungleEdge M",
            155 => "Birch Forest M",
            156 => "Birch Forest Hills M",
            157 => "Roofed Forest M",
            158 => "Cold Taiga M",
            160 => "Mega Spruce Taiga",
            161 => "Redwood Taiga Hills M",
            162 => "Extreme Hills+ M",
            163 => "Savanna M",
            164 => "Savanna Plateau M",
            165 => "Mesa (Bryce)",
            166 => "Mesa Plateau F M",
            167 => "Mesa Plateau M",
            _ => "Ocean",
        }
    }

    pub const fn getId(self) -> u8 {
        self.id
    }

    /// Registry-preserving counterpart of MCP `Biome#getBiomeForId`; unlike
    /// `getBiome`, this reports registry holes as None so GenLayer edge/hill
    /// logic can keep the source null semantics.
    pub fn getBiomeForId(id: i32) -> Option<Self> {
        if matches!(id, 0..=39 | 127 | 129..=134 | 140 | 149 | 151 | 155..=158 | 160..=167) {
            Some(Self::getBiome(id as u8))
        } else {
            None
        }
    }

    pub const fn getBaseHeight(self) -> f32 {
        match self.id {
            0 | 10 => -1.0,
            1 | 2 | 12 | 35 | 129 => 0.125,
            3 | 34 | 131 | 162 => 1.0,
            5 | 14 | 30 | 32 | 149 | 151 | 155 | 157 | 160 | 161 => 0.2,
            6 => -0.2,
            7 | 11 => -0.5,
            15 | 16 | 26 => 0.0,
            17 | 18 | 19 | 22 | 28 | 31 | 33 => 0.45,
            20 => 0.8,
            24 => -1.8,
            36 | 38 | 39 => 1.5,
            130 => 0.225,
            133 | 158 => 0.3,
            134 => -0.1,
            140 => 0.425,
            156 => 0.55,
            163 => 0.3625,
            164 => 1.05,
            166 | 167 => 0.45,
            _ => 0.1,
        }
    }

    pub const fn getHeightVariation(self) -> f32 {
        match self.id {
            0 | 6 | 10 | 24 => 0.1,
            1 | 2 | 12 | 35 | 129 => 0.05,
            3 | 34 | 131 | 162 => 0.5,
            7 | 11 => 0.0,
            13 | 17 | 18 | 19 | 20 | 22 | 28 | 31 | 33 | 166 | 167 => 0.3,
            14 => 0.3,
            15 | 16 | 26 | 36 | 38 | 39 => 0.025,
            25 => 0.8,
            130 => 0.25,
            132 | 133 | 149 | 151 | 155 | 157 | 158 => 0.4,
            134 => 0.3,
            140 => 0.45000002,
            156 => 0.5,
            163 => 1.225,
            164 => 1.2125001,
            _ => 0.2,
        }
    }

    pub const fn getTemperature(self) -> f32 {
        self.temperature
    }
    pub const fn isSnowyBiome(self) -> bool {
        matches!(self.id, 10 | 11 | 12 | 13 | 26 | 30 | 31 | 140 | 158)
    }
    pub const fn isMutation(self) -> bool {
        matches!(
            self.id,
            129 | 130
                | 131
                | 132
                | 133
                | 134
                | 140
                | 149
                | 151
                | 155
                | 156
                | 157
                | 158
                | 160
                | 161
                | 162
                | 163
                | 164
                | 165
                | 166
                | 167
        )
    }
    pub fn getMutationForBiome(self) -> Option<Self> {
        Some(Self::getBiome(match self.id {
            1 => 129,
            2 => 130,
            3 => 131,
            4 => 132,
            5 => 133,
            6 => 134,
            12 => 140,
            21 => 149,
            23 => 151,
            27 => 155,
            28 => 156,
            29 => 157,
            30 => 158,
            32 => 160,
            33 => 161,
            34 => 162,
            35 => 163,
            36 => 164,
            37 => 165,
            38 => 166,
            39 => 167,
            _ => return None,
        }))
    }
    pub const fn getBiomeClass(self) -> BiomeClass {
        match self.id {
            0 | 10 | 24 => BiomeClass::Ocean,
            1 | 129 => BiomeClass::Plains,
            2 | 17 | 130 => BiomeClass::Desert,
            3 | 20 | 34 | 131 | 162 => BiomeClass::Hills,
            4 | 18 | 27 | 28 | 29 | 132 | 157 => BiomeClass::Forest,
            155 | 156 => BiomeClass::Forest,
            5 | 19 | 30 | 31 | 32 | 33 | 133 | 158 | 160 | 161 => BiomeClass::Taiga,
            6 | 134 => BiomeClass::Swamp,
            7 | 11 => BiomeClass::River,
            8 => BiomeClass::Hell,
            9 => BiomeClass::End,
            12 | 13 | 140 => BiomeClass::Snow,
            14 | 15 => BiomeClass::MushroomIsland,
            16 | 26 => BiomeClass::Beach,
            21 | 22 | 23 | 149 | 151 => BiomeClass::Jungle,
            25 => BiomeClass::StoneBeach,
            35 | 36 => BiomeClass::Savanna,
            163 | 164 => BiomeClass::Savanna,
            37 | 38 | 39 | 165 | 166 | 167 => BiomeClass::Mesa,
            127 => BiomeClass::Void,
            _ => BiomeClass::Ocean,
        }
    }
    pub const fn getTempCategory(self) -> TempCategory {
        if self.temperature < 0.2 {
            TempCategory::Cold
        } else if self.temperature < 1.0 {
            TempCategory::Medium
        } else {
            TempCategory::Warm
        }
    }
    pub const fn isMesa(self) -> bool {
        matches!(self.getBiomeClass(), BiomeClass::Mesa)
    }
    pub const fn isJungleClass(self) -> bool {
        matches!(self.getBiomeClass(), BiomeClass::Jungle)
    }

    pub const fn getRainfall(self) -> f32 {
        self.rainfall
    }
    pub const fn getWaterColor(self) -> i32 {
        self.waterColor
    }
    pub const fn ignorePlayerSpawnSuitability(self) -> bool {
        self.id == 127
    }

    /// Source constructor defaults/overrides for `Biome#topBlock` and
    /// `Biome#fillerBlock`. Special biome subclasses may change these for a
    /// single terrain column before delegating to `generateBiomeTerrain`.
    pub const fn terrainTopFiller(self) -> (IBlockState, IBlockState) {
        const GRASS: IBlockState = IBlockState::fromGlobalStateId(2 << 4);
        const DIRT: IBlockState = IBlockState::fromGlobalStateId(3 << 4);
        const SAND: IBlockState = IBlockState::fromGlobalStateId(12 << 4);
        const STONE: IBlockState = IBlockState::fromGlobalStateId(1 << 4);
        const MYCELIUM: IBlockState = IBlockState::fromGlobalStateId(110 << 4);
        const SNOW: IBlockState = IBlockState::fromGlobalStateId(80 << 4);
        const RED_SAND: IBlockState = IBlockState::fromGlobalStateId((12 << 4) | 1);
        const STAINED_HARDENED_CLAY: IBlockState = IBlockState::fromGlobalStateId(159 << 4);
        match self.id {
            2 | 17 | 130 | 16 | 26 => (SAND, SAND),
            25 => (STONE, STONE),
            14 | 15 => (MYCELIUM, DIRT),
            140 => (SNOW, DIRT),
            37 | 38 | 39 | 165 | 166 | 167 => (RED_SAND, STAINED_HARDENED_CLAY),
            _ => (GRASS, DIRT),
        }
    }

    /// Exact base `Biome#generateBiomeTerrain` column replacement algorithm.
    ///
    /// The source method intentionally uses `(z & 15)` as the ChunkPrimer X
    /// coordinate and `(x & 15)` as its Z coordinate; that orientation is
    /// retained rather than normalized for Rust convenience.
    #[allow(clippy::too_many_arguments)]
    pub fn generateBiomeTerrain(
        self,
        sea_level: i32,
        rand: &mut JavaRandom,
        primer: &mut ChunkPrimer,
        x: i32,
        z: i32,
        noise: f64,
        top_in: IBlockState,
        filler_in: IBlockState,
    ) {
        const AIR: IBlockState = IBlockState::fromGlobalStateId(0);
        const STONE: IBlockState = IBlockState::fromGlobalStateId(1 << 4);
        const BEDROCK: IBlockState = IBlockState::fromGlobalStateId(7 << 4);
        const WATER: IBlockState = IBlockState::fromGlobalStateId(9 << 4);
        const ICE: IBlockState = IBlockState::fromGlobalStateId(79 << 4);
        const GRAVEL: IBlockState = IBlockState::fromGlobalStateId(13 << 4);
        const SANDSTONE: IBlockState = IBlockState::fromGlobalStateId(24 << 4);
        const RED_SANDSTONE: IBlockState = IBlockState::fromGlobalStateId(179 << 4);
        let mut top = top_in;
        let mut filler = filler_in;
        let mut remaining = -1_i32;
        let thickness = (noise / 3.0 + 3.0 + rand.next_f64() * 0.25) as i32;
        let local_z = (x & 15) as usize;
        let local_x = (z & 15) as usize;
        for y in (0..=255_i32).rev() {
            if y <= rand.next_i32_bound(5) {
                primer.setBlockState(local_x, y as usize, local_z, BEDROCK);
                continue;
            }
            let state = primer.getBlockState(local_x, y as usize, local_z);
            if state.isAir() {
                remaining = -1;
                continue;
            }
            if state.getBlockId() != 1 {
                continue;
            }
            if remaining == -1 {
                if thickness <= 0 {
                    top = AIR;
                    filler = STONE;
                } else if y >= sea_level - 4 && y <= sea_level + 1 {
                    top = top_in;
                    filler = filler_in;
                }
                if y < sea_level && top.isAir() {
                    top = if self.getFloatTemperature(BlockPos::new(x, y, z)) < 0.15 {
                        ICE
                    } else {
                        WATER
                    };
                }
                remaining = thickness;
                if y >= sea_level - 1 {
                    primer.setBlockState(local_x, y as usize, local_z, top);
                } else if y < sea_level - 7 - thickness {
                    top = AIR;
                    filler = STONE;
                    primer.setBlockState(local_x, y as usize, local_z, GRAVEL);
                } else {
                    primer.setBlockState(local_x, y as usize, local_z, filler);
                }
            } else if remaining > 0 {
                remaining -= 1;
                primer.setBlockState(local_x, y as usize, local_z, filler);
                if remaining == 0 && filler.getBlockId() == 12 && thickness > 1 {
                    remaining = rand.next_i32_bound(4) + (y - 63).max(0);
                    filler = if filler.getMetadata() == 1 {
                        RED_SANDSTONE
                    } else {
                        SANDSTONE
                    };
                }
            }
        }
    }

    /// MCP 1.12.2 `Biome#getSkyColorByTemp`.
    pub fn getSkyColorByTemp(self, currentTemperature: f32) -> i32 {
        let temperature = (currentTemperature / 3.0).clamp(-1.0, 1.0);
        hsv_to_rgb(
            0.622_222_24 - temperature * 0.05,
            0.5 + temperature * 0.1,
            1.0,
        )
    }

    /// Source-equivalent base branch of `Biome.getFloatTemperature`.
    /// The high-altitude `TEMPERATURE_NOISE` perturbation follows the source
    /// `NoiseGeneratorPerlin` path; below y=65 vanilla returns the base value.
    pub fn getFloatTemperature(self, pos: BlockPos) -> f32 {
        if pos.y <= 64 {
            self.temperature
        } else {
            let noise = temperature_noise().getValue(pos.x as f64 / 8.0, pos.z as f64 / 8.0);
            let perturbation = (noise * 4.0) as f32;
            self.temperature - (perturbation + pos.y as f32 - 64.0) * 0.05 / 30.0
        }
    }

    pub fn getGrassColorAtPos(self, pos: BlockPos, grass: &ColorizerGrass) -> i32 {
        match self.colorKind {
            BiomeColorKind::Swamp => {
                let noise =
                    grass_color_noise().getValue(pos.x as f64 * 0.0225, pos.z as f64 * 0.0225);
                if noise < -0.1 {
                    5_011_004
                } else {
                    6_975_545
                }
            }
            BiomeColorKind::Mesa => 9_470_285,
            BiomeColorKind::RoofedForest => {
                let base = grass.getGrassColor(
                    self.getFloatTemperature(pos).clamp(0.0, 1.0) as f64,
                    self.rainfall.clamp(0.0, 1.0) as f64,
                );
                ((base & 16_711_422) + 2_634_762) >> 1
            }
            BiomeColorKind::Default => grass.getGrassColor(
                self.getFloatTemperature(pos).clamp(0.0, 1.0) as f64,
                self.rainfall.clamp(0.0, 1.0) as f64,
            ),
        }
    }

    pub fn getFoliageColorAtPos(self, pos: BlockPos, foliage: &ColorizerFoliage) -> i32 {
        match self.colorKind {
            BiomeColorKind::Swamp => 6_975_545,
            BiomeColorKind::Mesa => 10_387_789,
            _ => foliage.getFoliageColor(
                self.getFloatTemperature(pos).clamp(0.0, 1.0) as f64,
                self.rainfall.clamp(0.0, 1.0) as f64,
            ),
        }
    }
}

fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> i32 {
    let sector = (hue * 6.0) as i32 % 6;
    let fraction = hue * 6.0 - sector as f32;
    let minimum = value * (1.0 - saturation);
    let descending = value * (1.0 - fraction * saturation);
    let ascending = value * (1.0 - (1.0 - fraction) * saturation);
    let (red, green, blue) = match sector {
        0 => (value, ascending, minimum),
        1 => (descending, value, minimum),
        2 => (minimum, value, ascending),
        3 => (minimum, descending, value),
        4 => (ascending, minimum, value),
        5 => (value, minimum, descending),
        _ => unreachable!("hue sector must be 0..5"),
    };
    let red = (red * 255.0) as i32;
    let green = (green * 255.0) as i32;
    let blue = (blue * 255.0) as i32;
    red.clamp(0, 255) << 16 | green.clamp(0, 255) << 8 | blue.clamp(0, 255)
}

static TEMPERATURE_NOISE: OnceLock<NoiseGeneratorPerlin> = OnceLock::new();
static GRASS_COLOR_NOISE: OnceLock<NoiseGeneratorPerlin> = OnceLock::new();

fn temperature_noise() -> &'static NoiseGeneratorPerlin {
    TEMPERATURE_NOISE.get_or_init(|| {
        let mut random = JavaRandom::new(1234);
        NoiseGeneratorPerlin::new(&mut random, 1)
    })
}

pub(crate) fn grass_color_noise() -> &'static NoiseGeneratorPerlin {
    GRASS_COLOR_NOISE.get_or_init(|| {
        let mut random = JavaRandom::new(2345);
        NoiseGeneratorPerlin::new(&mut random, 1)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_biome_parameters_match_source_values() {
        let plains = Biome::getBiome(1);
        assert_eq!(plains.getRainfall(), 0.4);
        let swamp = Biome::getBiome(6);
        assert_eq!(swamp.getWaterColor(), 14_745_518);
        assert_eq!(
            Biome::getBiome(35).getFloatTemperature(BlockPos::ORIGIN),
            1.2
        );
    }

    #[test]
    fn generation_metadata_matches_mcp_registry_and_class_overrides() {
        assert_eq!(Biome::getBiome(28).getBaseHeight(), 0.45);
        assert_eq!(Biome::getBiome(28).getHeightVariation(), 0.3);
        assert_eq!(Biome::getBiome(0).getTempCategory(), TempCategory::Medium);
        assert_eq!(Biome::getBiome(10).getTempCategory(), TempCategory::Cold);
        assert_eq!(Biome::getBiome(155).getBiomeClass(), BiomeClass::Forest);
        assert_eq!(Biome::getBiome(156).getBiomeClass(), BiomeClass::Forest);
        assert_eq!(Biome::getBiome(163).getBiomeClass(), BiomeClass::Savanna);
        assert_eq!(Biome::getBiome(164).getBiomeClass(), BiomeClass::Savanna);
    }

    #[test]
    fn sky_color_uses_source_temperature_hsv_formula() {
        assert_eq!(Biome::getBiome(1).getSkyColorByTemp(0.8), 7_907_327);
    }
}
