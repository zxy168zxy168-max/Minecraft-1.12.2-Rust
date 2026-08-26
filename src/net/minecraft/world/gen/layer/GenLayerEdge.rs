use super::GenLayer::{GenLayer, GenLayerSeed, Layer};
#[derive(Debug, Clone, Copy)]
pub enum Mode {
    CoolWarm,
    HeatIce,
    Special,
}
#[derive(Debug)]
pub struct GenLayerEdge {
    seed: GenLayerSeed,
    parent: Layer,
    mode: Mode,
}
impl GenLayerEdge {
    pub fn new(seed: i64, parent: Layer, mode: Mode) -> Self {
        Self {
            seed: GenLayerSeed::new(seed),
            parent,
            mode,
        }
    }
    fn coolWarm(&mut self, x: i32, z: i32, w: i32, h: i32) -> Vec<i32> {
        let pw = w + 2;
        let p = self.parent.lock().unwrap().getInts(x - 1, z - 1, pw, h + 2);
        let mut o = vec![0; (w * h) as usize];
        for yy in 0..h {
            for xx in 0..w {
                self.seed.initChunkSeed((xx + x) as i64, (yy + z) as i64);
                let mut c = p[(xx + 1 + (yy + 1) * pw) as usize];
                if c == 1 {
                    let n = p[(xx + 1 + yy * pw) as usize];
                    let e = p[(xx + 2 + (yy + 1) * pw) as usize];
                    let west = p[(xx + (yy + 1) * pw) as usize];
                    let s = p[(xx + 1 + (yy + 2) * pw) as usize];
                    if [n, e, west, s].iter().any(|v| matches!(*v, 3 | 4)) {
                        c = 2;
                    }
                }
                o[(xx + yy * w) as usize] = c;
            }
        }
        o
    }
    fn heatIce(&mut self, x: i32, z: i32, w: i32, h: i32) -> Vec<i32> {
        let pw = w + 2;
        let p = self.parent.lock().unwrap().getInts(x - 1, z - 1, pw, h + 2);
        let mut o = vec![0; (w * h) as usize];
        for yy in 0..h {
            for xx in 0..w {
                let mut c = p[(xx + 1 + (yy + 1) * pw) as usize];
                if c == 4 {
                    let n = p[(xx + 1 + yy * pw) as usize];
                    let e = p[(xx + 2 + (yy + 1) * pw) as usize];
                    let west = p[(xx + (yy + 1) * pw) as usize];
                    let s = p[(xx + 1 + (yy + 2) * pw) as usize];
                    if [n, e, west, s].iter().any(|v| matches!(*v, 1 | 2)) {
                        c = 3;
                    }
                }
                o[(xx + yy * w) as usize] = c;
            }
        }
        o
    }
    fn special(&mut self, x: i32, z: i32, w: i32, h: i32) -> Vec<i32> {
        let p = self.parent.lock().unwrap().getInts(x, z, w, h);
        let mut o = vec![0; (w * h) as usize];
        for yy in 0..h {
            for xx in 0..w {
                self.seed.initChunkSeed((xx + x) as i64, (yy + z) as i64);
                let mut c = p[(xx + yy * w) as usize];
                if c != 0 && self.seed.nextInt(13) == 0 {
                    c |= (1 + self.seed.nextInt(15)) << 8 & 3840;
                }
                o[(xx + yy * w) as usize] = c;
            }
        }
        o
    }
}
impl GenLayer for GenLayerEdge {
    fn initWorldGenSeed(&mut self, s: i64) {
        self.parent.lock().unwrap().initWorldGenSeed(s);
        self.seed.initWorldGenSeed(s)
    }
    fn getInts(&mut self, x: i32, z: i32, w: i32, h: i32) -> Vec<i32> {
        match self.mode {
            Mode::CoolWarm => self.coolWarm(x, z, w, h),
            Mode::HeatIce => self.heatIce(x, z, w, h),
            Mode::Special => self.special(x, z, w, h),
        }
    }
}
