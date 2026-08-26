use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::net::minecraft::world::biome::Biome::Biome;
use crate::net::minecraft::world::gen::ChunkGeneratorSettings::ChunkGeneratorSettings;
use crate::net::minecraft::world::WorldType::WorldType;

const MULTIPLIER: i64 = 6_364_136_223_846_793_005;
const ADDEND: i64 = 1_442_695_040_888_963_407;

pub trait GenLayer: Send + Debug {
    fn initWorldGenSeed(&mut self, seed: i64);
    fn getInts(&mut self, areaX: i32, areaY: i32, areaWidth: i32, areaHeight: i32) -> Vec<i32>;
}

pub type Layer = Arc<Mutex<Box<dyn GenLayer>>>;

pub fn layer<T: GenLayer + 'static>(value: T) -> Layer {
    Arc::new(Mutex::new(Box::new(value)))
}

/// The three entries returned by MCP `initializeAllBiomeGenerators`. The first
/// and third intentionally share the exact RiverMix object, as the Java array
/// stores the same reference twice.
#[derive(Clone)]
pub struct GenLayerSet {
    pub genBiomes: Layer,
    pub biomeIndexLayer: Layer,
    pub riverMix: Layer,
}

#[derive(Debug, Clone, Copy)]
pub struct GenLayerSeed {
    baseSeed: i64,
    worldGenSeed: i64,
    chunkSeed: i64,
}

impl GenLayerSeed {
    pub fn new(seed: i64) -> Self {
        let mut base = seed;
        for _ in 0..3 {
            base = base.wrapping_mul(base.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND));
            base = base.wrapping_add(seed);
        }
        Self {
            baseSeed: base,
            worldGenSeed: 0,
            chunkSeed: 0,
        }
    }

    pub fn initWorldGenSeed(&mut self, seed: i64) {
        self.worldGenSeed = seed;
        for _ in 0..3 {
            self.worldGenSeed = self.worldGenSeed.wrapping_mul(
                self.worldGenSeed
                    .wrapping_mul(MULTIPLIER)
                    .wrapping_add(ADDEND),
            );
            self.worldGenSeed = self.worldGenSeed.wrapping_add(self.baseSeed);
        }
    }

    pub fn initChunkSeed(&mut self, x: i64, z: i64) {
        self.chunkSeed = self.worldGenSeed;
        for add in [x, z, x, z] {
            self.chunkSeed = self
                .chunkSeed
                .wrapping_mul(self.chunkSeed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND));
            self.chunkSeed = self.chunkSeed.wrapping_add(add);
        }
    }

    pub fn nextInt(&mut self, bound: i32) -> i32 {
        debug_assert!(bound > 0);
        let mut value = ((self.chunkSeed >> 24) % bound as i64) as i32;
        if value < 0 {
            value += bound;
        }
        self.chunkSeed = self
            .chunkSeed
            .wrapping_mul(self.chunkSeed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND));
        self.chunkSeed = self.chunkSeed.wrapping_add(self.worldGenSeed);
        value
    }

    pub fn selectRandom(&mut self, values: &[i32]) -> i32 {
        values[self.nextInt(values.len() as i32) as usize]
    }
    pub fn selectModeOrRandom(&mut self, a: i32, b: i32, c: i32, d: i32) -> i32 {
        if b == c && c == d {
            b
        } else if a == b && a == c {
            a
        } else if a == b && a == d {
            a
        } else if a == c && a == d {
            a
        } else if a == b && c != d {
            a
        } else if a == c && b != d {
            a
        } else if a == d && b != c {
            a
        } else if b == c && a != d {
            b
        } else if b == d && a != c {
            b
        } else if c == d && a != b {
            c
        } else {
            self.selectRandom(&[a, b, c, d])
        }
    }
}

pub fn isBiomeOceanic(id: i32) -> bool {
    matches!(id, 0 | 10 | 24)
}

pub fn biomesEqualOrMesaPlateau(a: i32, b: i32) -> bool {
    if a == b {
        return true;
    }
    let (Some(ba), Some(bb)) = (Biome::getBiomeForId(a), Biome::getBiomeForId(b)) else {
        return false;
    };
    if !matches!(a, 38 | 39) {
        ba.getBiomeClass() == bb.getBiomeClass()
    } else {
        matches!(b, 38 | 39)
    }
}

