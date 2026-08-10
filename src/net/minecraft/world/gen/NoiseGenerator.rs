/// MCP 1.12.2 `NoiseGenerator` is an empty abstract ownership base.
///
/// Rust uses a marker trait so concrete vanilla noise generators retain the
/// same conceptual hierarchy without inventing runtime behaviour.
pub trait NoiseGenerator {}
