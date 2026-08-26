/// Direct semantic port of `net.minecraft.client.gui.ScaledResolution`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaledResolution {
    scaled_width_d: f64,
    scaled_height_d: f64,
    scaled_width: i32,
    scaled_height: i32,
    scale_factor: i32,
}

impl ScaledResolution {
    pub fn new(display_width: i32, display_height: i32, gui_scale: i32, unicode: bool) -> Self {
        let mut scale_factor = 1_i32;
        let mut requested_scale = gui_scale;
        if requested_scale == 0 {
            requested_scale = 1000;
        }

        while scale_factor < requested_scale
            && display_width / (scale_factor + 1) >= 320
            && display_height / (scale_factor + 1) >= 240
        {
            scale_factor += 1;
        }

        if unicode && scale_factor % 2 != 0 && scale_factor != 1 {
            scale_factor -= 1;
        }

        let scaled_width_d = display_width as f64 / scale_factor as f64;
        let scaled_height_d = display_height as f64 / scale_factor as f64;
        let scaled_width = scaled_width_d.ceil() as i32;
        let scaled_height = scaled_height_d.ceil() as i32;

        Self {
            scaled_width_d,
            scaled_height_d,
            scaled_width,
            scaled_height,
            scale_factor,
        }
    }

    pub const fn scaled_width(self) -> i32 {
        self.scaled_width
    }
    pub const fn scaled_height(self) -> i32 {
        self.scaled_height
    }
    pub const fn scaled_width_f64(self) -> f64 {
        self.scaled_width_d
    }
    pub const fn scaled_height_f64(self) -> f64 {
        self.scaled_height_d
    }
    pub const fn scale_factor(self) -> i32 {
        self.scale_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_scale_matches_vanilla_constraints() {
        let resolution = ScaledResolution::new(1920, 1080, 0, false);
        assert_eq!(resolution.scale_factor(), 4);
        assert_eq!(resolution.scaled_width(), 480);
        assert_eq!(resolution.scaled_height(), 270);
    }

    #[test]
    fn unicode_forces_even_scale_except_one() {
        let normal = ScaledResolution::new(960, 720, 3, false);
        let unicode = ScaledResolution::new(960, 720, 3, true);
        assert_eq!(normal.scale_factor(), 3);
        assert_eq!(unicode.scale_factor(), 2);
    }

    #[test]
    fn uses_java_ceil_for_non_integral_dimensions() {
        let resolution = ScaledResolution::new(1366, 768, 3, false);
        assert_eq!(resolution.scale_factor(), 3);
        assert_eq!(resolution.scaled_width(), 456);
        assert_eq!(resolution.scaled_height(), 256);
    }
}
