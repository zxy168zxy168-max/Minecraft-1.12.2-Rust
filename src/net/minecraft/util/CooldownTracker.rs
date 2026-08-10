use std::collections::HashMap;

/// Rust equivalent of MCP 1.12.2 `CooldownTracker`.
///
/// Java keys the map by `Item` singleton. The protocol-facing Rust item model
/// uses the stable numeric item registry id, which is the equivalent identity
/// for this client and is also what `SPacketCooldown` serializes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CooldownTracker {
    cooldowns: HashMap<i16, Cooldown>,
    ticks: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cooldown {
    createTicks: i32,
    expireTicks: i32,
}

impl CooldownTracker {
    pub fn hasCooldown(&self, itemId: i16) -> bool {
        self.getCooldown(itemId, 0.0) > 0.0
    }

    /// MCP `CooldownTracker#getCooldown(Item,float)`.
    pub fn getCooldown(&self, itemId: i16, partialTicks: f32) -> f32 {
        let Some(cooldown) = self.cooldowns.get(&itemId) else {
            return 0.0;
        };
        let total = (cooldown.expireTicks - cooldown.createTicks) as f32;
        if total <= 0.0 {
            return 0.0;
        }
        let remaining = cooldown.expireTicks as f32 - (self.ticks as f32 + partialTicks);
        (remaining / total).clamp(0.0, 1.0)
    }

    /// MCP `CooldownTracker#tick`.
    pub fn tick(&mut self) {
        self.ticks = self.ticks.saturating_add(1);
        let ticks = self.ticks;
        self.cooldowns.retain(|_, cooldown| cooldown.expireTicks > ticks);
    }

    /// MCP `CooldownTracker#setCooldown`.
    pub fn setCooldown(&mut self, itemId: i16, ticksIn: i32) {
        self.cooldowns.insert(
            itemId,
            Cooldown {
                createTicks: self.ticks,
                expireTicks: self.ticks.saturating_add(ticksIn),
            },
        );
    }

    /// MCP `CooldownTracker#removeCooldown`.
    pub fn removeCooldown(&mut self, itemId: i16) {
        self.cooldowns.remove(&itemId);
    }
}

#[cfg(test)]
mod tests {
    use super::CooldownTracker;

    #[test]
    fn cooldown_fraction_and_expiry_match_mcp() {
        let mut tracker = CooldownTracker::default();
        tracker.setCooldown(368, 20);
        assert_eq!(tracker.getCooldown(368, 0.0), 1.0);
        for _ in 0..10 {
            tracker.tick();
        }
        assert!((tracker.getCooldown(368, 0.0) - 0.5).abs() < f32::EPSILON);
        assert!(tracker.hasCooldown(368));
        for _ in 0..10 {
            tracker.tick();
        }
        assert!(!tracker.hasCooldown(368));
    }

    #[test]
    fn packet_style_zero_removal_is_immediate() {
        let mut tracker = CooldownTracker::default();
        tracker.setCooldown(442, 100);
        tracker.removeCooldown(442);
        assert!(!tracker.hasCooldown(442));
    }
}
