use crate::compat::Java::JavaRandom;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::world::biome::Biome::Biome;
use crate::net::minecraft::world::biome::BiomeCache::BiomeCache;
use crate::net::minecraft::world::gen::layer::GenLayer::{initializeAllBiomeGenerators, Layer};
use crate::net::minecraft::world::gen::ChunkGeneratorSettings::{ChunkGeneratorSettings, Factory};
use crate::net::minecraft::world::WorldType::WorldType;

/// MCP 1.12.2 `BiomeProvider` backed by the exact GenLayer graph.
#[derive(Debug, Clone)]
pub struct BiomeProvider {
    settings: Option<ChunkGeneratorSettings>,
    genBiomes: Layer,
    biomeIndexLayer: Layer,
    biomeCache: BiomeCache,
    biomesToSpawnIn: [Biome; 7],
}
impl BiomeProvider {
    pub fn new(seed: i64, worldType: WorldType, options: &str) -> Self {
        let settings = if worldType == WorldType::Customized && !options.is_empty() {
            Some(Factory::jsonToFactory(options).build())
        } else {
            None
        };
        let set = initializeAllBiomeGenerators(seed, worldType, settings.as_ref());
        Self {
            settings,
            genBiomes: set.genBiomes,
            biomeIndexLayer: set.biomeIndexLayer,
            biomeCache: BiomeCache::new(),
            biomesToSpawnIn: [
                Biome::getBiome(4),
                Biome::getBiome(1),
                Biome::getBiome(5),
                Biome::getBiome(19),
                Biome::getBiome(18),
                Biome::getBiome(21),
                Biome::getBiome(22),
            ],
        }
    }
    pub fn getBiomesToSpawnIn(&self) -> [Biome; 7] {
        self.biomesToSpawnIn
    }
    pub fn getBiome(&self, pos: BlockPos) -> Biome {
        let layer = self.biomeIndexLayer.clone();
        self.biomeCache
            .getBiome(pos.x, pos.z, Biome::getBiome(0), move |x, z| {
                Self::layerBiomes(&layer, x, z, 16, 16)
            })
    }
    pub fn getTemperatureAtHeight(&self, temp: f32, _height: i32) -> f32 {
        temp
    }
    fn layerBiomes(layer: &Layer, x: i32, z: i32, w: i32, h: i32) -> Vec<Biome> {
        layer
            .lock()
            .unwrap()
            .getInts(x, z, w, h)
            .into_iter()
            .map(|id| Biome::getBiomeForId(id).unwrap_or(Biome::getBiome(0)))
            .collect()
    }
    pub fn getBiomesForGeneration(
        &self,
        mut reuse: Option<Vec<Biome>>,
        x: i32,
        z: i32,
        w: i32,
        h: i32,
    ) -> Vec<Biome> {
        let count = (w * h) as usize;
        let ids = self.genBiomes.lock().unwrap().getInts(x, z, w, h);
        let mut out = reuse.take().unwrap_or_default();
        if out.len() < count {
            out.resize(count, Biome::getBiome(0));
        }
        for i in 0..count {
            out[i] = Biome::getBiomeForId(ids[i]).unwrap_or(Biome::getBiome(0));
        }
        out
    }
    pub fn getBiomes(
        &self,
        reuse: Option<Vec<Biome>>,
        x: i32,
        z: i32,
        w: i32,
        h: i32,
    ) -> Vec<Biome> {
        self.getBiomesCached(reuse, x, z, w, h, true)
    }
    pub fn getBiomesCached(
        &self,
        mut reuse: Option<Vec<Biome>>,
        x: i32,
        z: i32,
        w: i32,
        h: i32,
        cacheFlag: bool,
    ) -> Vec<Biome> {
        let count = (w * h) as usize;
        let mut out = reuse.take().unwrap_or_default();
        if out.len() < count {
            out.resize(count, Biome::getBiome(0));
        }
        if cacheFlag && w == 16 && h == 16 && (x & 15) == 0 && (z & 15) == 0 {
            let layer = self.biomeIndexLayer.clone();
            let cached = self.biomeCache.getCachedBiomes(x, z, move |cx, cz| {
                Self::layerBiomes(&layer, cx, cz, 16, 16)
            });
            out[..count].copy_from_slice(&cached[..count]);
            return out;
        }
        let ids = self.biomeIndexLayer.lock().unwrap().getInts(x, z, w, h);
        for i in 0..count {
            out[i] = Biome::getBiomeForId(ids[i]).unwrap_or(Biome::getBiome(0));
        }
        out
    }
    pub fn areBiomesViable(&self, x: i32, z: i32, radius: i32, allowed: &[Biome]) -> bool {
        let minx = (x - radius) >> 2;
        let minz = (z - radius) >> 2;
        let maxx = (x + radius) >> 2;
        let maxz = (z + radius) >> 2;
        let w = maxx - minx + 1;
        let h = maxz - minz + 1;
        self.genBiomes
            .lock()
            .unwrap()
            .getInts(minx, minz, w, h)
            .into_iter()
            .all(|id| Biome::getBiomeForId(id).is_some_and(|b| allowed.contains(&b)))
    }
    pub fn findBiomePosition(
        &self,
        x: i32,
        z: i32,
        range: i32,
        biomes: &[Biome],
        random: &mut JavaRandom,
    ) -> Option<BlockPos> {
        let minx = (x - range) >> 2;
        let minz = (z - range) >> 2;
        let maxx = (x + range) >> 2;
        let maxz = (z + range) >> 2;
        let w = maxx - minx + 1;
        let h = maxz - minz + 1;
        let ids = self.genBiomes.lock().unwrap().getInts(minx, minz, w, h);
        let mut found = None;
        let mut count = 0;
        for (index, id) in ids.into_iter().enumerate() {
            let bx = (minx + (index as i32) % w) << 2;
            let bz = (minz + (index as i32) / w) << 2;
            if Biome::getBiomeForId(id).is_some_and(|b| biomes.contains(&b))
                && (found.is_none() || random.next_i32_bound(count + 1) == 0)
            {
                found = Some(BlockPos::new(bx, 0, bz));
                count += 1;
            }
        }
        found
    }
    pub fn cleanupCache(&self) {
        self.biomeCache.cleanupCache()
    }
    pub fn func_190944_c(&self) -> bool {
        self.settings.as_ref().is_some_and(|s| s.fixedBiome >= 0)
    }
    pub fn func_190943_d(&self) -> Option<Biome> {
        self.settings
            .as_ref()
            .filter(|s| s.fixedBiome >= 0)
            .and_then(|s| Biome::getBiomeForId(s.fixedBiome))
    }
    pub fn settings(&self) -> Option<&ChunkGeneratorSettings> {
        self.settings.as_ref()
    }
}
