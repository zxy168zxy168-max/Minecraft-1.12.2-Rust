use super::GenLayer::{GenLayer, GenLayerSeed, Layer};
#[derive(Debug)]
pub struct GenLayerDeepOcean {
    seed: GenLayerSeed,
    parent: Layer,
}
impl GenLayerDeepOcean {
    pub fn new(seed: i64, parent: Layer) -> Self {
        Self {
            seed: GenLayerSeed::new(seed),
            parent,
        }
    }
}
impl GenLayer for GenLayerDeepOcean {
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
                let vals = [
                    p[(xx + 1 + yy * pw) as usize],
                    p[(xx + 2 + (yy + 1) * pw) as usize],
                    p[(xx + (yy + 1) * pw) as usize],
                    p[(xx + 1 + (yy + 2) * pw) as usize],
                ];
                let c = p[(xx + 1 + (yy + 1) * pw) as usize];
                let ocean = vals.into_iter().filter(|v| *v == 0).count();
                o[(xx + yy * w) as usize] = if c == 0 && ocean > 3 { 24 } else { c };
            }
        }
        o
    }
}
