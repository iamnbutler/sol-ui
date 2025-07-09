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
}

/// Helper trait for Color
pub trait ColorExt {
    fn rgb(r: f32, g: f32, b: f32) -> Self;
    fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self;
    fn with_alpha(self, alpha: f32) -> Self;
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
}
