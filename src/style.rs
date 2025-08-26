use glam::Vec2;

use crate::color::{
    colors::{BLACK, WHITE},
    Color,
};

/// Text styling information
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub size: f32,
    pub color: Color,
    // TODO: Add font family, weight, etc.
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: 16.0,
            color: WHITE,
        }
    }
}

/// Corner radii for a frame (top-left, top-right, bottom-right, bottom-left)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CornerRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadii {
    /// Create uniform corner radii
    pub fn uniform(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    /// Create corner radii with different values for each corner
    pub fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }
}

/// Shadow properties for frames
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    /// Offset in pixels (x, y)
    pub offset: Vec2,
    /// Blur radius in pixels
    pub blur: f32,
    /// Shadow color
    pub color: Color,
}

/// Background fill type for frames
#[derive(Debug, Clone, PartialEq)]
pub enum Fill {
    /// Solid color fill
    Solid(Color),
    /// Linear gradient fill
    LinearGradient {
        /// Start color
        start: Color,
        /// End color
        end: Color,
        /// Angle in radians (0 = left to right, PI/2 = bottom to top)
        angle: f32,
    },
    /// Radial gradient fill
    RadialGradient {
        /// Center color
        center: Color,
        /// Edge color
        edge: Color,
    },
}

/// Frame styling information for SDF-based rendering
#[derive(Debug, Clone, PartialEq)]
pub struct ElementStyle {
    /// Background fill of the frame
    pub fill: Fill,
    /// Border width in pixels (0 for no border)
    pub border_width: f32,
    /// Border color
    pub border_color: Color,
    /// Corner radii
    pub corner_radii: CornerRadii,
    /// Optional shadow
    pub shadow: Option<Shadow>,
}

impl Default for ElementStyle {
    fn default() -> Self {
        Self {
            fill: Fill::Solid(WHITE),
            border_width: 0.0,
            border_color: BLACK,
            corner_radii: CornerRadii::uniform(0.0),
            shadow: None,
        }
    }
}

impl ElementStyle {
    /// Create a new frame style with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a solid background color
    pub fn with_background(mut self, color: Color) -> Self {
        self.fill = Fill::Solid(color);
        self
    }

    /// Set a linear gradient background
    pub fn with_linear_gradient(mut self, start: Color, end: Color, angle: f32) -> Self {
        self.fill = Fill::LinearGradient { start, end, angle };
        self
    }

    /// Set a radial gradient background
    pub fn with_radial_gradient(mut self, center: Color, edge: Color) -> Self {
        self.fill = Fill::RadialGradient { center, edge };
        self
    }

    /// Set the border width and color
    pub fn with_border(mut self, width: f32, color: Color) -> Self {
        self.border_width = width;
        self.border_color = color;
        self
    }

