use glam::Vec2;

use crate::color::{
    Color, ColorExt,
    colors::{BLACK, WHITE},
};
use crate::geometry::Rect;

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
pub struct FrameStyle {
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

impl Default for FrameStyle {
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

impl FrameStyle {
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
    Frame { rect: Rect, style: FrameStyle },
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
    pub fn add_frame(&mut self, rect: Rect, style: FrameStyle) {
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
