//! Types and utilites that sit between the UI system and rendering pipeline

use crate::{
    color::{Color, ColorExt},
    geometry::{Corners, Edges, Rect},
    style::{ElementStyle, Fill, TextStyle},
};
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

/// A draw command represents a single drawing operation
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Draw a filled rectangle
    Rect { rect: Rect, color: Color },
    /// Draw text
    Text {
        position: Vec2,
        text: String,
        style: TextStyle,
    },
    /// Draw an SDF frame with rounded corners and optional border
    Frame { rect: Rect, style: ElementStyle },
    /// Push a clipping rectangle
    PushClip { rect: Rect },
    /// Pop the current clipping rectangle
    PopClip,
}

/// A list of draw commands to be rendered
#[derive(Clone)]
pub struct DrawList {
    commands: Vec<DrawCommand>,
    clip_stack: Vec<Rect>,
    /// The viewport bounds for culling (None means no culling)
    viewport: Option<Rect>,
    /// Statistics for culling
    culling_stats: CullingStats,
    /// Debug mode for visualizing culled elements
    debug_culling: bool,
}

/// Statistics for viewport culling
#[derive(Clone, Default)]
pub struct CullingStats {
    /// Number of draw calls that were culled (not rendered)
    pub culled_count: usize,
    /// Number of draw calls that were rendered
    pub rendered_count: usize,
}

impl CullingStats {
    /// Get the total number of draw calls attempted
    pub fn total_count(&self) -> usize {
        self.culled_count + self.rendered_count
    }

    /// Get the culling percentage (0.0 to 100.0)
    pub fn culling_percentage(&self) -> f32 {
        let total = self.total_count();
        if total == 0 {
            0.0
        } else {
            (self.culled_count as f32 / total as f32) * 100.0
        }
    }

    /// Reset the statistics
    pub fn reset(&mut self) {
        self.culled_count = 0;
        self.rendered_count = 0;
    }
}

/// A marker for a position in the draw list
#[derive(Debug, Clone, Copy)]
pub struct DrawListPos(usize);

impl DrawListPos {
    /// Get the index position
    pub fn index(&self) -> usize {
        self.0
    }
}

