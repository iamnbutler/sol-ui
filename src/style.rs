use glam::Vec2;

use crate::color::{
    Color,
    colors::{BLACK, WHITE},
};

// Re-export FontWeight for public API
pub use parley::FontWeight;

/// Text styling information
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// Font size in logical pixels
    pub size: f32,
    /// Text color
    pub color: Color,
    /// Font family name (e.g., "system-ui", "Helvetica", "monospace")
    /// Must be a static string literal for use with parley's FontStack
    pub font_family: &'static str,
    /// Font weight (e.g., FontWeight::NORMAL, FontWeight::BOLD)
    pub weight: FontWeight,
    /// Line height multiplier (1.0 = same as font size)
    pub line_height: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: 16.0,
            color: WHITE,
            font_family: "system-ui",
            weight: FontWeight::NORMAL,
            line_height: 1.2,
        }
    }
}

impl TextStyle {
    /// Create a new text style with default values (16px, white)
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the text size in points
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set the text color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the font family (must be a static string)
    pub fn with_font_family(mut self, family: &'static str) -> Self {
        self.font_family = family;
        self
    }

    /// Set the font weight
    pub fn with_weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }

    /// Set the line height multiplier
    pub fn line_height(mut self, line_height: f32) -> Self {
        self.line_height = line_height;
        self
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
