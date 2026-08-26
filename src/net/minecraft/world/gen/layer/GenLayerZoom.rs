use super::GenLayer::{layer, GenLayer, GenLayerSeed, Layer};
#[derive(Debug)]
pub struct GenLayerZoom {
    seed: GenLayerSeed,
    parent: Layer,
    fuzzy: bool,
}
impl GenLayerZoom {
    pub fn new(seed: i64, parent: Layer) -> Self {
        Self {
            seed: GenLayerSeed::new(seed),
            parent,
            fuzzy: false,
        }
    }
    pub(crate) fn newFuzzy(seed: i64, parent: Layer) -> Self {
        Self {
            seed: GenLayerSeed::new(seed),
            parent,
            fuzzy: true,
        }
    }
    fn select(&mut self, a: i32, b: i32, c: i32, d: i32) -> i32 {
        if self.fuzzy {
            self.seed.selectRandom(&[a, b, c, d])
        } else {
            self.seed.selectModeOrRandom(a, b, c, d)
        }
    }
}
pub fn magnify(seed: i64, mut parent: Layer, times: i32) -> Layer {
    for i in 0..times {
        parent = layer(GenLayerZoom::new(seed + i as i64, parent));
    }
    parent
}
impl GenLayer for GenLayerZoom {
    fn initWorldGenSeed(&mut self, s: i64) {
        self.parent.lock().unwrap().initWorldGenSeed(s);
        self.seed.initWorldGenSeed(s)
    }
    fn getInts(&mut self, areaX: i32, areaY: i32, w: i32, h: i32) -> Vec<i32> {
        let px = areaX >> 1;
        let py = areaY >> 1;
        let pw = (w >> 1) + 2;
        let ph = (h >> 1) + 2;
        let p = self.parent.lock().unwrap().getInts(px, py, pw, ph);
        let ow = (pw - 1) << 1;
        let oh = (ph - 1) << 1;
        let mut expanded = vec![0; (ow * oh) as usize];
        for y in 0..ph - 1 {
            let mut out = ((y << 1) * ow) as usize;
            let mut tl = p[(y * pw) as usize];
            let mut bl = p[((y + 1) * pw) as usize];
            for x in 0..pw - 1 {
                self.seed
                    .initChunkSeed(((x + px) << 1) as i64, ((y + py) << 1) as i64);
                let tr = p[(x + 1 + y * pw) as usize];
                let br = p[(x + 1 + (y + 1) * pw) as usize];
                expanded[out] = tl;
                expanded[out + ow as usize] = self.seed.selectRandom(&[tl, bl]);
                out += 1;
                expanded[out] = self.seed.selectRandom(&[tl, tr]);
                expanded[out + ow as usize] = self.select(tl, tr, bl, br);
                out += 1;
                tl = tr;
                bl = br;
            }
        }
        let mut o = vec![0; (w * h) as usize];
        for y in 0..h {
            let src = ((y + (areaY & 1)) * ow + (areaX & 1)) as usize;
            let dst = (y * w) as usize;
            o[dst..dst + w as usize].copy_from_slice(&expanded[src..src + w as usize]);
        }
        o
    }
}
