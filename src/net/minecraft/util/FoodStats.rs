use crate::net::minecraft::world::EnumDifficulty::EnumDifficulty;

/// Full MCP 1.12.2 `FoodStats` port.
///
/// Vanilla ticks this from the authoritative world only (`EntityPlayer#onUpdate`
/// guards it with `!world.isRemote`), so a thin client never runs the
/// exhaustion/regeneration logic itself; the values arrive via
/// `SPacketUpdateHealth`. The whole class is ported anyway so the future
/// integrated server path executes the same algorithm.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodStats {
    foodLevel: i32,
    foodSaturationLevel: f32,
    foodExhaustionLevel: f32,
    foodTimer: i32,
    prevFoodLevel: i32,
}

impl Default for FoodStats {
    fn default() -> Self {
        Self {
            foodLevel: 20,
            foodSaturationLevel: 5.0,
            foodExhaustionLevel: 0.0,
            foodTimer: 0,
            prevFoodLevel: 20,
        }
    }
}

/// Side effects `FoodStats#onUpdate` performs on the player, returned so the
/// caller (the player tick) applies them — the Rust equivalent of the MCP
/// method receiving the `EntityPlayer`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FoodStatsAction {
    None,
    /// `EntityLivingBase#heal`: natural regeneration from saturation/food.
    Heal(f32),
    /// `attackEntityFrom(DamageSource.STARVE, 1.0F)`: starvation damage.
    Starve(f32),
}

impl FoodStats {
    /// `FoodStats#addStats(int, float)`: foodLevelIn heals and
    /// foodSaturationModifier scaled twice into saturation, both capped.
    pub fn addStats(&mut self, foodLevelIn: i32, foodSaturationModifier: f32) {
        self.foodLevel = (foodLevelIn + self.foodLevel).min(20);
        self.foodSaturationLevel = (self.foodSaturationLevel
            + foodLevelIn as f32 * foodSaturationModifier * 2.0)
            .min(self.foodLevel as f32);
    }

    /// `FoodStats#addStats(ItemFood, ItemStack)` equivalent: heal amount and
    /// saturation modifier from the item's food definition.
    pub fn addStatsForFood(&mut self, healAmount: i32, saturationModifier: f32) {
        self.addStats(healAmount, saturationModifier);
    }

    /// `FoodStats#onUpdate(EntityPlayer)`: consumes exhaustion, heals from
    /// saturation/food when `naturalRegeneration` applies, and starves when
    /// the food level hits zero. The returned action must be applied by the
    /// caller (heal damage / starvation damage).
    pub fn onUpdate(
        &mut self,
        difficulty: EnumDifficulty,
        naturalRegeneration: bool,
        shouldHeal: bool,
        health: f32,
    ) -> FoodStatsAction {
        self.prevFoodLevel = self.foodLevel;
        if self.foodExhaustionLevel > 4.0 {
            self.foodExhaustionLevel -= 4.0;
            if self.foodSaturationLevel > 0.0 {
                self.foodSaturationLevel = (self.foodSaturationLevel - 1.0).max(0.0);
            } else if difficulty != EnumDifficulty::Peaceful {
                self.foodLevel = (self.foodLevel - 1).max(0);
            }
        }

        if naturalRegeneration && self.foodSaturationLevel > 0.0 && shouldHeal && self.foodLevel >= 20 {
            self.foodTimer += 1;
            if self.foodTimer >= 10 {
                // Java: `float f = Math.min(foodSaturationLevel, 6.0F);
                // player.heal(f / 6.0F); this.addExhaustion(f);` — the
                // exhaustion added is f (up to 6.0), not the heal amount.
                let saturation = self.foodSaturationLevel.min(6.0);
                self.addExhaustion(saturation);
                self.foodTimer = 0;
                return FoodStatsAction::Heal(saturation / 6.0);
            }
        } else if naturalRegeneration && self.foodLevel >= 18 && shouldHeal {
            self.foodTimer += 1;
            if self.foodTimer >= 80 {
                self.addExhaustion(6.0);
                self.foodTimer = 0;
                return FoodStatsAction::Heal(1.0);
            }
        } else if self.foodLevel <= 0 {
            self.foodTimer += 1;
            if self.foodTimer >= 80 {
                if health > 10.0
                    || difficulty == EnumDifficulty::Hard
                    || (health > 1.0 && difficulty == EnumDifficulty::Normal)
                {
                    self.foodTimer = 0;
                    return FoodStatsAction::Starve(1.0);
                }
                self.foodTimer = 0;
            }
        } else {
            self.foodTimer = 0;
        }
        FoodStatsAction::None
    }

