//! Compatibility helpers for Java semantics used by Minecraft 1.12.2.

use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Java `String.hashCode()` over UTF-16 code units.
///
/// Rust strings are UTF-8, while Java hashes UTF-16 units. Using
/// `encode_utf16()` is required for non-BMP characters.
pub fn string_hash_code(value: &str) -> i32 {
    value.encode_utf16().fold(0_i32, |hash, unit| {
        hash.wrapping_mul(31).wrapping_add(unit as i32)
    })
}

/// Equivalent to Java narrowing after an unsigned right shift.
pub const fn unsigned_right_shift_i32(value: i32, shift: u32) -> i32 {
    ((value as u32) >> (shift & 31)) as i32
}

/// Equivalent to Java narrowing after an unsigned right shift.
pub const fn unsigned_right_shift_i64(value: i64, shift: u32) -> i64 {
    ((value as u64) >> (shift & 63)) as i64
}

/// Faithful `java.util.Random` implementation.
///
/// Minecraft 1.12.2 relies on Java's 48-bit linear congruential generator.
/// Replacing this with a Rust RNG would alter particles, randomized models,
/// world-adjacent visual choices, and protocol-visible behavior.
#[derive(Debug, Clone, PartialEq)]
pub struct JavaRandom {
    seed: u64,
    nextNextGaussian: f64,
    haveNextNextGaussian: bool,
}

/// Equivalent ownership semantics for `java.lang.Math.random()`: one
/// process-global `java.util.Random` stream, distinct from `World#rand` and
/// `Item#itemRand`. Minecraft 1.12.2 uses it for effects such as the eight
/// Nether-water evaporation smoke positions.
pub fn math_random_f64() -> f64 {
    static MATH_RANDOM: OnceLock<Mutex<JavaRandom>> = OnceLock::new();
    let random = MATH_RANDOM.get_or_init(|| {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        Mutex::new(JavaRandom::new(seed))
    });
    random
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .next_f64()
}

impl JavaRandom {
    const MULTIPLIER: u64 = 0x5DEECE66D;
    const ADDEND: u64 = 0xB;
    const MASK: u64 = (1_u64 << 48) - 1;

    pub fn new(seed: i64) -> Self {
        let mut random = Self {
            seed: 0,
            nextNextGaussian: 0.0,
            haveNextNextGaussian: false,
        };
        random.set_seed(seed);
        random
    }

    pub fn set_seed(&mut self, seed: i64) {
        self.seed = ((seed as u64) ^ Self::MULTIPLIER) & Self::MASK;
        self.haveNextNextGaussian = false;
    }

    pub fn next_bits(&mut self, bits: u32) -> i32 {
        assert!(bits <= 32, "java.util.Random supports at most 32 bits");
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND)
            & Self::MASK;
        (self.seed >> (48 - bits)) as i32
    }

    pub fn next_i32(&mut self) -> i32 {
        self.next_bits(32)
    }

    pub fn next_i32_bound(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");

        if (bound & -bound) == bound {
            return (((bound as i64) * (self.next_bits(31) as i64)) >> 31) as i32;
        }

        loop {
            let bits = self.next_bits(31);
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                return value;
            }
        }
    }

    pub fn next_i64(&mut self) -> i64 {
        ((self.next_bits(32) as i64) << 32).wrapping_add(self.next_bits(32) as i64)
    }

    pub fn next_bool(&mut self) -> bool {
        self.next_bits(1) != 0
    }

    pub fn next_f32(&mut self) -> f32 {
        self.next_bits(24) as f32 / (1_u32 << 24) as f32
    }

    pub fn next_f64(&mut self) -> f64 {
        let high = (self.next_bits(26) as i64) << 27;
        let low = self.next_bits(27) as i64;
        (high + low) as f64 / (1_u64 << 53) as f64
    }

    /// Exact `java.util.Random#nextGaussian`, including the cached second
    /// Box-Muller sample and cache reset performed by `setSeed`.
    pub fn next_gaussian(&mut self) -> f64 {
        if self.haveNextNextGaussian {
            self.haveNextNextGaussian = false;
            return self.nextNextGaussian;
        }
        loop {
            let first = 2.0 * self.next_f64() - 1.0;
            let second = 2.0 * self.next_f64() - 1.0;
            let radiusSquared = first * first + second * second;
            if radiusSquared >= 1.0 || radiusSquared == 0.0 {
                continue;
            }
            let multiplier = (-2.0 * radiusSquared.ln() / radiusSquared).sqrt();
            self.nextNextGaussian = second * multiplier;
            self.haveNextNextGaussian = true;
            return first * multiplier;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_string_hash_matches_known_values() {
        assert_eq!(string_hash_code("minecraft"), 695073197);
        assert_eq!(string_hash_code("minecraft:stone"), -1133948840);
        assert_eq!(string_hash_code(""), 0);
    }

    #[test]
    fn java_random_matches_jdk_sequence() {
        let mut random = JavaRandom::new(0);
        assert_eq!(random.next_i32(), -1155484576);
        assert_eq!(random.next_i32(), -723955400);
        assert_eq!(random.next_i32(), 1033096058);
    }

    #[test]
    fn java_random_bound_matches_jdk_sequence() {
        let mut random = JavaRandom::new(12345);
        assert_eq!(random.next_i32_bound(100), 51);
        assert_eq!(random.next_i32_bound(100), 80);
        assert_eq!(random.next_i32_bound(100), 41);
    }

    #[test]
    fn java_random_gaussian_matches_jdk_sequence_and_cache_reset() {
        let mut random = JavaRandom::new(0);
        assert!((random.next_gaussian() - 0.8025330637390305).abs() < 1.0e-15);
        assert!((random.next_gaussian() + 0.9015460884175122).abs() < 1.0e-15);
        random.set_seed(0);
        assert!((random.next_gaussian() - 0.8025330637390305).abs() < 1.0e-15);
    }
}
