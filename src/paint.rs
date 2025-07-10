//! Paint types for the two-phase rendering system

use crate::color::Color;
use crate::draw::TextStyle;
use crate::geometry::{Corners, Edges, Rect};
use glam::Vec2;

/// A quad to be rendered
#[derive(Clone, Debug)]
pub struct PaintQuad {
    /// The bounds of the quad within the window
    pub bounds: Rect,
    /// The radii of the quad's corners
    pub corner_radii: Corners,
    /// The background color of the quad
    pub fill: Color,
    /// The widths of the quad's borders
    pub border_widths: Edges,
    /// The color of the quad's borders
    pub border_color: Color,
}

impl PaintQuad {
    /// Create a simple filled quad with no borders or corner radius
    pub fn filled(bounds: Rect, color: Color) -> Self {
        Self {
            bounds,
            fill: color,
            corner_radii: Corners::zero(),
            border_widths: Edges::zero(),
            border_color: crate::color::colors::TRANSPARENT,
        }
    }
}

/// Text to be rendered
#[derive(Clone, Debug)]
pub struct PaintText {
    /// Position to render the text
    pub position: Vec2,
    /// The text content
    pub text: String,
    /// Text styling
    pub style: TextStyle,
}

/// A shadow to be rendered
#[derive(Clone, Debug)]
pub struct PaintShadow {
    /// The bounds of the shadow
    pub bounds: Rect,
    /// Corner radii for rounded shadows
    pub corner_radii: Corners,
    /// Shadow color
    pub color: Color,
    /// Blur radius
    pub blur_radius: f32,
    /// Offset from the original element
    pub offset: Vec2,
}

/// An image to be rendered
#[derive(Clone, Debug)]
pub struct PaintImage {
    /// The bounds of the image
    pub bounds: Rect,
    /// Path or identifier for the image
    pub source: String,
    /// Corner radii for rounded images
    pub corner_radii: Corners,
}
