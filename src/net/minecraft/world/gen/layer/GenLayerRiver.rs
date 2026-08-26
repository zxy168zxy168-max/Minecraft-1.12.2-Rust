use super::GenLayer::{GenLayer, GenLayerSeed, Layer};
#[derive(Debug)]
pub struct GenLayerRiver {
    seed: GenLayerSeed,
    parent: Layer,
}
impl GenLayerRiver {
    pub fn new(seed: i64, parent: Layer) -> Self {
        Self {
            seed: GenLayerSeed::new(seed),
            parent,
        }
    }
    fn riverFilter(v: i32) -> i32 {
        if v >= 2 {
            2 + (v & 1)
        } else {
            v
        }
    }
}
impl GenLayer for GenLayerRiver {
    fn initWorldGenSeed(&mut self, s: i64) {
        self.parent.lock().unwrap().initWorldGenSeed(s);
        self.seed.initWorldGenSeed(s)
    }
    fn getInts(&mut self, x: i32, z: i32, w: i32, h: i32) -> Vec<i32> {
        let pw = w + 2;
        let p = self.parent.lock().unwrap().getInts(x - 1, z - 1, pw, h + 2);
        let mut o = vec![0; (w * h) as usize];
        for yy in 0..h {
            for xx in 0..w {
                let west = Self::riverFilter(p[(xx + (yy + 1) * pw) as usize]);
                let e = Self::riverFilter(p[(xx + 2 + (yy + 1) * pw) as usize]);
                let n = Self::riverFilter(p[(xx + 1 + yy * pw) as usize]);
                let s = Self::riverFilter(p[(xx + 1 + (yy + 2) * pw) as usize]);
                let c = Self::riverFilter(p[(xx + 1 + (yy + 1) * pw) as usize]);
                o[(xx + yy * w) as usize] = if c == west && c == e && c == n && c == s {
                    -1
                } else {
                    7
                };
            }
        }
        o
    }
}
