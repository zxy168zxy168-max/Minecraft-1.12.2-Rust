use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};

pub const SQRT_2: f32 = std::f32::consts::SQRT_2;
pub const PI: f32 = std::f32::consts::PI;
pub const PI2: f32 = std::f32::consts::PI * 2.0;
pub const PI_D2: f32 = std::f32::consts::PI / 2.0;
pub const DEG_2_RAD: f32 = 0.017_453_292;

static FAST_MATH: AtomicBool = AtomicBool::new(false);
static SIN_TABLE: Lazy<Box<[f32; 65_536]>> = Lazy::new(|| {
    let mut table = Box::new([0.0; 65_536]);
    for (index, value) in table.iter_mut().enumerate() {
        *value = ((index as f64) * std::f64::consts::PI * 2.0 / 65_536.0).sin() as f32;
    }
    table
});
static SIN_TABLE_FAST: Lazy<Box<[f32; 4_096]>> = Lazy::new(|| {
    let mut table = Box::new([0.0; 4_096]);
    for (index, value) in table.iter_mut().enumerate() {
        let angle = ((index as f32 + 0.5) / 4096.0 * (PI * 2.0)) as f64;
        *value = angle.sin() as f32;
    }
    for degrees in (0..360).step_by(90) {
        let index = ((degrees as f32 * 11.377_778) as i32 & 4095) as usize;
        table[index] = ((degrees as f32 * DEG_2_RAD) as f64).sin() as f32;
    }
    table
});

pub fn set_fast_math(enabled: bool) {
    FAST_MATH.store(enabled, Ordering::Relaxed);
}
pub fn fast_math() -> bool {
    FAST_MATH.load(Ordering::Relaxed)
}

#[inline]
pub fn sin(value: f32) -> f32 {
    if fast_math() {
        SIN_TABLE_FAST[((value * 651.8986) as i32 & 4095) as usize]
    } else {
        SIN_TABLE[((value * 10_430.378) as i32 & 65_535) as usize]
    }
}

#[inline]
pub fn cos(value: f32) -> f32 {
    if fast_math() {
        SIN_TABLE_FAST[(((value + PI_D2) * 651.8986) as i32 & 4095) as usize]
    } else {
        SIN_TABLE[((value * 10_430.378 + 16_384.0) as i32 & 65_535) as usize]
    }
}

#[inline]
pub fn floor_f32(value: f32) -> i32 {
    let integer = value as i32;
    if value < integer as f32 {
        integer - 1
    } else {
        integer
    }
}

#[inline]
pub fn floor_f64(value: f64) -> i32 {
    let integer = value as i32;
    if value < integer as f64 {
        integer - 1
    } else {
        integer
    }
}

#[inline]
pub fn floor_i64(value: f64) -> i64 {
    let integer = value as i64;
    if value < integer as f64 {
        integer - 1
    } else {
        integer
    }
}

#[inline]
pub fn ceil_f32(value: f32) -> i32 {
    let integer = value as i32;
    if value > integer as f32 {
        integer + 1
    } else {
        integer
    }
}

#[inline]
pub fn ceil_f64(value: f64) -> i32 {
    let integer = value as i32;
    if value > integer as f64 {
        integer + 1
    } else {
        integer
    }
}

#[inline]
pub fn clamp_i32(value: i32, minimum: i32, maximum: i32) -> i32 {
    value.clamp(minimum, maximum)
}
#[inline]
pub fn clamp_f32(value: f32, minimum: f32, maximum: f32) -> f32 {
    if value < minimum {
        minimum
    } else if value > maximum {
        maximum
    } else {
        value
    }
}
#[inline]
pub fn clamp_f64(value: f64, minimum: f64, maximum: f64) -> f64 {
    if value < minimum {
        minimum
    } else if value > maximum {
        maximum
    } else {
        value
    }
}
#[inline]
pub fn clamped_lerp(lower: f64, upper: f64, slide: f64) -> f64 {
    if slide < 0.0 {
        lower
    } else if slide > 1.0 {
        upper
    } else {
        lower + (upper - lower) * slide
    }
}
#[inline]
pub fn positive_modulo_f32(numerator: f32, denominator: f32) -> f32 {
    (numerator % denominator + denominator) % denominator
}
#[inline]
pub fn positive_modulo_f64(numerator: f64, denominator: f64) -> f64 {
    (numerator % denominator + denominator) % denominator
}
#[inline]
pub fn wrap_degrees_f32(value: f32) -> f32 {
    let mut wrapped = value % 360.0;
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    wrapped
}
#[inline]
pub fn wrap_degrees_f64(value: f64) -> f64 {
    let mut wrapped = value % 360.0;
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    wrapped
}

pub const fn smallest_encompassing_power_of_two(value: i32) -> i32 {
    let mut result = value - 1;
    result |= result >> 1;
    result |= result >> 2;
    result |= result >> 4;
    result |= result >> 8;
    result |= result >> 16;
    result + 1
}

pub const fn is_power_of_two(value: i32) -> bool {
    value != 0 && (value & (value - 1)) == 0
}

pub fn log2_debruijn(value: i32) -> i32 {
    const TABLE: [i32; 32] = [
        0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8, 31, 27, 13, 23, 21, 19, 16, 7,
        26, 12, 18, 6, 11, 5, 10, 9,
    ];
    let normalized = if is_power_of_two(value) {
        value
    } else {
        smallest_encompassing_power_of_two(value)
    };
    TABLE[((((normalized as i64) * 125_613_361_i64) >> 27) & 31) as usize]
}

pub fn log2(value: i32) -> i32 {
    log2_debruijn(value) - if is_power_of_two(value) { 0 } else { 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minecraft_floor_semantics() {
        assert_eq!(floor_f64(-0.1), -1);
        assert_eq!(floor_f64(-1.0), -1);
        assert_eq!(floor_f64(1.9), 1);
    }

    #[test]
    fn angle_wrapping_matches_minecraft() {
        assert_eq!(wrap_degrees_f32(181.0), -179.0);
        assert_eq!(wrap_degrees_f32(-181.0), 179.0);
    }

    #[test]
    fn block_position_bit_width_inputs_match_mcp() {
        assert_eq!(log2(smallest_encompassing_power_of_two(30_000_000)), 25);
    }
}
