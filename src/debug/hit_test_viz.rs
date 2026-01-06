//! Hit test visualization
//!
//! Visualizes hit test regions and z-order for debugging interaction issues.

use crate::{
    color::{Color, ColorExt, colors},
    geometry::{Edges, Rect},
    render::{PaintContext, PaintQuad, PaintText},
    style::TextStyle,
};
use glam::Vec2;

/// A hit test entry for visualization
#[derive(Debug, Clone)]
pub struct HitTestEntry {
    pub element_id: u64,
    pub bounds: Rect,
    pub z_index: i32,
}

/// Hit test visualization overlay
pub struct HitTestVisualization {
    entries: Vec<HitTestEntry>,
    hovered_entry: Option<usize>,
    show_z_index: bool,
    show_element_id: bool,
}

impl HitTestVisualization {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            hovered_entry: None,
            show_z_index: true,
            show_element_id: true,
        }
    }

    /// Register a hit test entry
    pub fn register_entry(&mut self, element_id: u64, bounds: Rect, z_index: i32) {
        self.entries.push(HitTestEntry {
            element_id,
            bounds,
            z_index,
        });
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.hovered_entry = None;
    }

    /// Set the hovered entry based on mouse position
    pub fn update_hover(&mut self, position: Vec2) {
        // Find the topmost entry under the mouse
        self.hovered_entry = None;
        let mut highest_z = i32::MIN;

        for (i, entry) in self.entries.iter().enumerate() {
            if entry.bounds.contains(crate::geometry::Point::new(position.x, position.y)) {
                if entry.z_index > highest_z {
                    highest_z = entry.z_index;
                    self.hovered_entry = Some(i);
                }
            }
        }
    }

    /// Toggle z-index display
    pub fn toggle_z_index(&mut self) {
        self.show_z_index = !self.show_z_index;
    }

    /// Toggle element ID display
    pub fn toggle_element_id(&mut self) {
        self.show_element_id = !self.show_element_id;
    }

    /// Get the number of hit test entries
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Paint the hit test visualization
    pub fn paint(&self, ctx: &mut PaintContext) {
        // Sort entries by z-index (lowest first so higher ones render on top)
        let mut sorted_entries: Vec<(usize, &HitTestEntry)> =
            self.entries.iter().enumerate().collect();
        sorted_entries.sort_by_key(|(_, e)| e.z_index);

        for (index, entry) in sorted_entries {
            let is_hovered = self.hovered_entry == Some(index);
            self.paint_entry(entry, is_hovered, ctx);
        }
    }

    fn paint_entry(&self, entry: &HitTestEntry, is_hovered: bool, ctx: &mut PaintContext) {
        // Calculate color based on z-index (different hues for different z-levels)
        let hue = (entry.z_index as f32 * 0.3).rem_euclid(1.0);
        let color = hsv_to_rgb(hue, 0.7, 0.9);
        let border_color = color.with_alpha(if is_hovered { 0.9 } else { 0.5 });
        let fill_color = color.with_alpha(if is_hovered { 0.3 } else { 0.1 });

        // Draw filled region
        ctx.paint_solid_quad(entry.bounds, fill_color);

        // Draw border
        ctx.paint_quad(PaintQuad {
            bounds: entry.bounds,
            fill: colors::TRANSPARENT,
            corner_radii: crate::geometry::Corners::zero(),
            border_widths: Edges::all(if is_hovered { 2.0 } else { 1.0 }),
            border_color,
        });

        // Draw label
        if self.show_z_index || self.show_element_id {
            let label = match (self.show_z_index, self.show_element_id) {
                (true, true) => format!("z:{} id:{}", entry.z_index, entry.element_id),
                (true, false) => format!("z:{}", entry.z_index),
                (false, true) => format!("id:{}", entry.element_id),
                (false, false) => String::new(),
            };

            if !label.is_empty() {
                let label_pos = entry.bounds.pos + Vec2::new(2.0, 2.0);
                let label_bg = Rect::from_pos_size(
                    label_pos - Vec2::new(1.0, 1.0),
                    Vec2::new(label.len() as f32 * 6.0 + 4.0, 12.0),
                );

                ctx.paint_solid_quad(label_bg, Color::rgba(0.0, 0.0, 0.0, 0.7));
                ctx.paint_text(PaintText {
                    position: label_pos,
                    text: label,
                    style: TextStyle {
                        size: 9.0,
                        color: border_color,
                        line_height: 1.2,
                    },
                });
            }
        }
    }
}

impl Default for HitTestVisualization {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert HSV to RGB color
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let c = v * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match (h * 6.0) as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color::rgba(r + m, g + m, b + m, 1.0)
}
