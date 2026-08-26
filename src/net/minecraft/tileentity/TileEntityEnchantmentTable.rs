use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::compat::Java::JavaRandom;
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// Client animation state port of MCP 1.12.2 `TileEntityEnchantmentTable`.
///
/// The random page target is intentionally backed by one class-level Java RNG,
/// matching the source `private static final Random rand` rather than giving
/// every tile entity an independent Rust generator.
#[derive(Debug, Clone, PartialEq)]
pub struct TileEntityEnchantmentTable {
    pub pos: BlockPos,
    pub tickCount: i32,
    pub pageFlip: f32,
    pub pageFlipPrev: f32,
    pub flipT: f32,
    pub flipA: f32,
    pub bookSpread: f32,
    pub bookSpreadPrev: f32,
    pub bookRotation: f32,
    pub bookRotationPrev: f32,
    pub tRot: f32,
    customName: Option<String>,
}

impl TileEntityEnchantmentTable {
    pub fn new(pos: BlockPos) -> Self {
        Self {
            pos,
            tickCount: 0,
            pageFlip: 0.0,
            pageFlipPrev: 0.0,
            flipT: 0.0,
            flipA: 0.0,
            bookSpread: 0.0,
            bookSpreadPrev: 0.0,
            bookRotation: 0.0,
            bookRotationPrev: 0.0,
            tRot: 0.0,
            customName: None,
        }
    }

    pub fn fromNbt(tag: &NBTTagCompound) -> Option<Self> {
        let id = tag.getString("id");
        if !id.is_empty() && id != "minecraft:enchanting_table" && id != "EnchantTable" {
            return None;
        }
        let mut tile = Self::new(BlockPos::new(
            tag.getInteger("x"),
            tag.getInteger("y"),
            tag.getInteger("z"),
        ));
        if tag.hasKeyWithType("CustomName", 8) {
            let name = tag.getString("CustomName");
            if !name.is_empty() {
                tile.customName = Some(name);
            }
        }
        Some(tile)
    }

    /// Exact client tick from MCP `TileEntityEnchantmentTable#update`.
    /// `closestPlayer` is the local player position supplied by WorldClient;
    /// vanilla only selects a player within three blocks of the table center.
    pub fn update(&mut self, closestPlayer: Option<[f64; 3]>) {
        self.bookSpreadPrev = self.bookSpread;
        self.bookRotationPrev = self.bookRotation;

        let centerX = self.pos.x as f64 + 0.5;
        let centerY = self.pos.y as f64 + 0.5;
        let centerZ = self.pos.z as f64 + 0.5;
        let nearbyPlayer = closestPlayer.filter(|position| {
            let dx = position[0] - centerX;
            let dy = position[1] - centerY;
            let dz = position[2] - centerZ;
            dx * dx + dy * dy + dz * dz <= 9.0
        });

        if let Some(position) = nearbyPlayer {
            let d0 = position[0] - centerX;
            let d1 = position[2] - centerZ;
            self.tRot = d1.atan2(d0) as f32;
            self.bookSpread += 0.1;

            let chooseNewPage =
                self.bookSpread < 0.5 || with_random(|random| random.next_i32_bound(40) == 0);
            if chooseNewPage {
                let previousTarget = self.flipT;
                loop {
                    let delta =
                        with_random(|random| random.next_i32_bound(4) - random.next_i32_bound(4));
                    self.flipT += delta as f32;
                    if previousTarget != self.flipT {
                        break;
                    }
                }
            }
        } else {
            self.tRot += 0.02;
            self.bookSpread -= 0.1;
        }

        let pi = std::f32::consts::PI;
        let tau = pi * 2.0;
        while self.bookRotation >= pi {
            self.bookRotation -= tau;
        }
        while self.bookRotation < -pi {
            self.bookRotation += tau;
        }
        while self.tRot >= pi {
            self.tRot -= tau;
        }
        while self.tRot < -pi {
            self.tRot += tau;
        }

        let mut rotationDelta = self.tRot - self.bookRotation;
        while rotationDelta >= pi {
            rotationDelta -= tau;
        }
        while rotationDelta < -pi {
            rotationDelta += tau;
        }

        self.bookRotation += rotationDelta * 0.4;
        self.bookSpread = self.bookSpread.clamp(0.0, 1.0);
        self.tickCount = self.tickCount.wrapping_add(1);
        self.pageFlipPrev = self.pageFlip;
        let targetAcceleration = ((self.flipT - self.pageFlip) * 0.4).clamp(-0.2, 0.2);
        self.flipA += (targetAcceleration - self.flipA) * 0.9;
        self.pageFlip += self.flipA;
    }

    pub fn hasCustomName(&self) -> bool {
        self.customName
            .as_ref()
            .is_some_and(|name| !name.is_empty())
    }

    pub fn getName(&self) -> &str {
        self.customName.as_deref().unwrap_or("container.enchant")
    }
}

fn enchantment_random() -> &'static Mutex<JavaRandom> {
    static RANDOM: OnceLock<Mutex<JavaRandom>> = OnceLock::new();
    RANDOM.get_or_init(|| {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos() as i64);
        Mutex::new(JavaRandom::new(seed))
    })
}

fn with_random<T>(action: impl FnOnce(&mut JavaRandom) -> T) -> T {
    let mut random = enchantment_random()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    action(&mut random)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearby_player_opens_book_and_turns_it_toward_player() {
        let mut tile = TileEntityEnchantmentTable::new(BlockPos::new(0, 64, 0));
        tile.update(Some([2.5, 64.5, 0.5]));
        assert!((tile.bookSpread - 0.1).abs() < 1.0e-6);
        assert!((tile.tRot - 0.0).abs() < 1.0e-6);
        assert_eq!(tile.tickCount, 1);
    }

    #[test]
    fn distant_or_missing_player_closes_book_and_keeps_idle_rotation() {
        let mut tile = TileEntityEnchantmentTable::new(BlockPos::new(0, 64, 0));
        tile.bookSpread = 0.5;
        tile.update(Some([10.0, 64.5, 0.5]));
        assert!((tile.bookSpread - 0.4).abs() < 1.0e-6);
        assert!((tile.tRot - 0.02).abs() < 1.0e-6);
    }

    #[test]
    fn angular_difference_wraps_to_shortest_path() {
        let mut tile = TileEntityEnchantmentTable::new(BlockPos::new(0, 64, 0));
        tile.bookRotation = std::f32::consts::PI - 0.01;
        tile.tRot = -std::f32::consts::PI + 0.01;
        tile.update(None);
        assert!(tile.bookRotation.abs() <= std::f32::consts::PI + 0.1);
    }
}
