use super::GenLayer::{GenLayer, Layer};
use super::GenLayerZoom::GenLayerZoom;
#[derive(Debug)]
pub struct GenLayerFuzzyZoom {
    inner: GenLayerZoom,
}
impl GenLayerFuzzyZoom {
    pub fn new(seed: i64, parent: Layer) -> Self {
        Self {
            inner: GenLayerZoom::newFuzzy(seed, parent),
        }
    }
}
impl GenLayer for GenLayerFuzzyZoom {
    fn initWorldGenSeed(&mut self, s: i64) {
        self.inner.initWorldGenSeed(s)
    }
    fn getInts(&mut self, x: i32, z: i32, w: i32, h: i32) -> Vec<i32> {
        self.inner.getInts(x, z, w, h)
    }
}
