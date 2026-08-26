use crate::compat::Java::JavaRandom;
use crate::net::minecraft::world::gen::NoiseGenerator::NoiseGenerator;
use crate::net::minecraft::world::gen::NoiseGeneratorImproved::NoiseGeneratorImproved;

/// Exact Rust port of MCP 1.12.2 `NoiseGeneratorOctaves`.
#[derive(Debug, Clone)]
pub struct NoiseGeneratorOctaves {
    generatorCollection: Vec<NoiseGeneratorImproved>,
    octaves: i32,
}

impl NoiseGenerator for NoiseGeneratorOctaves {}

impl NoiseGeneratorOctaves {
    pub fn new(seed: &mut JavaRandom, octavesIn: i32) -> Self {
        assert!(octavesIn >= 0, "octaves must be non-negative");
        let mut generatorCollection = Vec::with_capacity(octavesIn as usize);
        for _ in 0..octavesIn {
            generatorCollection.push(NoiseGeneratorImproved::new(seed));
        }
        Self {
            generatorCollection,
            octaves: octavesIn,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn generateNoiseOctaves(
        &self,
        noiseArray: Option<Vec<f64>>,
        xOffset: i32,
        yOffset: i32,
        zOffset: i32,
        xSize: i32,
        ySize: i32,
        zSize: i32,
        xScale: f64,
        yScale: f64,
        zScale: f64,
    ) -> Vec<f64> {
        let len = (xSize as usize)
            .saturating_mul(ySize as usize)
            .saturating_mul(zSize as usize);
        let mut noiseArray = match noiseArray {
            Some(mut array) => {
                array.fill(0.0);
                array
            }
            None => vec![0.0; len],
        };
        assert!(
            noiseArray.len() >= len,
            "noiseArray must hold xSize*ySize*zSize entries"
        );
        let mut d3 = 1.0_f64;
        for j in 0..self.octaves as usize {
            let mut d0 = xOffset as f64 * d3 * xScale;
            let d1 = yOffset as f64 * d3 * yScale;
            let mut d2 = zOffset as f64 * d3 * zScale;
            let mut k = d0.floor() as i64;
            let mut l = d2.floor() as i64;
            d0 -= k as f64;
            d2 -= l as f64;
            k %= 16_777_216_i64;
            l %= 16_777_216_i64;
            d0 += k as f64;
            d2 += l as f64;
            self.generatorCollection[j].populateNoiseArray(
                &mut noiseArray,
                d0,
                d1,
                d2,
                xSize,
                ySize,
                zSize,
                xScale * d3,
                yScale * d3,
                zScale * d3,
                d3,
            );
            d3 /= 2.0;
        }
        noiseArray
    }

    pub fn generateNoiseOctaves2D(
        &self,
        noiseArray: Option<Vec<f64>>,
        xOffset: i32,
        zOffset: i32,
        xSize: i32,
        zSize: i32,
        xScale: f64,
        zScale: f64,
        _unused: f64,
    ) -> Vec<f64> {
        self.generateNoiseOctaves(
            noiseArray, xOffset, 10, zOffset, xSize, 1, zSize, xScale, 1.0, zScale,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn octave_output_matches_mcp_java_golden_values() {
        let mut random = JavaRandom::new(12345);
        let octaves = NoiseGeneratorOctaves::new(&mut random, 4);
        let values = octaves.generateNoiseOctaves(None, -13, 5, 21, 3, 2, 4, 0.01, 0.02, 0.03);
        let expected = [
            1.7516236890156591,
            1.7769599623375014,
            1.6505084109106467,
            1.6757591021805118,
            1.5470136542184412,
            1.57215814052118,
            1.442603149939369,
            1.467623040458113,
            1.7643660532617234,
            1.789696932027535,
            1.6635893328820925,
            1.6888346356629746,
            1.5603898890171708,
            1.5855288433031207,
            1.4562460334121279,
            1.4812601500509834,
            1.7768225678256317,
            1.8021471060608398,
            1.6763907407085323,
            1.7016297063402948,
            1.5734951677151265,
            1.5986276412560871,
            1.4696278549873159,
            1.4946352506417204,
        ];
        assert_eq!(values.len(), expected.len());
        for (actual, expected) in values.iter().zip(expected) {
            assert!(
                (actual - expected).abs() < 1.0e-14,
                "{actual} != {expected}"
            );
        }
    }
}
