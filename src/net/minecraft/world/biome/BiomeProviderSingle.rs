use crate::compat::Java::JavaRandom;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::world::biome::Biome::Biome;

/// MCP 1.12.2 `BiomeProviderSingle`.
///
/// This is intentionally a concrete standalone provider rather than a fake
/// implementation of the still-pending `BiomeProvider`/GenLayer hierarchy.
/// Vanilla uses it for Nether, End, flat-world and debug-world fixed-biome
/// providers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiomeProviderSingle {
    biome: Biome,
}

impl BiomeProviderSingle {
    pub const fn new(biome: Biome) -> Self {
        Self { biome }
    }

    /// MCP `BiomeProviderSingle#getBiome`.
    pub const fn getBiome(&self, _pos: BlockPos) -> Biome {
        self.biome
    }

    /// Rust ownership equivalent of MCP `getBiomesForGeneration`.
    ///
    /// Java reuses the caller array when it is large enough. Rust preserves
    /// that observable reuse/content contract by accepting an optional Vec,
    /// resizing only when capacity/length is insufficient, and filling exactly
    /// `width * height` entries.
    pub fn getBiomesForGeneration(
        &self,
        biomes: Option<Vec<Biome>>,
        _x: i32,
        _z: i32,
        width: i32,
        height: i32,
    ) -> Vec<Biome> {
        self.fillBiomes(biomes, width, height)
    }

    /// MCP four-argument `getBiomes` overload.
    pub fn getBiomes(
        &self,
        oldBiomeList: Option<Vec<Biome>>,
        _x: i32,
        _z: i32,
        width: i32,
        depth: i32,
    ) -> Vec<Biome> {
        self.fillBiomes(oldBiomeList, width, depth)
    }

    /// MCP cached `getBiomes` overload. `cacheFlag` is deliberately ignored by
    /// the vanilla single-biome implementation.
    pub fn getBiomesCached(
        &self,
        listToReuse: Option<Vec<Biome>>,
        x: i32,
        z: i32,
        width: i32,
        length: i32,
        _cacheFlag: bool,
    ) -> Vec<Biome> {
        self.getBiomes(listToReuse, x, z, width, length)
    }

    /// Inherited MCP `BiomeProvider#getBiomesToSpawnIn` list.
    pub fn getBiomesToSpawnIn(&self) -> [Biome; 7] {
        [
            Biome::getBiome(4),  // forest
            Biome::getBiome(1),  // plains
            Biome::getBiome(5),  // taiga
            Biome::getBiome(19), // taiga hills
            Biome::getBiome(18), // forest hills
            Biome::getBiome(21), // jungle
            Biome::getBiome(22), // jungle hills
        ]
    }

    /// MCP `findBiomePosition` using the project's exact `java.util.Random`
    /// equivalent. Java int wrapping is retained for the coordinate arithmetic.
    pub fn findBiomePosition(
        &self,
        x: i32,
        z: i32,
        range: i32,
        biomes: &[Biome],
        random: &mut JavaRandom,
    ) -> Option<BlockPos> {
        if !biomes.contains(&self.biome) {
            return None;
        }
        let bound = range.wrapping_mul(2).wrapping_add(1);
        if bound <= 0 {
            // java.util.Random#nextInt would throw for a non-positive bound.
            panic!("bound must be positive");
        }
        Some(BlockPos::new(
            x.wrapping_sub(range)
                .wrapping_add(random.next_i32_bound(bound)),
            0,
            z.wrapping_sub(range)
                .wrapping_add(random.next_i32_bound(bound)),
        ))
    }

    /// MCP `areBiomesViable`.
    pub fn areBiomesViable(
        &self,
        _x: i32,
        _z: i32,
        _radius: i32,
        allowed: &[Biome],
    ) -> bool {
        allowed.contains(&self.biome)
    }

    /// MCP SRG `func_190944_c`.
    pub const fn func_190944_c(&self) -> bool {
        true
    }

    /// MCP SRG `func_190943_d`.
    pub const fn func_190943_d(&self) -> Biome {
        self.biome
    }

    fn fillBiomes(&self, mut reuse: Option<Vec<Biome>>, width: i32, height: i32) -> Vec<Biome> {
        let count = width
            .checked_mul(height)
            .and_then(|value| usize::try_from(value).ok())
            .expect("BiomeProviderSingle dimensions overflow");
        let mut biomes = reuse.take().unwrap_or_default();
        if biomes.len() < count {
            biomes.resize(count, self.biome);
        }
        biomes[..count].fill(self.biome);
        biomes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_requested_region_with_single_biome_and_reuses_tail() {
        let provider = BiomeProviderSingle::new(Biome::getBiome(8));
        let reuse = vec![Biome::getBiome(1); 6];
        let result = provider.getBiomes(Some(reuse), 0, 0, 2, 2);
        assert_eq!(result.len(), 6);
        assert!(result[..4].iter().all(|biome| biome.getId() == 8));
        // Arrays.fill(source, 0, width*depth, biome) leaves any reusable tail.
        assert_eq!(result[4].getId(), 1);
        assert_eq!(result[5].getId(), 1);
    }

    #[test]
    fn biome_lookup_and_viability_match_source_contract() {
        let end = Biome::getBiome(9);
        let provider = BiomeProviderSingle::new(end);
        assert_eq!(provider.getBiome(BlockPos::new(123, 70, -44)), end);
        assert!(provider.areBiomesViable(0, 0, 999, &[Biome::getBiome(1), end]));
        assert!(!provider.areBiomesViable(0, 0, 999, &[Biome::getBiome(1)]));
        assert!(provider.func_190944_c());
        assert_eq!(provider.func_190943_d(), end);
    }

    #[test]
    fn find_biome_position_uses_java_random_sequence() {
        let plains = Biome::getBiome(1);
        let provider = BiomeProviderSingle::new(plains);
        let mut random = JavaRandom::new(12345);
        assert_eq!(
            provider.findBiomePosition(100, -50, 10, &[plains], &mut random),
            Some(BlockPos::new(109, 0, -44))
        );
        assert_eq!(
            provider.findBiomePosition(100, -50, 10, &[Biome::getBiome(2)], &mut random),
            None
        );
    }
}
