use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use crate::net::minecraft::world::storage::IThreadedFileIO::IThreadedFileIO;

/// MCP 1.12.2 `ThreadedFileIOBase`.
///
/// The source owns one global low-priority worker, deduplicates queued loader
/// objects by identity, increments `writeQueuedCounter` only when an object
/// enters the queue, and increments `savedIOCounter` only when that object's
/// `writeNextIO` returns false. Rust uses a Condvar to wake the worker instead
/// of polling the empty queue for 25 ms; work ordering/counter semantics remain
/// source-equivalent while avoiding an idle spin/sleep thread.
pub struct ThreadedFileIOBase {
    queue: Mutex<Vec<Arc<dyn IThreadedFileIO>>>,
    wake: Condvar,
    writeQueuedCounter: AtomicU64,
    savedIOCounter: AtomicU64,
    isThreadWaiting: AtomicBool,
}

impl std::fmt::Debug for ThreadedFileIOBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadedFileIOBase")
            .field("writeQueuedCounter", &self.writeQueuedCounter.load(Ordering::Acquire))
            .field("savedIOCounter", &self.savedIOCounter.load(Ordering::Acquire))
            .field("isThreadWaiting", &self.isThreadWaiting.load(Ordering::Acquire))
            .finish_non_exhaustive()
    }
}

impl ThreadedFileIOBase {
    fn new() -> Arc<Self> {
        let instance = Arc::new(Self {
            queue: Mutex::new(Vec::new()),
            wake: Condvar::new(),
            writeQueuedCounter: AtomicU64::new(0),
            savedIOCounter: AtomicU64::new(0),
            isThreadWaiting: AtomicBool::new(false),
        });
        let worker = Arc::clone(&instance);
        let _ = thread::Builder::new().name("File IO Thread".to_owned()).spawn(move || worker.run());
        instance
    }

    /// MCP `getThreadedIOInstance` singleton.
    pub fn getThreadedIOInstance() -> &'static Arc<Self> {
        static INSTANCE: OnceLock<Arc<ThreadedFileIOBase>> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    fn run(self: Arc<Self>) {
        loop {
            self.processQueue();
        }
    }

    /// MCP `processQueue`. Java's synchronized-list wrapper locks each list
    /// operation individually, so the I/O callback itself is deliberately run
    /// without holding the Rust queue mutex. The loop index is decremented on
    /// removal just like `remove(i--)`, preserving source traversal order.
    fn processQueue(&self) {
        let mut index = 0usize;
        loop {
            let fileIo = {
                let queue = self.queue.lock().unwrap_or_else(|p| p.into_inner());
                queue.get(index).cloned()
            };
            let Some(fileIo) = fileIo else { break; };

            let keepQueued = fileIo.writeNextIO();
            if !keepQueued {
                let identity = fileIo.ioIdentity();
                let mut queue = self.queue.lock().unwrap_or_else(|p| p.into_inner());
                if index < queue.len() && queue[index].ioIdentity() == identity {
                    queue.remove(index);
                    self.savedIOCounter.fetch_add(1, Ordering::AcqRel);
                } else if let Some(actual) = queue.iter().position(|queued| queued.ioIdentity() == identity) {
                    queue.remove(actual);
                    self.savedIOCounter.fetch_add(1, Ordering::AcqRel);
                    if actual < index {
                        index = index.saturating_sub(1);
                    }
                }
                self.wake.notify_all();
            } else {
                index += 1;
            }

            // Exact 1.12.2 cadence: 10 ms between queued objects, zero while
            // waitForFinish is actively draining the queue.
            if !self.isThreadWaiting.load(Ordering::Acquire) {
                thread::sleep(Duration::from_millis(10));
            } else {
                thread::yield_now();
            }
        }

        let empty = self.queue.lock().unwrap_or_else(|p| p.into_inner()).is_empty();
        if empty {
            // Source sleeps 25 ms when idle. Condvar wake-up is used only as a
            // Rust runtime optimisation so newly queued work need not wait the
            // full poll interval; no queue/counter semantics are changed.
            let guard = self.queue.lock().unwrap_or_else(|p| p.into_inner());
            if guard.is_empty() {
                let _ = self.wake.wait_timeout(guard, Duration::from_millis(25));
            }
        }
    }

    /// MCP `queueIO`. Duplicate object identity does not increment the queued
    /// counter, matching `threadedIOQueue.contains(fileIo)`.
    pub fn queueIO(&self, fileIo: Arc<dyn IThreadedFileIO>) {
        let identity = fileIo.ioIdentity();
        let mut queue = self.queue.lock().unwrap_or_else(|p| p.into_inner());
        if queue.iter().any(|queued| queued.ioIdentity() == identity) {
            return;
        }
        self.writeQueuedCounter.fetch_add(1, Ordering::AcqRel);
        queue.push(fileIo);
        self.wake.notify_all();
    }

    /// MCP `waitForFinish`. This is intentionally counter-based rather than
    /// `queue.is_empty()` because the source waits for every unique queued I/O
    /// object to complete its final `writeNextIO` call.
    pub fn waitForFinish(&self) {
        self.isThreadWaiting.store(true, Ordering::Release);
        loop {
            let queued = self.writeQueuedCounter.load(Ordering::Acquire);
            let saved = self.savedIOCounter.load(Ordering::Acquire);
            if queued == saved { break; }
            let guard = self.queue.lock().unwrap_or_else(|p| p.into_inner());
            let _ = self.wake.wait_timeout(guard, Duration::from_millis(10));
        }
        self.isThreadWaiting.store(false, Ordering::Release);
    }

    pub fn writeQueuedCounter(&self) -> u64 { self.writeQueuedCounter.load(Ordering::Acquire) }
    pub fn savedIOCounter(&self) -> u64 { self.savedIOCounter.load(Ordering::Acquire) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockIO { id: usize, remaining: AtomicUsize }
    impl IThreadedFileIO for MockIO {
        fn writeNextIO(&self) -> bool {
            let current = self.remaining.load(Ordering::Acquire);
            if current == 0 { return false; }
            self.remaining.fetch_sub(1, Ordering::AcqRel);
            true
        }
        fn ioIdentity(&self) -> usize { self.id }
    }

    #[test]
    fn duplicate_identity_is_queued_once_and_finish_waits_for_false() {
        let base = ThreadedFileIOBase::getThreadedIOInstance();
        let beforeQueued = base.writeQueuedCounter();
        let beforeSaved = base.savedIOCounter();
        let io: Arc<dyn IThreadedFileIO> = Arc::new(MockIO { id: usize::MAX - beforeQueued as usize, remaining: AtomicUsize::new(2) });
        base.queueIO(Arc::clone(&io));
        base.queueIO(io);
        while base.savedIOCounter() < beforeSaved + 1 { thread::sleep(Duration::from_millis(2)); }
        assert_eq!(base.writeQueuedCounter(), beforeQueued + 1);
        assert_eq!(base.savedIOCounter(), beforeSaved + 1);
    }
}
