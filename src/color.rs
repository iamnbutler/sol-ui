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
        // Test basic color constants
        assert_eq!(colors::WHITE, Srgba::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(colors::BLACK, Srgba::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(colors::RED, Srgba::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(colors::GREEN, Srgba::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(colors::BLUE, Srgba::new(0.0, 0.0, 1.0, 1.0));
        assert_eq!(colors::TRANSPARENT, Srgba::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_gray_scale_constants() {
        // Test gray scale progression
        assert_eq!(colors::GRAY_100, Srgba::new(0.95, 0.95, 0.95, 1.0));
        assert_eq!(colors::GRAY_500, Srgba::new(0.6, 0.6, 0.6, 1.0));
        assert_eq!(colors::GRAY_900, Srgba::new(0.2, 0.2, 0.2, 1.0));
        
        // Ensure gray scale is monotonic (darker as numbers increase)
        assert!(colors::GRAY_100.red > colors::GRAY_200.red);
        assert!(colors::GRAY_200.red > colors::GRAY_300.red);
        assert!(colors::GRAY_800.red > colors::GRAY_900.red);
    }

    #[test]
    fn test_colored_scale_constants() {
        // Test blue scale
        assert_eq!(colors::BLUE_400, Srgba::new(0.38, 0.56, 0.95, 1.0));
        assert_eq!(colors::BLUE_500, Srgba::new(0.24, 0.48, 0.93, 1.0));
        assert_eq!(colors::BLUE_600, Srgba::new(0.15, 0.38, 0.85, 1.0));

        // Test red scale
        assert_eq!(colors::RED_400, Srgba::new(0.95, 0.44, 0.44, 1.0));
        assert_eq!(colors::RED_500, Srgba::new(0.93, 0.31, 0.31, 1.0));
        assert_eq!(colors::RED_600, Srgba::new(0.85, 0.22, 0.22, 1.0));

        // Test green scale
        assert_eq!(colors::GREEN_400, Srgba::new(0.48, 0.82, 0.5, 1.0));
        assert_eq!(colors::GREEN_500, Srgba::new(0.35, 0.76, 0.38, 1.0));
        assert_eq!(colors::GREEN_600, Srgba::new(0.28, 0.68, 0.31, 1.0));

        // Test purple scale
        assert_eq!(colors::PURPLE_400, Srgba::new(0.67, 0.51, 0.86, 1.0));
        assert_eq!(colors::PURPLE_500, Srgba::new(0.58, 0.4, 0.83, 1.0));
        assert_eq!(colors::PURPLE_600, Srgba::new(0.49, 0.32, 0.77, 1.0));
    }

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
        let color = Color::rgba(0.2, 0.4, 0.6, 0.8);
        assert_eq!(color.red, 0.2);
        assert_eq!(color.green, 0.4);
        assert_eq!(color.blue, 0.6);
        assert_eq!(color.alpha, 0.8);
    }

    #[test]
    fn test_color_ext_with_alpha() {
        let original = colors::RED;
        let with_alpha = original.with_alpha(0.5);
        
        assert_eq!(with_alpha.red, original.red);
        assert_eq!(with_alpha.green, original.green);
        assert_eq!(with_alpha.blue, original.blue);
        assert_eq!(with_alpha.alpha, 0.5);
    }

    #[test]
    fn test_color_ext_as_u8_arr() {
        let color = Color::rgba(1.0, 0.5, 0.0, 0.25);
        let u8_arr = color.as_u8_arr();
        
        assert_eq!(u8_arr, [255, 127, 0, 63]);
    }

    #[test]
    fn test_color_ext_as_u8_arr_edge_cases() {
        // Test pure white
        let white = colors::WHITE.as_u8_arr();
        assert_eq!(white, [255, 255, 255, 255]);

        // Test pure black
        let black = colors::BLACK.as_u8_arr();
        assert_eq!(black, [0, 0, 0, 255]);

        // Test transparent
        let transparent = colors::TRANSPARENT.as_u8_arr();
        assert_eq!(transparent, [0, 0, 0, 0]);
    }

    #[test]
    fn test_color_type_alias() {
        // Ensure Color type alias works correctly
        let color: Color = Srgba::new(0.1, 0.2, 0.3, 0.4);
        assert_eq!(color.red, 0.1);
        assert_eq!(color.green, 0.2);
        assert_eq!(color.blue, 0.3);
        assert_eq!(color.alpha, 0.4);
    }

    #[test]
    fn test_all_constants_have_full_alpha() {
        // All named color constants should have full alpha except TRANSPARENT
        assert_eq!(colors::WHITE.alpha, 1.0);
        assert_eq!(colors::BLACK.alpha, 1.0);
        assert_eq!(colors::RED.alpha, 1.0);
        assert_eq!(colors::GREEN.alpha, 1.0);
        assert_eq!(colors::BLUE.alpha, 1.0);
        
        // Gray scale
        assert_eq!(colors::GRAY_100.alpha, 1.0);
        assert_eq!(colors::GRAY_500.alpha, 1.0);
        assert_eq!(colors::GRAY_900.alpha, 1.0);
        
        // Colored scales
        assert_eq!(colors::BLUE_400.alpha, 1.0);
        assert_eq!(colors::RED_500.alpha, 1.0);
        assert_eq!(colors::GREEN_600.alpha, 1.0);
        assert_eq!(colors::PURPLE_400.alpha, 1.0);
        
        // Transparent is the exception
        assert_eq!(colors::TRANSPARENT.alpha, 0.0);
    }
}
