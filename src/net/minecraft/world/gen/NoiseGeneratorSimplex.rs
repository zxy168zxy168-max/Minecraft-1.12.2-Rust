use crate::compat::Java::JavaRandom;

/// Exact 2D branch of MCP 1.12.2 `NoiseGeneratorSimplex`.
#[derive(Debug, Clone)]
pub struct NoiseGeneratorSimplex {
    p: [i32; 512],
    pub xo: f64,
    pub yo: f64,
    pub zo: f64,
}

const GRAD3: [[i32; 3]; 12] = [
    [1, 1, 0],
    [-1, 1, 0],
    [1, -1, 0],
    [-1, -1, 0],
    [1, 0, 1],
    [-1, 0, 1],
    [1, 0, -1],
    [-1, 0, -1],
    [0, 1, 1],
    [0, -1, 1],
    [0, 1, -1],
    [0, -1, -1],
];

impl NoiseGeneratorSimplex {
    pub fn new(random: &mut JavaRandom) -> Self {
        let mut p = [0_i32; 512];
        let xo = random.next_f64() * 256.0;
        let yo = random.next_f64() * 256.0;
        let zo = random.next_f64() * 256.0;
        for (index, value) in p[..256].iter_mut().enumerate() {
            *value = index as i32;
        }
        for index in 0..256 {
            let selected = random.next_i32_bound((256 - index) as i32) as usize + index;
            p.swap(index, selected);
            p[index + 256] = p[index];
        }
        Self { p, xo, yo, zo }
    }

    fn fast_floor(value: f64) -> i32 {
        if value > 0.0 {
            value as i32
        } else {
            value as i32 - 1
        }
    }

    fn dot(gradient: [i32; 3], x: f64, y: f64) -> f64 {
        gradient[0] as f64 * x + gradient[1] as f64 * y
    }

    pub fn getValue(&self, x: f64, y: f64) -> f64 {
        let sqrt3 = 3.0_f64.sqrt();
        let f2 = 0.5 * (sqrt3 - 1.0);
        let skew = (x + y) * f2;
        let i = Self::fast_floor(x + skew);
        let j = Self::fast_floor(y + skew);
        let g2 = (3.0 - sqrt3) / 6.0;
        let unskew = (i + j) as f64 * g2;
        let origin_x = i as f64 - unskew;
        let origin_y = j as f64 - unskew;
        let x0 = x - origin_x;
        let y0 = y - origin_y;
        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };
        let x1 = x0 - i1 as f64 + g2;
        let y1 = y0 - j1 as f64 + g2;
        let x2 = x0 - 1.0 + 2.0 * g2;
        let y2 = y0 - 1.0 + 2.0 * g2;
        let ii = (i & 255) as usize;
        let jj = (j & 255) as usize;
        let gi0 = (self.p[ii + self.p[jj] as usize] % 12) as usize;
        let gi1 = (self.p[ii + i1 as usize + self.p[jj + j1 as usize] as usize] % 12) as usize;
        let gi2 = (self.p[ii + 1 + self.p[jj + 1] as usize] % 12) as usize;

        let contribution = |mut t: f64, gradient: [i32; 3], px: f64, py: f64| {
            if t < 0.0 {
                0.0
            } else {
                t *= t;
                t * t * Self::dot(gradient, px, py)
            }
        };
        let n0 = contribution(0.5 - x0 * x0 - y0 * y0, GRAD3[gi0], x0, y0);
        let n1 = contribution(0.5 - x1 * x1 - y1 * y1, GRAD3[gi1], x1, y1);
        let n2 = contribution(0.5 - x2 * x2 - y2 * y2, GRAD3[gi2], x2, y2);
        70.0 * (n0 + n1 + n2)
    }

    /// MCP 1.12.2 `NoiseGeneratorSimplex#add` used by
    /// `NoiseGeneratorPerlin#getRegion`. The destination index order is
    /// source-exact: z rows outside, x columns inside.
    #[allow(clippy::too_many_arguments)]
    pub fn add(
        &self,
        output: &mut [f64],
        x_start: f64,
        z_start: f64,
        x_size: i32,
        z_size: i32,
        x_scale: f64,
        z_scale: f64,
        amplitude: f64,
    ) {
        let needed = (x_size as usize).saturating_mul(z_size as usize);
        assert!(output.len() >= needed, "simplex output buffer too small");
        let sqrt3 = 3.0_f64.sqrt();
        let f2 = 0.5 * (sqrt3 - 1.0);
        let g2 = (3.0 - sqrt3) / 6.0;
        let mut out_index = 0usize;
        for z_index in 0..z_size {
            let d0 = (z_start + z_index as f64) * z_scale + self.yo;
            for x_index in 0..x_size {
                let d1 = (x_start + x_index as f64) * x_scale + self.xo;
                let d5 = (d1 + d0) * f2;
                let l = Self::fast_floor(d1 + d5);
                let i1 = Self::fast_floor(d0 + d5);
                let d6 = (l + i1) as f64 * g2;
                let d7 = l as f64 - d6;
                let d8 = i1 as f64 - d6;
                let d9 = d1 - d7;
                let d10 = d0 - d8;
                let (j1, k1) = if d9 > d10 { (1, 0) } else { (0, 1) };
                let d11 = d9 - j1 as f64 + g2;
                let d12 = d10 - k1 as f64 + g2;
                let d13 = d9 - 1.0 + 2.0 * g2;
                let d14 = d10 - 1.0 + 2.0 * g2;
                let l1 = (l & 255) as usize;
                let i2 = (i1 & 255) as usize;
                let j2 = (self.p[l1 + self.p[i2] as usize] % 12) as usize;
                let k2 =
                    (self.p[l1 + j1 as usize + self.p[i2 + k1 as usize] as usize] % 12) as usize;
                let l2 = (self.p[l1 + 1 + self.p[i2 + 1] as usize] % 12) as usize;
                let contribution = |mut t: f64, g: [i32; 3], px: f64, pz: f64| -> f64 {
                    if t < 0.0 {
                        0.0
                    } else {
                        t *= t;
                        t * t * Self::dot(g, px, pz)
                    }
                };
                let d2 = contribution(0.5 - d9 * d9 - d10 * d10, GRAD3[j2], d9, d10);
                let d3 = contribution(0.5 - d11 * d11 - d12 * d12, GRAD3[k2], d11, d12);
                let d4 = contribution(0.5 - d13 * d13 - d14 * d14, GRAD3[l2], d13, d14);
                output[out_index] += 70.0 * (d2 + d3 + d4) * amplitude;
                out_index += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_simplex_is_deterministic() {
        let mut random = JavaRandom::new(1234);
        let noise = NoiseGeneratorSimplex::new(&mut random);
        assert_eq!(
            noise.getValue(0.0, 0.0).to_bits(),
            noise.getValue(0.0, 0.0).to_bits()
        );
    }
}