    /// `FoodStats#readNBT`.
    pub fn readNBT(&mut self, tag: &crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound) {
        if tag.hasKey("foodLevel") {
            self.foodLevel = tag.getInteger("foodLevel");
            self.foodTimer = tag.getInteger("foodTickTimer");
            self.foodSaturationLevel = tag.getFloat("foodSaturationLevel");
            self.foodExhaustionLevel = tag.getFloat("foodExhaustionLevel");
        }
    }

    /// `FoodStats#writeNBT`.
    pub fn writeNBT(&self, tag: &mut crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound) {
        tag.setInteger("foodLevel", self.foodLevel);
        tag.setInteger("foodTickTimer", self.foodTimer);
        tag.setFloat("foodSaturationLevel", self.foodSaturationLevel);
        tag.setFloat("foodExhaustionLevel", self.foodExhaustionLevel);
    }

    pub const fn getFoodLevel(&self) -> i32 { self.foodLevel }
    pub const fn getPrevFoodLevel(&self) -> i32 { self.prevFoodLevel }

    /// `FoodStats#needFood`.
    pub const fn needFood(&self) -> bool { self.foodLevel < 20 }

    /// `FoodStats#addExhaustion`: capped at 40.
    pub fn addExhaustion(&mut self, exhaustion: f32) {
        self.foodExhaustionLevel = (self.foodExhaustionLevel + exhaustion).min(40.0);
    }

    pub const fn getSaturationLevel(&self) -> f32 { self.foodSaturationLevel }
    pub const fn getExhaustionLevel(&self) -> f32 { self.foodExhaustionLevel }
    pub const fn getFoodTimer(&self) -> i32 { self.foodTimer }
    pub fn setFoodLevel(&mut self, value: i32) { self.foodLevel = value; }
    pub fn setFoodSaturationLevel(&mut self, value: f32) { self.foodSaturationLevel = value; }
    pub fn setFoodExhaustionLevel(&mut self, value: f32) { self.foodExhaustionLevel = value; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_stats_caps_at_twenty_and_keeps_saturation_below_food() {
        let mut stats = FoodStats::default();
        stats.addStats(5, 0.6);
        assert_eq!(stats.getFoodLevel(), 20);
        assert_eq!(stats.getSaturationLevel(), 5.0 + 5.0 * 0.6 * 2.0);

        let mut stats = FoodStats::default();
        stats.addStats(2, 0.6);
        assert_eq!(stats.getFoodLevel(), 20);
        // 5.0 + 2*0.6*2 = 7.4, capped at foodLevel 20.
        assert_eq!(stats.getSaturationLevel(), 7.4);
    }

    #[test]
    fn exhaustion_burns_saturation_before_food() {
        let mut stats = FoodStats::default();
        stats.setFoodSaturationLevel(3.0);
        stats.addExhaustion(4.1);
        assert!(matches!(stats.onUpdate(EnumDifficulty::Normal, true, true, 20.0), FoodStatsAction::None));
        assert_eq!(stats.getFoodLevel(), 20);
        assert_eq!(stats.getSaturationLevel(), 2.0);
        assert!((stats.getExhaustionLevel() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn full_saturation_regenerates_quickly_without_lowering_food() {
        let mut stats = FoodStats::default();
        stats.setFoodSaturationLevel(5.0);
        // Saturation heal fires on the 10th tick while foodLevel >= 20.
        for _ in 0..9 {
            assert!(matches!(stats.onUpdate(EnumDifficulty::Normal, true, true, 15.0), FoodStatsAction::None));
        }
        let action = stats.onUpdate(EnumDifficulty::Normal, true, true, 15.0);
        assert!(matches!(action, FoodStatsAction::Heal(_)));
        assert_eq!(stats.getFoodLevel(), 20);
        // Java adds `f = min(saturation, 6.0F)` to exhaustion, not the heal
        // amount `f / 6.0F`.
        assert_eq!(stats.getExhaustionLevel(), 5.0);
    }

    #[test]
    fn starvation_damage_requires_survival_conditions() {
        let mut stats = FoodStats::default();
        stats.setFoodLevel(0);
        stats.setFoodSaturationLevel(0.0);
        // The 80th tick fires the starvation damage.
        for _ in 0..79 {
            assert!(matches!(stats.onUpdate(EnumDifficulty::Normal, true, true, 15.0), FoodStatsAction::None));
        }
        assert!(matches!(stats.onUpdate(EnumDifficulty::Normal, true, true, 15.0), FoodStatsAction::Starve(_)));
        // Peaceful never starves: with health below the survival threshold
        // the timer resets without damage (the >10 health clause is
        // difficulty-independent, matching `FoodStats#onUpdate`).
        let mut peaceful = FoodStats::default();
        peaceful.setFoodLevel(0);
        peaceful.setFoodSaturationLevel(0.0);
        for _ in 0..80 {
            assert!(matches!(peaceful.onUpdate(EnumDifficulty::Peaceful, true, true, 5.0), FoodStatsAction::None));
        }
    }
}
