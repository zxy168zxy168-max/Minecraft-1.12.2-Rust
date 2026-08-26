use super::GenLayer::{GenLayer, GenLayerSeed, Layer};
#[derive(Debug)]
pub struct GenLayerRemoveTooMuchOcean {
    seed: GenLayerSeed,
    parent: Layer,
}
impl GenLayerRemoveTooMuchOcean {
    pub fn new(seed: i64, parent: Layer) -> Self {
        Self {
            seed: GenLayerSeed::new(seed),
            parent,
        }
    }
}
impl GenLayer for GenLayerRemoveTooMuchOcean {
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
                let n = p[(xx + 1 + yy * pw) as usize];
                let e = p[(xx + 2 + (yy + 1) * pw) as usize];
                let west = p[(xx + (yy + 1) * pw) as usize];
                let s = p[(xx + 1 + (yy + 2) * pw) as usize];
                let c = p[(xx + 1 + (yy + 1) * pw) as usize];
                self.seed.initChunkSeed((xx + x) as i64, (yy + z) as i64);
                o[(xx + yy * w) as usize] = if c == 0
                    && n == 0
                    && e == 0
                    && west == 0
                    && s == 0
                    && self.seed.nextInt(2) == 0
                {
                    1
                } else {
                    c
                };
            }
        }
        o
    }
}
