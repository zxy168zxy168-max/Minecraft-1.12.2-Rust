/// Rust ownership-equivalent of MCP 1.12.2 `IntCache`.
///
/// Java reuses globally-owned `int[]` instances between layer calls. Rust
/// layers return owned vectors, so returning an old backing buffer would
/// require unsafe aliasing across simultaneously-live parent results. The
/// observable contract is an integer buffer of at least the requested length;
/// allocation ownership is the language-specific equivalent.
pub struct IntCache;
impl IntCache {
    pub fn getIntCache(size: usize) -> Vec<i32> {
        vec![0; size]
    }
    pub fn resetIntCache() {}
}
