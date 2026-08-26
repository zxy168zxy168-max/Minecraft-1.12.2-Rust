use crate::compat::Java::JavaRandom;
use crate::net::minecraft::world::gen::NoiseGeneratorSimplex::NoiseGeneratorSimplex;

/// MCP 1.12.2 `NoiseGeneratorPerlin` value path.
#[derive(Debug, Clone)]
pub struct NoiseGeneratorPerlin {
    noiseLevels: Vec<NoiseGeneratorSimplex>,
    levels: usize,
}

impl NoiseGeneratorPerlin {
    pub fn new(random: &mut JavaRandom, levelsIn: usize) -> Self {
        let noiseLevels = (0..levelsIn)
            .map(|_| NoiseGeneratorSimplex::new(random))
            .collect();
        Self {
            noiseLevels,
            levels: levelsIn,
        }
    }

    pub fn getValue(&self, x: f64, y: f64) -> f64 {
        let mut result = 0.0;
        let mut scale = 1.0;
        for level in 0..self.levels {
            result += self.noiseLevels[level].getValue(x * scale, y * scale) / scale;
            scale /= 2.0;
        }
        result
    }

    /// MCP `NoiseGeneratorPerlin#getRegion` default persistence parameter.
    #[allow(clippy::too_many_arguments)]
    pub fn getRegion(
        &self,
        reuse: Option<Vec<f64>>,
        x_start: f64,
        z_start: f64,
        x_size: i32,
        z_size: i32,
        x_scale: f64,
        z_scale: f64,
        persistence: f64,
    ) -> Vec<f64> {
        self.getRegionWithLacunarity(
            reuse,
            x_start,
            z_start,
            x_size,
            z_size,
            x_scale,
            z_scale,
            persistence,
            0.5,
        )
    }

    /// MCP `NoiseGeneratorPerlin#getRegion(..., persistence, lacunarity)`.
    #[allow(clippy::too_many_arguments)]
    pub fn getRegionWithLacunarity(
        &self,
        reuse: Option<Vec<f64>>,
        x_start: f64,
        z_start: f64,
        x_size: i32,
        z_size: i32,
        x_scale: f64,
        z_scale: f64,
        persistence: f64,
        lacunarity: f64,
    ) -> Vec<f64> {
        let len = (x_size as usize).saturating_mul(z_size as usize);
        let mut out = match reuse {
            Some(mut array) if array.len() >= len => {
                array.fill(0.0);
                array
            }
            _ => vec![0.0; len],
        };
        let mut d1 = 1.0_f64;
        let mut d0 = 1.0_f64;
        for level in 0..self.levels {
            self.noiseLevels[level].add(
                &mut out,
                x_start,
                z_start,
                x_size,
                z_size,
                x_scale * d0 * d1,
                z_scale * d0 * d1,
                0.55 / d1,
            );
            d0 *= persistence;
            d1 *= lacunarity;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn region_matches_mcp_java_golden_values() {
        let mut random = JavaRandom::new(12345);
        let perlin = NoiseGeneratorPerlin::new(&mut random, 4);
        let actual = perlin.getRegion(None, -13.0, 21.0, 3, 4, 0.0625, 0.03125, 1.0);
        let expected = [
            3.4828151133767897,
            3.4818697607465197,
            3.5042085698579055,
            3.4576300501007770,
            3.4452382825008880,
            3.4634651153733493,
            3.4348122788824350,
            3.4117051982336690,
            3.4254189294270083,
            3.4127431564036095,
            3.3796681503495978,
            3.3884563995903760,
        ];
        assert_eq!(actual.len(), expected.len());
        for (a, e) in actual.iter().zip(expected) {
            assert!((*a - e).abs() < 1.0e-14, "{a} != {e}");
        }
    }
}
