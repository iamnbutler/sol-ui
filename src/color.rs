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
    pub const YELLOW: Color = Srgba::new(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: Color = Srgba::new(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: Color = Srgba::new(1.0, 0.0, 1.0, 1.0);
    pub const ORANGE: Color = Srgba::new(1.0, 0.5, 0.0, 1.0);
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
    /// Create a color from a hex string (e.g., "#ff0000", "#f00", "#ff0000ff")
    ///
    /// Supports formats:
    /// - `#RGB` (3 chars) - shorthand, expands to RRGGBB
    /// - `#RGBA` (4 chars) - shorthand with alpha
    /// - `#RRGGBB` (6 chars) - standard hex
    /// - `#RRGGBBAA` (8 chars) - with alpha
    ///
    /// The `#` prefix is optional.
    ///
    /// # Panics
    /// Panics if the hex string is invalid. Use `try_hex` for fallible parsing.
    fn hex(hex: &str) -> Self;
    /// Try to create a color from a hex string, returning None if invalid
    fn try_hex(hex: &str) -> Option<Self>
    where
        Self: Sized;
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

    fn hex(hex: &str) -> Self {
        Self::try_hex(hex).unwrap_or_else(|| panic!("Invalid hex color: {}", hex))
    }

    fn try_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);

        let (r, g, b, a) = match hex.len() {
            // #RGB - shorthand
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                // Expand: 0xF -> 0xFF (multiply by 17)
                (r * 17, g * 17, b * 17, 255)
            }
            // #RGBA - shorthand with alpha
            4 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                let a = u8::from_str_radix(&hex[3..4], 16).ok()?;
                (r * 17, g * 17, b * 17, a * 17)
            }
            // #RRGGBB - standard
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                (r, g, b, 255)
            }
            // #RRGGBBAA - with alpha
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                (r, g, b, a)
            }
            _ => return None,
        };

        Some(Srgba::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ))
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
