/// Network-relevant subset of MCP 1.12.2 `PotionEffect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PotionEffect {
    potionId: u8,
    duration: i32,
    amplifier: u8,
    ambient: bool,
    showParticles: bool,
    potionDurationMax: bool,
}

impl PotionEffect {
    pub const fn new(
        potionId: u8,
        duration: i32,
        amplifier: u8,
        ambient: bool,
        showParticles: bool,
    ) -> Self {
        Self {
            potionId,
            duration,
            amplifier,
            ambient,
            showParticles,
            potionDurationMax: duration == 32_767,
        }
    }

    pub const fn getPotionId(&self) -> u8 {
        self.potionId
    }
    pub const fn getDuration(&self) -> i32 {
        self.duration
    }
    pub const fn getAmplifier(&self) -> u8 {
        self.amplifier
    }
    pub const fn getIsAmbient(&self) -> bool {
        self.ambient
    }
    pub const fn doesShowParticles(&self) -> bool {
        self.showParticles
    }
    pub const fn getIsPotionDurationMax(&self) -> bool {
        self.potionDurationMax
    }

    /// `PotionEffect#onUpdate` duration half. Effect-specific periodic actions
    /// are added by their concrete potion classes; absorption only needs the
    /// application/removal hooks and duration lifecycle.
    pub fn tickDuration(&mut self) -> bool {
        if self.duration > 0 {
            self.duration -= 1;
        }
        self.duration > 0
    }
}