/// MCP `GenLayer#initializeAllBiomeGenerators` is assembled here after every
/// concrete layer module, preserving the source seed/order graph.
pub fn initializeAllBiomeGenerators(
    seed: i64,
    worldType: WorldType,
    settings: Option<&ChunkGeneratorSettings>,
) -> GenLayerSet {
    use super::GenLayerAddIsland::GenLayerAddIsland;
    use super::GenLayerAddMushroomIsland::GenLayerAddMushroomIsland;
    use super::GenLayerAddSnow::GenLayerAddSnow;
    use super::GenLayerBiome::GenLayerBiome;
    use super::GenLayerBiomeEdge::GenLayerBiomeEdge;
    use super::GenLayerDeepOcean::GenLayerDeepOcean;
    use super::GenLayerEdge::{GenLayerEdge, Mode};
    use super::GenLayerFuzzyZoom::GenLayerFuzzyZoom;
    use super::GenLayerHills::GenLayerHills;
    use super::GenLayerIsland::GenLayerIsland;
    use super::GenLayerRareBiome::GenLayerRareBiome;
    use super::GenLayerRemoveTooMuchOcean::GenLayerRemoveTooMuchOcean;
    use super::GenLayerRiver::GenLayerRiver;
    use super::GenLayerRiverInit::GenLayerRiverInit;
    use super::GenLayerRiverMix::GenLayerRiverMix;
    use super::GenLayerShore::GenLayerShore;
    use super::GenLayerSmooth::GenLayerSmooth;
    use super::GenLayerVoronoiZoom::GenLayerVoronoiZoom;
    use super::GenLayerZoom::{magnify, GenLayerZoom};

    let mut g = layer(GenLayerIsland::new(1));
    g = layer(GenLayerFuzzyZoom::new(2000, g));
    let add_island = layer(GenLayerAddIsland::new(1, g));
    let zoom = layer(GenLayerZoom::new(2001, add_island));
    let mut add = layer(GenLayerAddIsland::new(2, zoom));
    add = layer(GenLayerAddIsland::new(50, add));
    add = layer(GenLayerAddIsland::new(70, add));
    let remove_ocean = layer(GenLayerRemoveTooMuchOcean::new(2, add));
    let snow = layer(GenLayerAddSnow::new(2, remove_ocean));
    let add2 = layer(GenLayerAddIsland::new(3, snow));
    let mut edge = layer(GenLayerEdge::new(2, add2, Mode::CoolWarm));
    edge = layer(GenLayerEdge::new(2, edge, Mode::HeatIce));
    edge = layer(GenLayerEdge::new(3, edge, Mode::Special));
    let mut zoom1 = layer(GenLayerZoom::new(2002, edge));
    zoom1 = layer(GenLayerZoom::new(2003, zoom1));
    let add3 = layer(GenLayerAddIsland::new(4, zoom1));
    let mushroom = layer(GenLayerAddMushroomIsland::new(5, add3));
    let deep = layer(GenLayerDeepOcean::new(4, mushroom));
    let gen4 = magnify(1000, deep, 0);
    let mut biome_size = settings.map(|s| s.biomeSize).unwrap_or(4);
    let river_size = settings.map(|s| s.riverSize).unwrap_or(4);
    if worldType == WorldType::LargeBiomes {
        biome_size = 6;
    }

    let river_root = magnify(1000, gen4.clone(), 0);
    let river_init = layer(GenLayerRiverInit::new(100, river_root));
    let biome = layer(GenLayerBiome::new(200, gen4, worldType, settings.cloned()));
    let biome_zoom = magnify(1000, biome, 2);
    let biome_edge = layer(GenLayerBiomeEdge::new(1000, biome_zoom));
    let hills_river = magnify(1000, river_init.clone(), 2);
    let mut hills = layer(GenLayerHills::new(1000, biome_edge, hills_river));
    let mut river = magnify(1000, river_init, 2);
    river = magnify(1000, river, river_size);
    let river = layer(GenLayerRiver::new(1, river));
    let river_smooth = layer(GenLayerSmooth::new(1000, river));
    hills = layer(GenLayerRareBiome::new(1001, hills));
    for k in 0..biome_size {
        hills = layer(GenLayerZoom::new((1000 + k) as i64, hills));
        if k == 0 {
            hills = layer(GenLayerAddIsland::new(3, hills));
        }
        if k == 1 || biome_size == 1 {
            hills = layer(GenLayerShore::new(1000, hills));
        }
    }
    let biome_smooth = layer(GenLayerSmooth::new(1000, hills));
    let river_mix = layer(GenLayerRiverMix::new(100, biome_smooth, river_smooth));
    let voronoi = layer(GenLayerVoronoiZoom::new(10, river_mix.clone()));
    river_mix
        .lock()
        .expect("river mix layer poisoned")
        .initWorldGenSeed(seed);
    voronoi
        .lock()
        .expect("voronoi layer poisoned")
        .initWorldGenSeed(seed);
    GenLayerSet {
        genBiomes: river_mix.clone(),
        biomeIndexLayer: voronoi,
        riverMix: river_mix,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn java_lcg_seed_path_is_stable() {
        let mut s = GenLayerSeed::new(1);
        s.initWorldGenSeed(12345);
        s.initChunkSeed(-7, 19);
        assert_eq!(s.nextInt(10), 8);
        assert_eq!(s.nextInt(10), 0);
    }
}
