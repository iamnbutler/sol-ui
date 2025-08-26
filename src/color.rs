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
    use super::colors::*;

    #[test]
    fn test_color_ext_rgb() {
        let color = Color::rgb(0.5, 0.7, 0.9);
        assert_eq!(color.red, 0.5);
        assert_eq!(color.green, 0.7);
        assert_eq!(color.blue, 0.9);
        assert_eq!(color.alpha, 1.0);
    }

    #[test]
    fn test_color_ext_rgba() {
        let color = Color::rgba(0.1, 0.2, 0.3, 0.5);
        assert_eq!(color.red, 0.1);
        assert_eq!(color.green, 0.2);
        assert_eq!(color.blue, 0.3);
        assert_eq!(color.alpha, 0.5);
    }

    #[test]
    fn test_color_ext_with_alpha() {
        let base_color = Color::rgb(0.6, 0.7, 0.8);
        let modified_color = base_color.with_alpha(0.3);
        
        assert_eq!(modified_color.red, 0.6);
        assert_eq!(modified_color.green, 0.7);
        assert_eq!(modified_color.blue, 0.8);
        assert_eq!(modified_color.alpha, 0.3);
    }

    #[test]
    fn test_color_ext_as_u8_arr() {
        let color = Color::rgba(1.0, 0.5, 0.0, 0.75);
        let u8_arr = color.as_u8_arr();
        
        assert_eq!(u8_arr[0], 255); // Red: 1.0 * 255
        assert_eq!(u8_arr[1], 127); // Green: 0.5 * 255 (rounded)
        assert_eq!(u8_arr[2], 0);   // Blue: 0.0 * 255
        assert_eq!(u8_arr[3], 191); // Alpha: 0.75 * 255 (rounded)
    }

    #[test]
    fn test_color_ext_as_u8_arr_edge_cases() {
        // Test minimum values
        let black_transparent = Color::rgba(0.0, 0.0, 0.0, 0.0);
        let black_u8 = black_transparent.as_u8_arr();
        assert_eq!(black_u8, [0, 0, 0, 0]);

        // Test maximum values
        let white_opaque = Color::rgba(1.0, 1.0, 1.0, 1.0);
        let white_u8 = white_opaque.as_u8_arr();
        assert_eq!(white_u8, [255, 255, 255, 255]);
    }

    #[test]
    fn test_basic_color_constants() {
        assert_eq!(WHITE.red, 1.0);
        assert_eq!(WHITE.green, 1.0);
        assert_eq!(WHITE.blue, 1.0);
        assert_eq!(WHITE.alpha, 1.0);

        assert_eq!(BLACK.red, 0.0);
        assert_eq!(BLACK.green, 0.0);
        assert_eq!(BLACK.blue, 0.0);
        assert_eq!(BLACK.alpha, 1.0);

        assert_eq!(TRANSPARENT.red, 0.0);
        assert_eq!(TRANSPARENT.green, 0.0);
        assert_eq!(TRANSPARENT.blue, 0.0);
        assert_eq!(TRANSPARENT.alpha, 0.0);
    }

    #[test]
    fn test_primary_color_constants() {
        // Red
        assert_eq!(RED.red, 1.0);
        assert_eq!(RED.green, 0.0);
        assert_eq!(RED.blue, 0.0);
        assert_eq!(RED.alpha, 1.0);

        // Green
        assert_eq!(GREEN.red, 0.0);
        assert_eq!(GREEN.green, 1.0);
        assert_eq!(GREEN.blue, 0.0);
        assert_eq!(GREEN.alpha, 1.0);

        // Blue
        assert_eq!(BLUE.red, 0.0);
        assert_eq!(BLUE.green, 0.0);
        assert_eq!(BLUE.blue, 1.0);
        assert_eq!(BLUE.alpha, 1.0);
    }

    #[test]
    fn test_gray_scale_progression() {
        // Test that gray values are in descending order
        assert!(GRAY_100.red > GRAY_200.red);
        assert!(GRAY_200.red > GRAY_300.red);
        assert!(GRAY_300.red > GRAY_400.red);
        assert!(GRAY_400.red > GRAY_500.red);
        assert!(GRAY_500.red > GRAY_600.red);
        assert!(GRAY_600.red > GRAY_700.red);
        assert!(GRAY_700.red > GRAY_800.red);
        assert!(GRAY_800.red > GRAY_900.red);

        // Test that gray colors have equal RGB components
        assert_eq!(GRAY_500.red, GRAY_500.green);
        assert_eq!(GRAY_500.green, GRAY_500.blue);
        assert_eq!(GRAY_500.alpha, 1.0);
    }

    #[test]
    fn test_color_scale_consistency() {
        // Test blue scale progression (should get darker from 400 to 600)
        assert!(BLUE_400.blue > BLUE_500.blue);
        assert!(BLUE_500.blue > BLUE_600.blue);

        // Test red scale progression
        assert!(RED_400.red > RED_500.red);
        assert!(RED_500.red > RED_600.red);

        // Test green scale progression
        assert!(GREEN_400.green > GREEN_500.green);
        assert!(GREEN_500.green > GREEN_600.green);

        // Test purple scale progression
        assert!(PURPLE_400.blue > PURPLE_500.blue);
        assert!(PURPLE_500.blue > PURPLE_600.blue);
    }

    #[test]
    fn test_all_colors_have_full_alpha() {
        // All predefined colors should have full alpha except TRANSPARENT
        assert_eq!(WHITE.alpha, 1.0);
        assert_eq!(BLACK.alpha, 1.0);
        assert_eq!(RED.alpha, 1.0);
        assert_eq!(GREEN.alpha, 1.0);
        assert_eq!(BLUE.alpha, 1.0);
        assert_eq!(GRAY_500.alpha, 1.0);
        assert_eq!(BLUE_500.alpha, 1.0);
        assert_eq!(RED_500.alpha, 1.0);
        assert_eq!(GREEN_500.alpha, 1.0);
        assert_eq!(PURPLE_500.alpha, 1.0);
        assert_eq!(TRANSPARENT.alpha, 0.0);
    }

    #[test]
    fn test_color_range_validation() {
        // All color components should be in valid range [0.0, 1.0]
        let test_colors = [
            WHITE, BLACK, RED, GREEN, BLUE, TRANSPARENT,
            GRAY_100, GRAY_500, GRAY_900,
            BLUE_400, BLUE_500, BLUE_600,
            RED_400, RED_500, RED_600,
            GREEN_400, GREEN_500, GREEN_600,
            PURPLE_400, PURPLE_500, PURPLE_600,
        ];

        for color in &test_colors {
            assert!(color.red >= 0.0 && color.red <= 1.0, "Red component out of range: {}", color.red);
            assert!(color.green >= 0.0 && color.green <= 1.0, "Green component out of range: {}", color.green);
            assert!(color.blue >= 0.0 && color.blue <= 1.0, "Blue component out of range: {}", color.blue);
            assert!(color.alpha >= 0.0 && color.alpha <= 1.0, "Alpha component out of range: {}", color.alpha);
        }
    }

    #[test]
    fn test_color_ext_chaining() {
        // Test that methods can be chained effectively
        let color = Color::rgb(0.2, 0.4, 0.6).with_alpha(0.8);
        assert_eq!(color.red, 0.2);
        assert_eq!(color.green, 0.4);
        assert_eq!(color.blue, 0.6);
        assert_eq!(color.alpha, 0.8);
    }
}
