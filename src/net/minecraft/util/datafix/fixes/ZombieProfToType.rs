use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::compat::Java::JavaRandom;
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

/// MCP 1.12.2 `ZombieProfToType` (DataVersion 502).
pub struct ZombieProfToType;

fn random() -> &'static Mutex<JavaRandom> {
    // Java's `new Random()` is likewise process-global here. Its JDK seed uses
    // a seed-uniquifier and nanoTime; wall-clock nanos are the Rust-equivalent
    // entropy source, while all generated values still use java.util.Random's
    // exact 48-bit LCG through `JavaRandom`.
    static RANDOM: OnceLock<Mutex<JavaRandom>> = OnceLock::new();
    RANDOM.get_or_init(|| {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        Mutex::new(JavaRandom::new(seed))
    })
}

impl ZombieProfToType {
    fn professionToType(value: i32) -> i32 {
        if (0..6).contains(&value) {
            value
        } else {
            -1
        }
    }
}
impl IFixableData for ZombieProfToType {
    fn getFixVersion(&self) -> i32 {
        502
    }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        if compound.getString("id") == "Zombie" && compound.getBoolean("IsVillager") {
            if !compound.hasKeyWithType("ZombieType", 99) {
                let mut value = -1;
                if compound.hasKeyWithType("VillagerProfession", 99) {
                    value = Self::professionToType(compound.getInteger("VillagerProfession"));
                }
                if value == -1 {
                    let roll = random()
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .next_i32_bound(6);
                    value = Self::professionToType(roll);
                }
                compound.setInteger("ZombieType", value);
            }
            compound.removeTag("IsVillager");
        }
        compound
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn villager_profession_is_preserved_as_zombie_type() {
        let mut zombie = NBTTagCompound::new();
        zombie.setString("id", "Zombie");
        zombie.setBoolean("IsVillager", true);
        zombie.setInteger("VillagerProfession", 4);
        let fixed = ZombieProfToType.fixTagCompound(zombie);
        assert_eq!(fixed.getInteger("ZombieType"), 4);
        assert!(!fixed.hasKey("IsVillager"));
    }
}
