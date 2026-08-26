use std::cmp::Ordering;
use std::sync::atomic::{AtomicI64, Ordering as AtomicOrdering};

use crate::net::minecraft::block::Block::Block;
use crate::net::minecraft::util::math::BlockPos::BlockPos;

static NEXT_TICK_ENTRY_ID: AtomicI64 = AtomicI64::new(0);

/// MCP 1.12.2 `NextTickListEntry`.
#[derive(Debug, Clone)]
pub struct NextTickListEntry {
    block: Block,
    pub position: BlockPos,
    pub scheduledTime: i64,
    pub priority: i32,
    tickEntryID: i64,
}

impl NextTickListEntry {
    pub fn new(positionIn: BlockPos, blockIn: Block) -> Self {
        Self {
            block: blockIn,
            position: positionIn,
            scheduledTime: 0,
            priority: 0,
            tickEntryID: NEXT_TICK_ENTRY_ID.fetch_add(1, AtomicOrdering::Relaxed),
        }
    }

    pub fn setScheduledTime(mut self, scheduledTimeIn: i64) -> Self {
        self.scheduledTime = scheduledTimeIn;
        self
    }
    pub fn setPriority(&mut self, priorityIn: i32) {
        self.priority = priorityIn;
    }
    pub const fn getBlock(&self) -> Block {
        self.block
    }
    pub const fn getTickEntryID(&self) -> i64 {
        self.tickEntryID
    }

    /// MCP `compareTo`: scheduled time, then priority, then insertion id.
    pub fn compareTo(&self, other: &Self) -> Ordering {
        self.scheduledTime
            .cmp(&other.scheduledTime)
            .then_with(|| self.priority.cmp(&other.priority))
            .then_with(|| self.tickEntryID.cmp(&other.tickEntryID))
    }
}

/// MCP `equals`: block identity + block position only. This intentionally
/// differs from `compareTo`, exactly as the Java class does (WorldServer uses
/// a HashSet for identity and a TreeSet for scheduling order).
impl PartialEq for NextTickListEntry {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.block == other.block
    }
}
impl Eq for NextTickListEntry {}

impl std::hash::Hash for NextTickListEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Java's hashCode is position.hashCode only; structural Rust hashing of
        // BlockPos is sufficient for the equality contract and is not exposed
        // as Java hashCode.
        self.position.hash(state);
    }
}

impl Ord for NextTickListEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compareTo(other)
    }
}
impl PartialOrd for NextTickListEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.compareTo(other))
    }
}

impl std::fmt::Display for NextTickListEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {:?}, {}, {}, {}",
            Block::getIdFromBlock(self.block),
            self.position,
            self.scheduledTime,
            self.priority,
            self.tickEntryID
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality_and_schedule_order_follow_java_split_contract() {
        let block = Block::getBlockById(1);
        let pos = BlockPos::new(1, 2, 3);
        let first = NextTickListEntry::new(pos, block).setScheduledTime(100);
        let second = NextTickListEntry::new(pos, block).setScheduledTime(200);
        assert_eq!(first, second);
        assert_eq!(first.compareTo(&second), Ordering::Less);
    }
}