    /// Set uniform corner radius
    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radii = CornerRadii::uniform(radius);
        self
    }

    /// Set individual corner radii
    pub fn with_corner_radii(mut self, radii: CornerRadii) -> Self {
        self.corner_radii = radii;
        self
    }

    /// Add a shadow to the frame
    pub fn with_shadow(mut self, offset: Vec2, blur: f32, color: Color) -> Self {
        self.shadow = Some(Shadow {
            offset,
            blur,
            color,
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::colors::{BLUE, GREEN, RED};

    #[test]
    fn test_text_style_default() {
        let style = TextStyle::default();
        assert_eq!(style.size, 16.0);
        assert_eq!(style.color, WHITE);
    }

    #[test]
    fn test_corner_radii_uniform() {
        let radii = CornerRadii::uniform(5.0);
        assert_eq!(radii.top_left, 5.0);
        assert_eq!(radii.top_right, 5.0);
        assert_eq!(radii.bottom_right, 5.0);
        assert_eq!(radii.bottom_left, 5.0);
    }

    #[test]
    fn test_corner_radii_new() {
        let radii = CornerRadii::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(radii.top_left, 1.0);
        assert_eq!(radii.top_right, 2.0);
        assert_eq!(radii.bottom_right, 3.0);
        assert_eq!(radii.bottom_left, 4.0);
    }

    #[test]
    fn test_shadow_properties() {
        let shadow = Shadow {
            offset: Vec2::new(2.0, 3.0),
            blur: 5.0,
            color: RED,
        };

        assert_eq!(shadow.offset.x, 2.0);
        assert_eq!(shadow.offset.y, 3.0);
        assert_eq!(shadow.blur, 5.0);
        assert_eq!(shadow.color, RED);
    }

    #[test]
    fn test_fill_solid() {
        let fill = Fill::Solid(BLUE);
        match fill {
            Fill::Solid(color) => assert_eq!(color, BLUE),
            _ => panic!("Expected solid fill"),
        }
    }

    #[test]
    fn test_fill_linear_gradient() {
        let fill = Fill::LinearGradient {
            start: RED,
            end: BLUE,
            angle: std::f32::consts::PI / 4.0,
        };

        match fill {
            Fill::LinearGradient { start, end, angle } => {
                assert_eq!(start, RED);
                assert_eq!(end, BLUE);
                assert_eq!(angle, std::f32::consts::PI / 4.0);
            }
            _ => panic!("Expected linear gradient fill"),
        }
    }

    #[test]
    fn test_fill_radial_gradient() {
        let fill = Fill::RadialGradient {
            center: WHITE,
            edge: BLACK,
        };

        match fill {
            Fill::RadialGradient { center, edge } => {
                assert_eq!(center, WHITE);
                assert_eq!(edge, BLACK);
            }
            _ => panic!("Expected radial gradient fill"),
        }
    }

    #[test]
    fn test_element_style_default() {
        let style = ElementStyle::default();

        assert_eq!(style.fill, Fill::Solid(WHITE));
        assert_eq!(style.border_width, 0.0);
        assert_eq!(style.border_color, BLACK);
        assert_eq!(style.corner_radii, CornerRadii::uniform(0.0));
        assert!(style.shadow.is_none());
    }

    #[test]
    fn test_element_style_new() {
        let style = ElementStyle::new();
        assert_eq!(style, ElementStyle::default());
    }

    #[test]
    fn test_element_style_with_background() {
        let style = ElementStyle::new().with_background(RED);

        assert_eq!(style.fill, Fill::Solid(RED));
        assert_eq!(style.border_width, 0.0); // Other properties unchanged
    }

    #[test]
    fn test_element_style_with_linear_gradient() {
        let style = ElementStyle::new().with_linear_gradient(RED, BLUE, std::f32::consts::PI / 2.0);

        match style.fill {
            Fill::LinearGradient { start, end, angle } => {
                assert_eq!(start, RED);
                assert_eq!(end, BLUE);
                assert_eq!(angle, std::f32::consts::PI / 2.0);
            }
            _ => panic!("Expected linear gradient"),
        }
    }

    #[test]
    fn test_element_style_with_radial_gradient() {
        let style = ElementStyle::new().with_radial_gradient(WHITE, GREEN);

        match style.fill {
            Fill::RadialGradient { center, edge } => {
                assert_eq!(center, WHITE);
                assert_eq!(edge, GREEN);
            }
            _ => panic!("Expected radial gradient"),
        }
    }

    #[test]
    fn test_element_style_with_border() {
        let style = ElementStyle::new().with_border(2.5, RED);

        assert_eq!(style.border_width, 2.5);
        assert_eq!(style.border_color, RED);
    }

    #[test]
    fn test_element_style_with_corner_radius() {
        let style = ElementStyle::new().with_corner_radius(10.0);

        assert_eq!(style.corner_radii, CornerRadii::uniform(10.0));
    }

    #[test]
    fn test_element_style_with_corner_radii() {
        let radii = CornerRadii::new(1.0, 2.0, 3.0, 4.0);
        let style = ElementStyle::new().with_corner_radii(radii);

        assert_eq!(style.corner_radii, radii);
    }

    #[test]
    fn test_element_style_with_shadow() {
        let offset = Vec2::new(3.0, 4.0);
        let style = ElementStyle::new().with_shadow(offset, 2.5, RED);

        assert!(style.shadow.is_some());
        let shadow = style.shadow.unwrap();
        assert_eq!(shadow.offset, offset);
        assert_eq!(shadow.blur, 2.5);
        assert_eq!(shadow.color, RED);
    }

    #[test]
    fn test_element_style_chaining() {
        let style = ElementStyle::new()
            .with_background(BLUE)
            .with_border(1.5, RED)
            .with_corner_radius(8.0)
            .with_shadow(Vec2::new(2.0, 2.0), 4.0, BLACK);

        assert_eq!(style.fill, Fill::Solid(BLUE));
        assert_eq!(style.border_width, 1.5);
        assert_eq!(style.border_color, RED);
        assert_eq!(style.corner_radii, CornerRadii::uniform(8.0));

        assert!(style.shadow.is_some());
        let shadow = style.shadow.unwrap();
        assert_eq!(shadow.offset, Vec2::new(2.0, 2.0));
        assert_eq!(shadow.blur, 4.0);
        assert_eq!(shadow.color, BLACK);
    }

    #[test]
    fn test_fill_clone() {
        let original = Fill::Solid(RED);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_fill_debug() {
        let fill = Fill::Solid(RED);
        let debug_string = format!("{:?}", fill);
        assert!(debug_string.contains("Solid"));
    }

    #[test]
    fn test_corner_radii_copy() {
        let original = CornerRadii::uniform(5.0);
        let copied = original;
        assert_eq!(original, copied);
    }

    #[test]
    fn test_shadow_copy() {
        let original = Shadow {
            offset: Vec2::new(1.0, 2.0),
            blur: 3.0,
            color: BLUE,
        };
        let copied = original;
        assert_eq!(original, copied);
    }
}
