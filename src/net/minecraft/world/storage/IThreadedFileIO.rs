/// Rust equivalent of MCP 1.12.2 `IThreadedFileIO`.
///
/// Java queue identity is object identity. Rust trait objects do not expose
/// that identity directly, so `ioIdentity` is the language-equivalent hook used
/// only by `ThreadedFileIOBase` to preserve `List.contains(fileIo)` semantics.
pub trait IThreadedFileIO: Send + Sync {
    /// MCP `writeNextIO`: `true` means the object still has work and must stay
    /// in the global file-I/O queue; `false` removes it from the queue.
    fn writeNextIO(&self) -> bool;
    fn ioIdentity(&self) -> usize;
}
