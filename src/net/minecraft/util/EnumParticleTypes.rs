/// Exact Minecraft 1.12.2 `EnumParticleTypes` registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnumParticleTypes {
    ExplosionNormal,
    ExplosionLarge,
    ExplosionHuge,
    FireworksSpark,
    WaterBubble,
    WaterSplash,
    WaterWake,
    Suspended,
    SuspendedDepth,
    Crit,
    CritMagic,
    SmokeNormal,
    SmokeLarge,
    Spell,
    SpellInstant,
    SpellMob,
    SpellMobAmbient,
    SpellWitch,
    DripWater,
    DripLava,
    VillagerAngry,
    VillagerHappy,
    TownAura,
    Note,
    Portal,
    EnchantmentTable,
    Flame,
    Lava,
    Footstep,
    Cloud,
    Redstone,
    Snowball,
    SnowShovel,
    Slime,
    Heart,
    Barrier,
    ItemCrack,
    BlockCrack,
    BlockDust,
    WaterDrop,
    ItemTake,
    MobAppearance,
    DragonBreath,
    EndRod,
    DamageIndicator,
    SweepAttack,
    FallingDust,
    Totem,
    Spit,
}

impl EnumParticleTypes {
    pub const VALUES: [Self; 49] = [
        Self::ExplosionNormal,
        Self::ExplosionLarge,
        Self::ExplosionHuge,
        Self::FireworksSpark,
        Self::WaterBubble,
        Self::WaterSplash,
        Self::WaterWake,
        Self::Suspended,
        Self::SuspendedDepth,
        Self::Crit,
        Self::CritMagic,
        Self::SmokeNormal,
        Self::SmokeLarge,
        Self::Spell,
        Self::SpellInstant,
        Self::SpellMob,
        Self::SpellMobAmbient,
        Self::SpellWitch,
        Self::DripWater,
        Self::DripLava,
        Self::VillagerAngry,
        Self::VillagerHappy,
        Self::TownAura,
        Self::Note,
        Self::Portal,
        Self::EnchantmentTable,
        Self::Flame,
        Self::Lava,
        Self::Footstep,
        Self::Cloud,
        Self::Redstone,
        Self::Snowball,
        Self::SnowShovel,
        Self::Slime,
        Self::Heart,
        Self::Barrier,
        Self::ItemCrack,
        Self::BlockCrack,
        Self::BlockDust,
        Self::WaterDrop,
        Self::ItemTake,
        Self::MobAppearance,
        Self::DragonBreath,
        Self::EndRod,
        Self::DamageIndicator,
        Self::SweepAttack,
        Self::FallingDust,
        Self::Totem,
        Self::Spit,
    ];

    pub const fn particleId(self) -> i32 {
        self as i32
    }

    pub const fn particleName(self) -> &'static str {
        match self {
            Self::ExplosionNormal => "explode",
            Self::ExplosionLarge => "largeexplode",
            Self::ExplosionHuge => "hugeexplosion",
            Self::FireworksSpark => "fireworksSpark",
            Self::WaterBubble => "bubble",
            Self::WaterSplash => "splash",
            Self::WaterWake => "wake",
            Self::Suspended => "suspended",
            Self::SuspendedDepth => "depthsuspend",
            Self::Crit => "crit",
            Self::CritMagic => "magicCrit",
            Self::SmokeNormal => "smoke",
            Self::SmokeLarge => "largesmoke",
            Self::Spell => "spell",
            Self::SpellInstant => "instantSpell",
            Self::SpellMob => "mobSpell",
            Self::SpellMobAmbient => "mobSpellAmbient",
            Self::SpellWitch => "witchMagic",
            Self::DripWater => "dripWater",
            Self::DripLava => "dripLava",
            Self::VillagerAngry => "angryVillager",
            Self::VillagerHappy => "happyVillager",
            Self::TownAura => "townaura",
            Self::Note => "note",
            Self::Portal => "portal",
            Self::EnchantmentTable => "enchantmenttable",
            Self::Flame => "flame",
            Self::Lava => "lava",
            Self::Footstep => "footstep",
            Self::Cloud => "cloud",
            Self::Redstone => "reddust",
            Self::Snowball => "snowballpoof",
            Self::SnowShovel => "snowshovel",
            Self::Slime => "slime",
            Self::Heart => "heart",
            Self::Barrier => "barrier",
            Self::ItemCrack => "iconcrack",
            Self::BlockCrack => "blockcrack",
            Self::BlockDust => "blockdust",
            Self::WaterDrop => "droplet",
            Self::ItemTake => "take",
            Self::MobAppearance => "mobappearance",
            Self::DragonBreath => "dragonbreath",
            Self::EndRod => "endRod",
            Self::DamageIndicator => "damageIndicator",
            Self::SweepAttack => "sweepAttack",
            Self::FallingDust => "fallingdust",
            Self::Totem => "totem",
            Self::Spit => "spit",
        }
    }

    pub const fn argumentCount(self) -> usize {
        match self {
            Self::ItemCrack => 2,
            Self::BlockCrack | Self::BlockDust | Self::FallingDust => 1,
            _ => 0,
        }
    }

    pub const fn shouldIgnoreRange(self) -> bool {
        matches!(
            self,
            Self::ExplosionNormal
                | Self::ExplosionLarge
                | Self::ExplosionHuge
                | Self::MobAppearance
                | Self::DamageIndicator
                | Self::SweepAttack
                | Self::Spit
        )
    }

    pub const fn fromId(id: i32) -> Option<Self> {
        if id >= 0 && id < Self::VALUES.len() as i32 {
            Some(Self::VALUES[id as usize])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protocol_ids_and_arguments_match_1122() {
        assert_eq!(EnumParticleTypes::WaterBubble.particleId(), 4);
        assert_eq!(EnumParticleTypes::SmokeNormal.particleId(), 11);
        assert_eq!(EnumParticleTypes::SpellMob.particleId(), 15);
        assert_eq!(EnumParticleTypes::DragonBreath.particleId(), 42);
        assert_eq!(EnumParticleTypes::EndRod.particleId(), 43);
        assert_eq!(EnumParticleTypes::ItemCrack.argumentCount(), 2);
        assert_eq!(EnumParticleTypes::fromId(48), Some(EnumParticleTypes::Spit));
        assert_eq!(EnumParticleTypes::fromId(49), None);
    }
}
