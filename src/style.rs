use glam::Vec2;

use crate::color::{
    Color,
    colors::{BLACK, WHITE},
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
    use crate::color::{Color, ColorExt, colors::*};

    #[test]
    fn test_text_style_default() {
        let style = TextStyle::default();
        assert_eq!(style.size, 16.0);
        assert_eq!(style.color, WHITE);
    }

    #[test]
    fn test_text_style_creation() {
        let style = TextStyle {
            size: 24.0,
            color: Color::rgb(0.5, 0.5, 0.5),
        };
        assert_eq!(style.size, 24.0);
        assert_eq!(style.color.red, 0.5);
        assert_eq!(style.color.green, 0.5);
        assert_eq!(style.color.blue, 0.5);
    }

    #[test]
    fn test_corner_radii_uniform() {
        let radii = CornerRadii::uniform(10.0);
        assert_eq!(radii.top_left, 10.0);
        assert_eq!(radii.top_right, 10.0);
        assert_eq!(radii.bottom_right, 10.0);
        assert_eq!(radii.bottom_left, 10.0);
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
    fn test_shadow_creation() {
        let shadow = Shadow {
            offset: Vec2::new(5.0, 5.0),
            blur: 10.0,
            color: BLACK,
        };
        assert_eq!(shadow.offset, Vec2::new(5.0, 5.0));
        assert_eq!(shadow.blur, 10.0);
        assert_eq!(shadow.color, BLACK);
    }

    #[test]
    fn test_fill_solid() {
        let fill = Fill::Solid(RED);
        if let Fill::Solid(color) = fill {
            assert_eq!(color, RED);
        } else {
            panic!("Expected Fill::Solid");
        }
    }

    #[test]
    fn test_fill_linear_gradient() {
        let fill = Fill::LinearGradient {
            start: RED,
            end: BLUE,
            angle: std::f32::consts::PI / 2.0,
        };
        
        if let Fill::LinearGradient { start, end, angle } = fill {
            assert_eq!(start, RED);
            assert_eq!(end, BLUE);
            assert_eq!(angle, std::f32::consts::PI / 2.0);
        } else {
            panic!("Expected Fill::LinearGradient");
        }
    }

    #[test]
    fn test_fill_radial_gradient() {
        let fill = Fill::RadialGradient {
            center: WHITE,
            edge: BLACK,
        };
        
        if let Fill::RadialGradient { center, edge } = fill {
            assert_eq!(center, WHITE);
            assert_eq!(edge, BLACK);
        } else {
            panic!("Expected Fill::RadialGradient");
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
        let default_style = ElementStyle::default();
        
        assert_eq!(style.fill, default_style.fill);
        assert_eq!(style.border_width, default_style.border_width);
        assert_eq!(style.border_color, default_style.border_color);
        assert_eq!(style.corner_radii, default_style.corner_radii);
        assert_eq!(style.shadow, default_style.shadow);
    }

    #[test]
    fn test_element_style_with_background() {
        let style = ElementStyle::new().with_background(RED);
        assert_eq!(style.fill, Fill::Solid(RED));
    }

    #[test]
    fn test_element_style_with_linear_gradient() {
        let style = ElementStyle::new()
            .with_linear_gradient(RED, BLUE, std::f32::consts::PI);
        
        if let Fill::LinearGradient { start, end, angle } = style.fill {
            assert_eq!(start, RED);
            assert_eq!(end, BLUE);
            assert_eq!(angle, std::f32::consts::PI);
        } else {
            panic!("Expected Fill::LinearGradient");
        }
    }

    #[test]
    fn test_element_style_with_radial_gradient() {
        let style = ElementStyle::new()
            .with_radial_gradient(WHITE, BLACK);
        
        if let Fill::RadialGradient { center, edge } = style.fill {
            assert_eq!(center, WHITE);
            assert_eq!(edge, BLACK);
        } else {
            panic!("Expected Fill::RadialGradient");
        }
    }

    #[test]
    fn test_element_style_with_border() {
        let style = ElementStyle::new().with_border(2.0, BLUE);
        assert_eq!(style.border_width, 2.0);
        assert_eq!(style.border_color, BLUE);
    }

    #[test]
    fn test_element_style_with_corner_radius() {
        let style = ElementStyle::new().with_corner_radius(15.0);
        assert_eq!(style.corner_radii, CornerRadii::uniform(15.0));
    }

    #[test]
    fn test_element_style_with_corner_radii() {
        let radii = CornerRadii::new(5.0, 10.0, 15.0, 20.0);
        let style = ElementStyle::new().with_corner_radii(radii);
        assert_eq!(style.corner_radii, radii);
    }

    #[test]
    fn test_element_style_with_shadow() {
        let offset = Vec2::new(3.0, 3.0);
        let blur = 5.0;
        let color = GRAY_500;
        
        let style = ElementStyle::new().with_shadow(offset, blur, color);
        
        assert!(style.shadow.is_some());
        let shadow = style.shadow.unwrap();
        assert_eq!(shadow.offset, offset);
        assert_eq!(shadow.blur, blur);
        assert_eq!(shadow.color, color);
    }

    #[test]
    fn test_element_style_chaining() {
        let style = ElementStyle::new()
            .with_background(BLUE_500)
            .with_border(1.0, BLUE_600)
            .with_corner_radius(8.0)
            .with_shadow(Vec2::new(2.0, 2.0), 4.0, GRAY_800);
        
        assert_eq!(style.fill, Fill::Solid(BLUE_500));
        assert_eq!(style.border_width, 1.0);
        assert_eq!(style.border_color, BLUE_600);
        assert_eq!(style.corner_radii, CornerRadii::uniform(8.0));
        assert!(style.shadow.is_some());
        
        let shadow = style.shadow.unwrap();
        assert_eq!(shadow.offset, Vec2::new(2.0, 2.0));
        assert_eq!(shadow.blur, 4.0);
        assert_eq!(shadow.color, GRAY_800);
    }

    #[test]
    fn test_fill_pattern_matching() {
        let fills = vec![
            Fill::Solid(RED),
            Fill::LinearGradient { start: RED, end: BLUE, angle: 0.0 },
            Fill::RadialGradient { center: WHITE, edge: BLACK },
        ];
        
        for fill in fills {
            match fill {
                Fill::Solid(_) => {
                    // Test successful pattern matching for solid
                }
                Fill::LinearGradient { .. } => {
                    // Test successful pattern matching for linear gradient
                }
                Fill::RadialGradient { .. } => {
                    // Test successful pattern matching for radial gradient
                }
            }
        }
    }

    #[test]
    fn test_corner_radii_equality() {
        let radii1 = CornerRadii::new(1.0, 2.0, 3.0, 4.0);
        let radii2 = CornerRadii::new(1.0, 2.0, 3.0, 4.0);
        let radii3 = CornerRadii::new(4.0, 3.0, 2.0, 1.0);
        
        assert_eq!(radii1, radii2);
        assert_ne!(radii1, radii3);
    }

    #[test]
    fn test_shadow_equality() {
        let shadow1 = Shadow {
            offset: Vec2::new(1.0, 1.0),
            blur: 5.0,
            color: BLACK,
        };
        let shadow2 = Shadow {
            offset: Vec2::new(1.0, 1.0),
            blur: 5.0,
            color: BLACK,
        };
        let shadow3 = Shadow {
            offset: Vec2::new(2.0, 2.0),
            blur: 5.0,
            color: BLACK,
        };
        
        assert_eq!(shadow1, shadow2);
        assert_ne!(shadow1, shadow3);
    }

    #[test]
    fn test_element_style_partial_eq() {
        let style1 = ElementStyle::new()
            .with_background(RED)
            .with_border(1.0, BLACK);
        
        let style2 = ElementStyle::new()
            .with_background(RED)
            .with_border(1.0, BLACK);
        
        let style3 = ElementStyle::new()
            .with_background(BLUE)
            .with_border(1.0, BLACK);
        
        assert_eq!(style1, style2);
        assert_ne!(style1, style3);
    }
}
