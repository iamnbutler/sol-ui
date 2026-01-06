use glam::Vec2;

use crate::color::{
    Color,
    colors::{BLACK, WHITE},
};
use crate::geometry::Corners;

/// Text styling information
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub size: f32,
    pub color: Color,
    /// Line height multiplier (1.0 = tight, 1.2 = normal, 1.5 = loose)
    pub line_height: f32,
    // TODO: Add font family, weight, etc.
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: 16.0,
            color: WHITE,
            line_height: 1.2,
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
    pub corner_radii: Corners,
    /// Optional shadow
    pub shadow: Option<Shadow>,
}

impl Default for ElementStyle {
    fn default() -> Self {
        Self {
            fill: Fill::Solid(WHITE),
            border_width: 0.0,
            border_color: BLACK,
            corner_radii: Corners::all(0.0),
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
        self.corner_radii = Corners::all(radius);
        self
    }

    /// Set individual corner radii
    pub fn with_corner_radii(mut self, radii: Corners) -> Self {
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