impl DrawList {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            clip_stack: Vec::new(),
            viewport: None,
            culling_stats: CullingStats::default(),
            debug_culling: false,
        }
    }

    /// Create a new DrawList with viewport culling enabled
    pub fn with_viewport(viewport: Rect) -> Self {
        Self {
            commands: Vec::new(),
            clip_stack: Vec::new(),
            viewport: Some(viewport),
            culling_stats: CullingStats::default(),
            debug_culling: false,
        }
    }

    /// Set the viewport for culling
    pub fn set_viewport(&mut self, viewport: Option<Rect>) {
        self.viewport = viewport;
    }

    /// Get the current viewport
    pub fn viewport(&self) -> &Option<Rect> {
        &self.viewport
    }

    /// Enable or disable debug visualization of culled elements
    pub fn set_debug_culling(&mut self, enabled: bool) {
        self.debug_culling = enabled;
    }

    /// Check if debug culling visualization is enabled
    pub fn is_debug_culling(&self) -> bool {
        self.debug_culling
    }

    /// Check if a rectangle is visible within the current viewport and clip bounds
    fn is_visible(&self, rect: &Rect) -> bool {
        // First check against viewport if set
        if let Some(viewport) = &self.viewport {
            if viewport.intersect(rect).is_none() {
                return false;
            }
        }

        // Then check against current clip rect if any
        if let Some(clip) = self.clip_stack.last() {
            clip.intersect(rect).is_some()
        } else {
            true
        }
    }

    /// Get the visibility ratio of a rectangle (0.0 = fully culled, 1.0 = fully visible)
    fn _amount_visible(&self, rect: &Rect) -> f32 {
        let mut visibility = 1.0;

        // Check against viewport
        if let Some(viewport) = &self.viewport {
            visibility *= rect.visibility_ratio_in(viewport);
            if visibility == 0.0 {
                return 0.0;
            }
        }

        // Check against clip stack
        if let Some(clip) = self.clip_stack.last() {
            visibility *= rect.visibility_ratio_in(clip);
        }

        visibility
    }

    /// Add a filled rectangle to the draw list
    pub fn add_rect(&mut self, rect: Rect, color: Color) {
        // Skip if completely transparent
        if color.alpha <= 0.0 {
            return;
        }

        // Skip if not visible (viewport culling)
        if !self.is_visible(&rect) {
            self.culling_stats.culled_count += 1;

            // In debug mode, render culled elements with a special style
            if self.debug_culling {
                let debug_color = Color::rgba(1.0, 0.0, 0.0, 0.2); // Semi-transparent red
                self.commands.push(DrawCommand::Rect {
                    rect,
                    color: debug_color,
                });
            }
            return;
        }

        self.culling_stats.rendered_count += 1;
        self.commands.push(DrawCommand::Rect { rect, color });
    }

    /// Add text to the draw list
    pub fn add_text(&mut self, position: Vec2, text: impl Into<String>, style: TextStyle) {
        let text = text.into();
        if text.is_empty() {
            return;
        }

        // Approximate text bounds for culling (this is a rough estimate)
        // In a real implementation, you'd want to measure the text properly
        let approx_width = text.len() as f32 * style.size * 0.6;
        let approx_height = style.size * 1.2;
        let text_rect = Rect::from_pos_size(position, Vec2::new(approx_width, approx_height));

        // Skip if not visible (viewport culling)
        if !self.is_visible(&text_rect) {
            self.culling_stats.culled_count += 1;

            // In debug mode, render culled text with a special style
            if self.debug_culling {
                let debug_style = TextStyle {
                    color: Color::rgba(1.0, 0.0, 0.0, 0.3), // Semi-transparent red
                    ..style
                };
                self.commands.push(DrawCommand::Text {
                    position,
                    text,
                    style: debug_style,
                });
            }
            return;
        }

        self.culling_stats.rendered_count += 1;
        self.commands.push(DrawCommand::Text {
            position,
            text,
            style,
        });
    }

    /// Push a clipping rectangle
    pub fn push_clip(&mut self, rect: Rect) {
        // Calculate intersection with current clip rect if any
        let clip_rect = if let Some(current) = self.clip_stack.last() {
            match current.intersect(&rect) {
                Some(intersection) => intersection,
                None => {
                    // Empty intersection, use zero-sized rect
                    Rect::new(rect.pos.x, rect.pos.y, 0.0, 0.0)
                }
            }
        } else {
            rect
        };

        self.clip_stack.push(clip_rect);
        self.commands
            .push(DrawCommand::PushClip { rect: clip_rect });
    }

    /// Pop the current clipping rectangle
    pub fn pop_clip(&mut self) {
        if self.clip_stack.pop().is_some() {
            self.commands.push(DrawCommand::PopClip);
        }
    }

    /// Get the current clip rectangle if any
    pub fn current_clip(&self) -> Option<&Rect> {
        self.clip_stack.last()
    }

    /// Clear all commands
    pub fn clear(&mut self) {
        self.commands.clear();
        self.clip_stack.clear();
        self.culling_stats.reset();
    }

    /// Get all commands
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// Get mutable access to commands (use with care)
    pub fn commands_mut(&mut self) -> &mut Vec<DrawCommand> {
        &mut self.commands
    }

    /// Check if the draw list is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Record the current position in the draw list
    pub fn current_pos(&self) -> DrawListPos {
        DrawListPos(self.commands.len())
    }

    /// Get the culling statistics
    pub fn culling_stats(&self) -> &CullingStats {
        &self.culling_stats
    }

    /// Insert a rectangle at a specific position in the draw list
    pub fn insert_rect_at(&mut self, pos: DrawListPos, rect: Rect, color: Color) {
        // Skip if completely transparent
        if color.alpha <= 0.0 {
            return;
        }

        self.commands
            .insert(pos.0, DrawCommand::Rect { rect, color });
    }

    /// Add an SDF frame to the draw list
    pub fn add_frame(&mut self, rect: Rect, style: ElementStyle) {
        // Skip if completely transparent
        let has_visible_fill = match &style.fill {
            Fill::Solid(color) => color.alpha > 0.0,
            Fill::LinearGradient { start, end, .. } => start.alpha > 0.0 || end.alpha > 0.0,
            Fill::RadialGradient { center, edge } => center.alpha > 0.0 || edge.alpha > 0.0,
        };
        let has_visible_border = style.border_width > 0.0 && style.border_color.alpha > 0.0;
        let has_visible_shadow = style.shadow.as_ref().map_or(false, |s| s.color.alpha > 0.0);

        if !has_visible_fill && !has_visible_border && !has_visible_shadow {
            return;
        }

        // Expand rect to account for shadow if present
        let expanded_rect = if let Some(shadow) = &style.shadow {
            let offset = shadow.offset.abs();
            let expansion = offset + Vec2::splat(shadow.blur);
            Rect::from_pos_size(rect.pos - expansion, rect.size + expansion * 2.0)
        } else {
            rect
        };

        // Skip if not visible (viewport culling)
        if !self.is_visible(&expanded_rect) {
            self.culling_stats.culled_count += 1;

            // In debug mode, render culled frames with a special style
            if self.debug_culling {
                let mut debug_style = style.clone();
                // Make the frame semi-transparent red
                debug_style.fill = Fill::Solid(Color::rgba(1.0, 0.0, 0.0, 0.2));
                debug_style.border_color = Color::rgba(1.0, 0.0, 0.0, 0.5);
                debug_style.border_width = debug_style.border_width.max(1.0);
                self.commands.push(DrawCommand::Frame {
                    rect,
                    style: debug_style,
                });
            }
            return;
        }

        self.culling_stats.rendered_count += 1;
        self.commands.push(DrawCommand::Frame { rect, style });
    }
}

impl Default for DrawList {
    fn default() -> Self {
        Self::new()
    }
}
