use super::GenLayer::{GenLayer, GenLayerSeed, Layer};
#[derive(Debug)]
pub struct GenLayerVoronoiZoom {
    seed: GenLayerSeed,
    parent: Layer,
}
impl GenLayerVoronoiZoom {
    pub fn new(seed: i64, parent: Layer) -> Self {
        Self {
            seed: GenLayerSeed::new(seed),
            parent,
        }
    }
}
impl GenLayer for GenLayerVoronoiZoom {
    fn initWorldGenSeed(&mut self, s: i64) {
        self.parent.lock().unwrap().initWorldGenSeed(s);
        self.seed.initWorldGenSeed(s)
    }
    fn getInts(&mut self, mut x: i32, mut z: i32, w: i32, h: i32) -> Vec<i32> {
        x -= 2;
        z -= 2;
        let px = x >> 2;
        let pz = z >> 2;
        let pw = (w >> 2) + 2;
        let ph = (h >> 2) + 2;
        let p = self.parent.lock().unwrap().getInts(px, pz, pw, ph);
        let ow = (pw - 1) << 2;
        let oh = (ph - 1) << 2;
        let mut e = vec![0; (ow * oh) as usize];
        for yy in 0..ph - 1 {
            let mut tl = p[(yy * pw) as usize];
            let mut bl = p[((yy + 1) * pw) as usize];
            for xx in 0..pw - 1 {
                self.seed
                    .initChunkSeed(((xx + px) << 2) as i64, ((yy + pz) << 2) as i64);
                let d1 = (self.seed.nextInt(1024) as f64 / 1024.0 - 0.5) * 3.6;
                let d2 = (self.seed.nextInt(1024) as f64 / 1024.0 - 0.5) * 3.6;
                self.seed
                    .initChunkSeed(((xx + px + 1) << 2) as i64, ((yy + pz) << 2) as i64);
                let d3 = (self.seed.nextInt(1024) as f64 / 1024.0 - 0.5) * 3.6 + 4.0;
                let d4 = (self.seed.nextInt(1024) as f64 / 1024.0 - 0.5) * 3.6;
                self.seed
                    .initChunkSeed(((xx + px) << 2) as i64, ((yy + pz + 1) << 2) as i64);
                let d5 = (self.seed.nextInt(1024) as f64 / 1024.0 - 0.5) * 3.6;
                let d6 = (self.seed.nextInt(1024) as f64 / 1024.0 - 0.5) * 3.6 + 4.0;
                self.seed
                    .initChunkSeed(((xx + px + 1) << 2) as i64, ((yy + pz + 1) << 2) as i64);
                let d7 = (self.seed.nextInt(1024) as f64 / 1024.0 - 0.5) * 3.6 + 4.0;
                let d8 = (self.seed.nextInt(1024) as f64 / 1024.0 - 0.5) * 3.6 + 4.0;
                let tr = p[(xx + 1 + yy * pw) as usize] & 255;
                let br = p[(xx + 1 + (yy + 1) * pw) as usize] & 255;
                for iy in 0..4 {
                    let mut out = (((yy << 2) + iy) * ow + (xx << 2)) as usize;
                    for ix in 0..4 {
                        let a = (iy as f64 - d2).powi(2) + (ix as f64 - d1).powi(2);
                        let b = (iy as f64 - d4).powi(2) + (ix as f64 - d3).powi(2);
                        let c = (iy as f64 - d6).powi(2) + (ix as f64 - d5).powi(2);
                        let d = (iy as f64 - d8).powi(2) + (ix as f64 - d7).powi(2);
                        e[out] = if a < b && a < c && a < d {
                            tl
                        } else if b < a && b < c && b < d {
                            tr
                        } else if c < a && c < b && c < d {
                            bl
                        } else {
                            br
                        };
                        out += 1;
                    }
                }
                tl = tr;
                bl = br;
            }
        }
        let mut o = vec![0; (w * h) as usize];
        for yy in 0..h {
            let src = ((yy + (z & 3)) * ow + (x & 3)) as usize;
            let dst = (yy * w) as usize;
            o[dst..dst + w as usize].copy_from_slice(&e[src..src + w as usize]);
        }
        o
    }
}
