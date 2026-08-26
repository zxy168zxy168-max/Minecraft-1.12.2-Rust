use super::GenLayer::{biomesEqualOrMesaPlateau, GenLayer, GenLayerSeed, Layer};
use crate::net::minecraft::world::biome::Biome::Biome;
#[derive(Debug)]
pub struct GenLayerHills {
    seed: GenLayerSeed,
    parent: Layer,
    riverLayer: Layer,
}
impl GenLayerHills {
    pub fn new(seed: i64, parent: Layer, riverLayer: Layer) -> Self {
        Self {
            seed: GenLayerSeed::new(seed),
            parent,
            riverLayer,
        }
    }
}
impl GenLayer for GenLayerHills {
    fn initWorldGenSeed(&mut self, s: i64) {
        /* Vanilla base GenLayer initializes only parent; riverLayer is deliberately not traversed here. */
        self.parent.lock().unwrap().initWorldGenSeed(s);
        self.seed.initWorldGenSeed(s)
    }
    fn getInts(&mut self, x: i32, z: i32, w: i32, h: i32) -> Vec<i32> {
        let p = self
            .parent
            .lock()
            .unwrap()
            .getInts(x - 1, z - 1, w + 2, h + 2);
        let r = self
            .riverLayer
            .lock()
            .unwrap()
            .getInts(x - 1, z - 1, w + 2, h + 2);
        let mut o = vec![0; (w * h) as usize];
        for yy in 0..h {
            for xx in 0..w {
                self.seed.initChunkSeed((xx + x) as i64, (yy + z) as i64);
                let idx = (xx + 1 + (yy + 1) * (w + 2)) as usize;
                let k = p[idx];
                let l = r[idx];
                let flag = (l - 2) % 29 == 0;
                let biome = Biome::getBiomeForId(k);
                let mutated = biome.is_some_and(|b| b.isMutation());
                let out = if k != 0 && l >= 2 && (l - 2) % 29 == 1 && !mutated {
                    biome
                        .and_then(|b| b.getMutationForBiome())
                        .map(|b| b.getId() as i32)
                        .unwrap_or(k)
                } else if self.seed.nextInt(3) != 0 && !flag {
                    k
                } else {
                    let mut hill = match k {
                        2 => 17,
                        4 => 18,
                        27 => 28,
                        29 => 1,
                        5 => 19,
                        32 => 33,
                        30 => 31,
                        1 => {
                            if self.seed.nextInt(3) == 0 {
                                18
                            } else {
                                4
                            }
                        }
                        12 => 13,
                        21 => 22,
                        0 => 24,
                        3 => 34,
                        35 => 36,
                        _ => k,
                    };
                    if biomesEqualOrMesaPlateau(k, 38) {
                        hill = 37;
                    } else if k == 24 && self.seed.nextInt(3) == 0 {
                        hill = if self.seed.nextInt(2) == 0 { 1 } else { 4 };
                    }
                    if flag && hill != k {
                        hill = Biome::getBiomeForId(hill)
                            .and_then(|b| b.getMutationForBiome())
                            .map(|b| b.getId() as i32)
                            .unwrap_or(k);
                    }
                    if hill == k {
                        k
                    } else {
                        let pw = w + 2;
                        let ns = [
                            p[(xx + 1 + yy * pw) as usize],
                            p[(xx + 2 + (yy + 1) * pw) as usize],
                            p[(xx + (yy + 1) * pw) as usize],
                            p[(xx + 1 + (yy + 2) * pw) as usize],
                        ];
                        if ns
                            .into_iter()
                            .filter(|v| biomesEqualOrMesaPlateau(*v, k))
                            .count()
                            >= 3
                        {
                            hill
                        } else {
                            k
                        }
                    }
                };
                o[(xx + yy * w) as usize] = out;
            }
        }
        o
    }
}
