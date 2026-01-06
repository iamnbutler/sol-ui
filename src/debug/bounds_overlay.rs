//! Element bounds overlay visualization
//!
//! Draws bounding boxes around elements for debugging layout issues.

use crate::{
    color::{Color, ColorExt, colors},
    geometry::{Edges, Rect},
    render::{PaintContext, PaintQuad},
    style::TextStyle,
};
use glam::Vec2;

/// A registered element bounds entry
#[derive(Debug, Clone)]
pub struct BoundsEntry {
    /// Element name/identifier
    pub name: String,
    /// Element bounds
    pub bounds: Rect,
    /// Color for this entry (auto-assigned)
    pub color: Color,
}

/// Bounds overlay that visualizes element boundaries
pub struct BoundsOverlay {
    entries: Vec<BoundsEntry>,
    show_labels: bool,
    show_dimensions: bool,
    color_index: usize,
}

impl BoundsOverlay {
    /// Get a color by index
    fn color_at(index: usize) -> Color {
        match index % 8 {
            0 => Color::rgba(1.0, 0.0, 0.0, 0.5),    // Red
            1 => Color::rgba(0.0, 1.0, 0.0, 0.5),    // Green
            2 => Color::rgba(0.0, 0.0, 1.0, 0.5),    // Blue
            3 => Color::rgba(1.0, 1.0, 0.0, 0.5),    // Yellow
            4 => Color::rgba(1.0, 0.0, 1.0, 0.5),    // Magenta
            5 => Color::rgba(0.0, 1.0, 1.0, 0.5),    // Cyan
            6 => Color::rgba(1.0, 0.5, 0.0, 0.5),    // Orange
            _ => Color::rgba(0.5, 0.0, 1.0, 0.5),    // Purple
        }
    }

    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            show_labels: true,
            show_dimensions: true,
            color_index: 0,
        }
    }

    /// Register element bounds for visualization
    pub fn register_bounds(&mut self, name: &str, bounds: Rect) {
        let color = Self::color_at(self.color_index);
        self.color_index += 1;

        self.entries.push(BoundsEntry {
            name: name.to_string(),
            bounds,
            color,
        });
    }

    /// Clear all registered bounds
    pub fn clear(&mut self) {
        self.entries.clear();
        self.color_index = 0;
    }

    /// Toggle label display
    pub fn toggle_labels(&mut self) {
        self.show_labels = !self.show_labels;
    }

    /// Toggle dimension display
    pub fn toggle_dimensions(&mut self) {
        self.show_dimensions = !self.show_dimensions;
    }

    /// Paint the bounds overlay
    pub fn paint(&self, ctx: &mut PaintContext) {
        for entry in &self.entries {
            self.paint_entry(entry, ctx);
        }
    }

    fn paint_entry(&self, entry: &BoundsEntry, ctx: &mut PaintContext) {
        // Draw border around element
        ctx.paint_quad(PaintQuad {
            bounds: entry.bounds,
            fill: colors::TRANSPARENT,
            corner_radii: crate::geometry::Corners::zero(),
            border_widths: Edges::all(1.0),
            border_color: entry.color,
        });

        // Draw label background and text
        if self.show_labels || self.show_dimensions {
            let label = if self.show_dimensions {
                format!(
                    "{} ({:.0}x{:.0})",
                    entry.name, entry.bounds.size.x, entry.bounds.size.y
                )
            } else {
                entry.name.clone()
            };

            let label_height = 14.0;
            let label_width = label.len() as f32 * 6.5 + 8.0;
            let label_bounds = Rect::from_pos_size(
                entry.bounds.pos - Vec2::new(0.0, label_height),
                Vec2::new(label_width.min(entry.bounds.size.x.max(60.0)), label_height),
            );

            // Background
            ctx.paint_solid_quad(label_bounds, Color::rgba(0.0, 0.0, 0.0, 0.7));

            // Text
            ctx.paint_text(crate::render::PaintText {
                position: label_bounds.pos + Vec2::new(4.0, 1.0),
                text: label,
                style: TextStyle {
                    size: 10.0,
                    color: entry.color,
                    line_height: 1.2,
                },
            });
        }

        // Draw corner markers
        self.paint_corner_markers(entry, ctx);
    }

    fn paint_corner_markers(&self, entry: &BoundsEntry, ctx: &mut PaintContext) {
        let marker_size = 4.0;
        let color = entry.color;

        // Top-left
        ctx.paint_solid_quad(
            Rect::from_pos_size(entry.bounds.pos, Vec2::splat(marker_size)),
            color,
        );

        // Top-right
        ctx.paint_solid_quad(
            Rect::from_pos_size(
                entry.bounds.pos + Vec2::new(entry.bounds.size.x - marker_size, 0.0),
                Vec2::splat(marker_size),
            ),
            color,
        );

        // Bottom-left
        ctx.paint_solid_quad(
            Rect::from_pos_size(
                entry.bounds.pos + Vec2::new(0.0, entry.bounds.size.y - marker_size),
                Vec2::splat(marker_size),
            ),
            color,
        );

        // Bottom-right
        ctx.paint_solid_quad(
            Rect::from_pos_size(
                entry.bounds.pos + entry.bounds.size - Vec2::splat(marker_size),
                Vec2::splat(marker_size),
            ),
            color,
        );
    }
}

impl Default for BoundsOverlay {
    fn default() -> Self {
        Self::new()
    }
}
