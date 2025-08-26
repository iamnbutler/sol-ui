use palette::Srgba;

/// Re-export palette's Srgba as our Color type
pub type Color = Srgba;

/// Common color constants
pub mod colors {
    use super::*;

    pub const WHITE: Color = Srgba::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Color = Srgba::new(0.0, 0.0, 0.0, 1.0);
    pub const RED: Color = Srgba::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Color = Srgba::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Color = Srgba::new(0.0, 0.0, 1.0, 1.0);
    pub const TRANSPARENT: Color = Srgba::new(0.0, 0.0, 0.0, 0.0);

    // Gray scale
    pub const GRAY_100: Color = Srgba::new(0.95, 0.95, 0.95, 1.0);
    pub const GRAY_200: Color = Srgba::new(0.9, 0.9, 0.9, 1.0);
    pub const GRAY_300: Color = Srgba::new(0.8, 0.8, 0.8, 1.0);
    pub const GRAY_400: Color = Srgba::new(0.7, 0.7, 0.7, 1.0);
    pub const GRAY_500: Color = Srgba::new(0.6, 0.6, 0.6, 1.0);
    pub const GRAY_600: Color = Srgba::new(0.5, 0.5, 0.5, 1.0);
    pub const GRAY_700: Color = Srgba::new(0.4, 0.4, 0.4, 1.0);
    pub const GRAY_800: Color = Srgba::new(0.3, 0.3, 0.3, 1.0);
    pub const GRAY_900: Color = Srgba::new(0.2, 0.2, 0.2, 1.0);

    // Blue scale
    pub const BLUE_400: Color = Srgba::new(0.38, 0.56, 0.95, 1.0);
    pub const BLUE_500: Color = Srgba::new(0.24, 0.48, 0.93, 1.0);
    pub const BLUE_600: Color = Srgba::new(0.15, 0.38, 0.85, 1.0);

    // Red scale
    pub const RED_400: Color = Srgba::new(0.95, 0.44, 0.44, 1.0);
    pub const RED_500: Color = Srgba::new(0.93, 0.31, 0.31, 1.0);
    pub const RED_600: Color = Srgba::new(0.85, 0.22, 0.22, 1.0);

    // Green scale
    pub const GREEN_400: Color = Srgba::new(0.48, 0.82, 0.5, 1.0);
    pub const GREEN_500: Color = Srgba::new(0.35, 0.76, 0.38, 1.0);
    pub const GREEN_600: Color = Srgba::new(0.28, 0.68, 0.31, 1.0);

    // Purple scale
    pub const PURPLE_400: Color = Srgba::new(0.67, 0.51, 0.86, 1.0);
    pub const PURPLE_500: Color = Srgba::new(0.58, 0.4, 0.83, 1.0);
    pub const PURPLE_600: Color = Srgba::new(0.49, 0.32, 0.77, 1.0);
}

/// Helper trait for Color
pub trait ColorExt {
    fn rgb(r: f32, g: f32, b: f32) -> Self;
    fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self;
    fn with_alpha(self, alpha: f32) -> Self;
    fn as_u8_arr(&self) -> [u8; 4];
}

impl ColorExt for Color {
    fn rgb(r: f32, g: f32, b: f32) -> Self {
        Srgba::new(r, g, b, 1.0)
    }

    fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Srgba::new(r, g, b, a)
    }

    fn with_alpha(self, alpha: f32) -> Self {
        Srgba::new(self.red, self.green, self.blue, alpha)
    }

    fn as_u8_arr(&self) -> [u8; 4] {
        [
            (self.red * 255.0) as u8,
            (self.green * 255.0) as u8,
            (self.blue * 255.0) as u8,
            (self.alpha * 255.0) as u8,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_constants() {
        assert_eq!(colors::WHITE, Srgba::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(colors::BLACK, Srgba::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(colors::RED, Srgba::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(colors::GREEN, Srgba::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(colors::BLUE, Srgba::new(0.0, 0.0, 1.0, 1.0));
        assert_eq!(colors::TRANSPARENT, Srgba::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_gray_scale_constants() {
        assert_eq!(colors::GRAY_100, Srgba::new(0.95, 0.95, 0.95, 1.0));
        assert_eq!(colors::GRAY_500, Srgba::new(0.6, 0.6, 0.6, 1.0));
        assert_eq!(colors::GRAY_900, Srgba::new(0.2, 0.2, 0.2, 1.0));
    }

    #[test]
    fn test_blue_scale_constants() {
        assert_eq!(colors::BLUE_400, Srgba::new(0.38, 0.56, 0.95, 1.0));
        assert_eq!(colors::BLUE_500, Srgba::new(0.24, 0.48, 0.93, 1.0));
        assert_eq!(colors::BLUE_600, Srgba::new(0.15, 0.38, 0.85, 1.0));
    }

    #[test]
    fn test_red_scale_constants() {
        assert_eq!(colors::RED_400, Srgba::new(0.95, 0.44, 0.44, 1.0));
        assert_eq!(colors::RED_500, Srgba::new(0.93, 0.31, 0.31, 1.0));
        assert_eq!(colors::RED_600, Srgba::new(0.85, 0.22, 0.22, 1.0));
    }

    #[test]
    fn test_green_scale_constants() {
        assert_eq!(colors::GREEN_400, Srgba::new(0.48, 0.82, 0.5, 1.0));
        assert_eq!(colors::GREEN_500, Srgba::new(0.35, 0.76, 0.38, 1.0));
        assert_eq!(colors::GREEN_600, Srgba::new(0.28, 0.68, 0.31, 1.0));
    }

    #[test]
    fn test_purple_scale_constants() {
        assert_eq!(colors::PURPLE_400, Srgba::new(0.67, 0.51, 0.86, 1.0));
        assert_eq!(colors::PURPLE_500, Srgba::new(0.58, 0.4, 0.83, 1.0));
        assert_eq!(colors::PURPLE_600, Srgba::new(0.49, 0.32, 0.77, 1.0));
    }

    #[test]
    fn test_color_ext_rgb() {
        let color = Color::rgb(0.5, 0.7, 0.3);
        assert_eq!(color.red, 0.5);
        assert_eq!(color.green, 0.7);
        assert_eq!(color.blue, 0.3);
        assert_eq!(color.alpha, 1.0);
    }

    #[test]
    fn test_color_ext_rgba() {
        let color = Color::rgba(0.2, 0.4, 0.6, 0.8);
        assert_eq!(color.red, 0.2);
        assert_eq!(color.green, 0.4);
        assert_eq!(color.blue, 0.6);
        assert_eq!(color.alpha, 0.8);
    }

    #[test]
    fn test_color_ext_with_alpha() {
        let original = Color::rgb(0.5, 0.7, 0.3);
        let with_alpha = original.with_alpha(0.6);

        assert_eq!(with_alpha.red, 0.5);
        assert_eq!(with_alpha.green, 0.7);
        assert_eq!(with_alpha.blue, 0.3);
        assert_eq!(with_alpha.alpha, 0.6);

        // Original should remain unchanged
        assert_eq!(original.alpha, 1.0);
    }

    #[test]
    fn test_color_ext_as_u8_arr() {
        let color = Color::rgba(0.5, 0.25, 0.75, 1.0);
        let arr = color.as_u8_arr();

        assert_eq!(arr[0], 127); // 0.5 * 255 ≈ 127
        assert_eq!(arr[1], 63); // 0.25 * 255 ≈ 63
        assert_eq!(arr[2], 191); // 0.75 * 255 ≈ 191
        assert_eq!(arr[3], 255); // 1.0 * 255 = 255
    }

    #[test]
    fn test_color_ext_as_u8_arr_edge_cases() {
        // Test minimum values
        let black = Color::rgba(0.0, 0.0, 0.0, 0.0);
        let black_arr = black.as_u8_arr();
        assert_eq!(black_arr, [0, 0, 0, 0]);

        // Test maximum values
        let white = Color::rgba(1.0, 1.0, 1.0, 1.0);
        let white_arr = white.as_u8_arr();
        assert_eq!(white_arr, [255, 255, 255, 255]);
    }

    #[test]
    fn test_color_type_alias() {
        // Test that our Color type alias works correctly
        let color1: Color = Srgba::new(0.1, 0.2, 0.3, 0.4);
        let color2: Color = Color::rgba(0.1, 0.2, 0.3, 0.4);

        assert_eq!(color1, color2);
    }

    #[test]
    fn test_color_component_access() {
        let color = Color::rgba(0.1, 0.2, 0.3, 0.4);

        assert_eq!(color.red, 0.1);
        assert_eq!(color.green, 0.2);
        assert_eq!(color.blue, 0.3);
        assert_eq!(color.alpha, 0.4);
    }

    #[test]
    fn test_color_precision() {
        // Test that color values maintain precision
        let precise_color = Color::rgba(0.123456, 0.789012, 0.345678, 0.901234);

        assert!((precise_color.red - 0.123456).abs() < f32::EPSILON);
        assert!((precise_color.green - 0.789012).abs() < f32::EPSILON);
        assert!((precise_color.blue - 0.345678).abs() < f32::EPSILON);
        assert!((precise_color.alpha - 0.901234).abs() < f32::EPSILON);
    }
}
