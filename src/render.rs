//! Types and utilites that sit between the UI system and rendering pipeline

use std::{cell::RefCell, rc::Rc};

use crate::{
    color::{Color, ColorExt},
    geometry::{Corners, Edges, Rect},
    interaction::{ElementId, HitTestBuilder},
    layout_engine::TaffyLayoutEngine,
    style::{ElementStyle, Fill, TextStyle},
    text_system::TextSystem,
};
use glam::Vec2;
use taffy::NodeId;

/// Context for the paint phase
pub struct PaintContext<'a> {
    pub(crate) draw_list: &'a mut DrawList,
    pub(crate) text_system: &'a mut TextSystem,
    pub(crate) layout_engine: &'a TaffyLayoutEngine,
    pub(crate) scale_factor: f32,
    pub(crate) parent_offset: Vec2,
    pub(crate) hit_test_builder: Option<Rc<RefCell<HitTestBuilder>>>,
}

impl<'a> PaintContext<'a> {
    /// Paint a quad with all its properties
    pub fn paint_quad(&mut self, quad: PaintQuad) {
        // For now, just handle the fill
        // TODO: Handle borders, corner radii, etc.
        self.draw_list.add_rect(quad.bounds, quad.fill);

        // Paint borders if present
        if quad.border_widths != Edges::zero()
            && quad.border_color != crate::color::colors::TRANSPARENT
        {
            // Top edge
            if quad.border_widths.top > 0.0 {
                self.draw_list.add_rect(
                    Rect::from_pos_size(
                        quad.bounds.pos,
                        Vec2::new(quad.bounds.size.x, quad.border_widths.top),
                    ),
                    quad.border_color,
                );
            }

            // Bottom edge
            if quad.border_widths.bottom > 0.0 {
                self.draw_list.add_rect(
                    Rect::from_pos_size(
                        quad.bounds.pos
                            + Vec2::new(0.0, quad.bounds.size.y - quad.border_widths.bottom),
                        Vec2::new(quad.bounds.size.x, quad.border_widths.bottom),
                    ),
                    quad.border_color,
                );
            }

            // Left edge
            if quad.border_widths.left > 0.0 {
                self.draw_list.add_rect(
                    Rect::from_pos_size(
                        quad.bounds.pos,
                        Vec2::new(quad.border_widths.left, quad.bounds.size.y),
                    ),
                    quad.border_color,
                );
            }

            // Right edge
            if quad.border_widths.right > 0.0 {
                self.draw_list.add_rect(
                    Rect::from_pos_size(
                        quad.bounds.pos
                            + Vec2::new(quad.bounds.size.x - quad.border_widths.right, 0.0),
                        Vec2::new(quad.border_widths.right, quad.bounds.size.y),
                    ),
                    quad.border_color,
                );
            }
        }
    }

    /// Paint text
    pub fn paint_text(&mut self, text: PaintText) {
        self.draw_list
            .add_text(text.position, &text.text, text.style);
    }

    /// Paint a shadow
    pub fn paint_shadow(&mut self, _shadow: PaintShadow) {
        // TODO: Add shadow support to draw list
        // For now this is a no-op
    }

    /// Helper to create a simple filled quad
    pub fn paint_solid_quad(&mut self, bounds: Rect, color: Color) {
        self.paint_quad(PaintQuad::filled(bounds, color));
    }

    /// Check if a rect is visible (for culling)
    pub fn is_visible(&self, rect: &Rect) -> bool {
        if let Some(viewport) = self.draw_list.viewport() {
            viewport.intersect(rect).is_some()
        } else {
            true
        }
    }

    /// Get the computed bounds for a node
    pub fn get_bounds(&self, node_id: NodeId) -> Rect {
        let local_bounds = self.layout_engine.layout_bounds(node_id);
        Rect::from_pos_size(self.parent_offset + local_bounds.pos, local_bounds.size)
    }

    /// Create a child paint context with updated offset
    pub fn child_context(&mut self, offset: Vec2) -> PaintContext {
        PaintContext {
            draw_list: self.draw_list,
            text_system: self.text_system,
            layout_engine: self.layout_engine,
            scale_factor: self.scale_factor,
            parent_offset: self.parent_offset + offset,
            hit_test_builder: self.hit_test_builder.clone(),
        }
    }

    /// Register an element for hit testing
    pub fn register_hit_test(&mut self, element_id: ElementId, bounds: Rect, z_index: i32) {
        if let Some(builder) = &self.hit_test_builder {
            // bounds are already in screen coordinates (absolute position)
            builder.borrow_mut().add_entry(element_id, bounds, z_index);
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::colors::{WHITE, BLACK, RED, BLUE, TRANSPARENT};
    use crate::style::{Fill, Shadow, ElementStyle};

    // === PaintQuad Tests ===

    #[test]
    fn test_paint_quad_filled() {
        let bounds = Rect::new(10.0, 20.0, 100.0, 50.0);
        let color = RED;
        
        let quad = PaintQuad::filled(bounds, color);
        
        assert_eq!(quad.bounds, bounds);
        assert_eq!(quad.fill, color);
        assert_eq!(quad.corner_radii, Corners::zero());
        assert_eq!(quad.border_widths, Edges::zero());
        assert_eq!(quad.border_color, TRANSPARENT);
    }

    #[test]
    fn test_paint_quad_creation() {
        let bounds = Rect::new(0.0, 0.0, 200.0, 100.0);
        let corner_radii = Corners::uniform(10.0);
        let border_widths = Edges::uniform(2.0);
        
        let quad = PaintQuad {
            bounds,
            corner_radii,
            fill: BLUE,
            border_widths,
            border_color: BLACK,
        };
        
        assert_eq!(quad.bounds, bounds);
        assert_eq!(quad.corner_radii, corner_radii);
        assert_eq!(quad.fill, BLUE);
        assert_eq!(quad.border_widths, border_widths);
        assert_eq!(quad.border_color, BLACK);
    }

    #[test]
    fn test_paint_quad_clone() {
        let original = PaintQuad::filled(Rect::new(5.0, 5.0, 50.0, 50.0), WHITE);
        let cloned = original.clone();
        
        assert_eq!(original.bounds, cloned.bounds);
        assert_eq!(original.fill, cloned.fill);
        assert_eq!(original.corner_radii, cloned.corner_radii);
    }

    // === PaintText Tests ===

    #[test]
    fn test_paint_text_creation() {
        let position = Vec2::new(100.0, 200.0);
        let text = "Hello World".to_string();
        let style = TextStyle::default();
        
        let paint_text = PaintText {
            position,
            text: text.clone(),
            style: style.clone(),
        };
        
        assert_eq!(paint_text.position, position);
        assert_eq!(paint_text.text, text);
        assert_eq!(paint_text.style, style);
    }

    #[test]
    fn test_paint_text_with_styled_text() {
        let position = Vec2::new(50.0, 100.0);
        let text = "Styled Text".to_string();
        let style = TextStyle {
            size: 24.0,
            color: RED,
        };
        
        let paint_text = PaintText {
            position,
            text: text.clone(),
            style: style.clone(),
        };
        
        assert_eq!(paint_text.text, text);
        assert_eq!(paint_text.style.size, 24.0);
        assert_eq!(paint_text.style.color, RED);
    }

    // === PaintShadow Tests ===

    #[test]
    fn test_paint_shadow_creation() {
        let bounds = Rect::new(10.0, 10.0, 100.0, 50.0);
        let corner_radii = Corners::uniform(5.0);
        let color = Color::rgba(0.0, 0.0, 0.0, 0.5);
        let blur_radius = 10.0;
        let offset = Vec2::new(2.0, 2.0);
        
        let shadow = PaintShadow {
            bounds,
            corner_radii,
            color,
            blur_radius,
            offset,
        };
        
        assert_eq!(shadow.bounds, bounds);
        assert_eq!(shadow.corner_radii, corner_radii);
        assert_eq!(shadow.color, color);
        assert_eq!(shadow.blur_radius, blur_radius);
        assert_eq!(shadow.offset, offset);
    }

    // === PaintImage Tests ===

    #[test]
    fn test_paint_image_creation() {
        let bounds = Rect::new(0.0, 0.0, 200.0, 150.0);
        let source = "path/to/image.png".to_string();
        let corner_radii = Corners::uniform(8.0);
        
        let image = PaintImage {
            bounds,
            source: source.clone(),
            corner_radii,
        };
        
        assert_eq!(image.bounds, bounds);
        assert_eq!(image.source, source);
        assert_eq!(image.corner_radii, corner_radii);
    }

    // === DrawCommand Tests ===

    #[test]
    fn test_draw_command_rect() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        let color = BLUE;
        
        let command = DrawCommand::Rect { rect, color };
        
        match command {
            DrawCommand::Rect { rect: r, color: c } => {
                assert_eq!(r, rect);
                assert_eq!(c, color);
            }
            _ => panic!("Expected Rect command"),
        }
    }

    #[test]
    fn test_draw_command_text() {
        let position = Vec2::new(50.0, 100.0);
        let text = "Test Text".to_string();
        let style = TextStyle::default();
        
        let command = DrawCommand::Text {
            position,
            text: text.clone(),
            style: style.clone(),
        };
        
        match command {
            DrawCommand::Text { position: p, text: t, style: s } => {
                assert_eq!(p, position);
                assert_eq!(t, text);
                assert_eq!(s, style);
            }
            _ => panic!("Expected Text command"),
        }
    }

    #[test]
    fn test_draw_command_frame() {
        let rect = Rect::new(5.0, 5.0, 90.0, 40.0);
        let style = ElementStyle::default();
        
        let command = DrawCommand::Frame { rect, style: style.clone() };
        
        match command {
            DrawCommand::Frame { rect: r, style: s } => {
                assert_eq!(r, rect);
                assert_eq!(s, style);
            }
            _ => panic!("Expected Frame command"),
        }
    }

    #[test]
    fn test_draw_command_push_pop_clip() {
        let clip_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        
        let push_command = DrawCommand::PushClip { rect: clip_rect };
        let pop_command = DrawCommand::PopClip;
        
        match push_command {
            DrawCommand::PushClip { rect } => assert_eq!(rect, clip_rect),
            _ => panic!("Expected PushClip command"),
        }
        
        match pop_command {
            DrawCommand::PopClip => {},
            _ => panic!("Expected PopClip command"),
        }
    }

    // === CullingStats Tests ===

    #[test]
    fn test_culling_stats_default() {
        let stats = CullingStats::default();
        
        assert_eq!(stats.culled_count, 0);
        assert_eq!(stats.rendered_count, 0);
        assert_eq!(stats.total_count(), 0);
        assert_eq!(stats.culling_percentage(), 0.0);
    }

    #[test]
    fn test_culling_stats_calculations() {
        let mut stats = CullingStats {
            culled_count: 30,
            rendered_count: 70,
        };
        
        assert_eq!(stats.total_count(), 100);
        assert_eq!(stats.culling_percentage(), 30.0);
        
        stats.reset();
        assert_eq!(stats.culled_count, 0);
        assert_eq!(stats.rendered_count, 0);
        assert_eq!(stats.total_count(), 0);
    }

    #[test]
    fn test_culling_stats_edge_cases() {
        // Test 100% culling
        let stats = CullingStats {
            culled_count: 50,
            rendered_count: 0,
        };
        assert_eq!(stats.culling_percentage(), 100.0);
        
        // Test 0% culling
        let stats = CullingStats {
            culled_count: 0,
            rendered_count: 25,
        };
        assert_eq!(stats.culling_percentage(), 0.0);
        
        // Test partial culling
        let stats = CullingStats {
            culled_count: 25,
            rendered_count: 75,
        };
        assert_eq!(stats.culling_percentage(), 25.0);
    }

    // === DrawListPos Tests ===

    #[test]
    fn test_draw_list_pos() {
        let pos = DrawListPos(42);
        assert_eq!(pos.index(), 42);
        
        let copied_pos = pos;
        assert_eq!(copied_pos.index(), 42);
    }

    // === DrawList Tests ===

    #[test]
    fn test_draw_list_new() {
        let draw_list = DrawList::new();
        
        assert!(draw_list.is_empty());
        assert_eq!(draw_list.commands().len(), 0);
        assert_eq!(draw_list.viewport(), &None);
        assert_eq!(draw_list.culling_stats().total_count(), 0);
        assert!(!draw_list.is_debug_culling());
    }

    #[test]
    fn test_draw_list_with_viewport() {
        let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
        let draw_list = DrawList::with_viewport(viewport);
        
        assert!(draw_list.is_empty());
        assert_eq!(draw_list.viewport(), &Some(viewport));
    }

    #[test]
    fn test_draw_list_default() {
        let draw_list = DrawList::default();
        assert!(draw_list.is_empty());
        assert_eq!(draw_list.viewport(), &None);
    }

    #[test]
    fn test_draw_list_viewport_management() {
        let mut draw_list = DrawList::new();
        let viewport = Rect::new(0.0, 0.0, 1024.0, 768.0);
        
        draw_list.set_viewport(Some(viewport));
        assert_eq!(draw_list.viewport(), &Some(viewport));
        
        draw_list.set_viewport(None);
        assert_eq!(draw_list.viewport(), &None);
    }

    #[test]
    fn test_draw_list_debug_culling() {
        let mut draw_list = DrawList::new();
        
        assert!(!draw_list.is_debug_culling());
        
        draw_list.set_debug_culling(true);
        assert!(draw_list.is_debug_culling());
        
        draw_list.set_debug_culling(false);
        assert!(!draw_list.is_debug_culling());
    }

    #[test]
    fn test_draw_list_add_rect() {
        let mut draw_list = DrawList::new();
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        let color = RED;
        
        draw_list.add_rect(rect, color);
        
        assert!(!draw_list.is_empty());
        assert_eq!(draw_list.commands().len(), 1);
        assert_eq!(draw_list.culling_stats().rendered_count, 1);
        
        match &draw_list.commands()[0] {
            DrawCommand::Rect { rect: r, color: c } => {
                assert_eq!(*r, rect);
                assert_eq!(*c, color);
            }
            _ => panic!("Expected Rect command"),
        }
    }

    #[test]
    fn test_draw_list_add_transparent_rect() {
        let mut draw_list = DrawList::new();
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        let transparent_color = Color::rgba(1.0, 0.0, 0.0, 0.0);
        
        draw_list.add_rect(rect, transparent_color);
        
        // Should not add transparent rectangles
        assert!(draw_list.is_empty());
        assert_eq!(draw_list.culling_stats().rendered_count, 0);
    }

    #[test]
    fn test_draw_list_add_text() {
        let mut draw_list = DrawList::new();
        let position = Vec2::new(100.0, 200.0);
        let text = "Hello World";
        let style = TextStyle::default();
        
        draw_list.add_text(position, text, style.clone());
        
        assert!(!draw_list.is_empty());
        assert_eq!(draw_list.commands().len(), 1);
        assert_eq!(draw_list.culling_stats().rendered_count, 1);
        
        match &draw_list.commands()[0] {
            DrawCommand::Text { position: p, text: t, style: s } => {
                assert_eq!(*p, position);
                assert_eq!(t, text);
                assert_eq!(*s, style);
            }
            _ => panic!("Expected Text command"),
        }
    }

    #[test]
    fn test_draw_list_add_empty_text() {
        let mut draw_list = DrawList::new();
        let position = Vec2::new(100.0, 200.0);
        let empty_text = "";
        let style = TextStyle::default();
        
        draw_list.add_text(position, empty_text, style);
        
        // Should not add empty text
        assert!(draw_list.is_empty());
        assert_eq!(draw_list.culling_stats().rendered_count, 0);
    }

    #[test]
    fn test_draw_list_clip_management() {
        let mut draw_list = DrawList::new();
        let clip_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        
        assert_eq!(draw_list.current_clip(), None);
        
        draw_list.push_clip(clip_rect);
        assert_eq!(draw_list.current_clip(), Some(&clip_rect));
        assert_eq!(draw_list.commands().len(), 1);
        
        draw_list.pop_clip();
        assert_eq!(draw_list.current_clip(), None);
        assert_eq!(draw_list.commands().len(), 2);
        
        // Test commands
        match &draw_list.commands()[0] {
            DrawCommand::PushClip { rect } => assert_eq!(*rect, clip_rect),
            _ => panic!("Expected PushClip command"),
        }
        
        match &draw_list.commands()[1] {
            DrawCommand::PopClip => {},
            _ => panic!("Expected PopClip command"),
        }
    }

    #[test]
    fn test_draw_list_nested_clips() {
        let mut draw_list = DrawList::new();
        let outer_clip = Rect::new(0.0, 0.0, 200.0, 200.0);
        let inner_clip = Rect::new(50.0, 50.0, 100.0, 100.0);
        let expected_intersection = Rect::new(50.0, 50.0, 100.0, 100.0);
        
        draw_list.push_clip(outer_clip);
        draw_list.push_clip(inner_clip);
        
        // The current clip should be the intersection
        assert_eq!(draw_list.current_clip(), Some(&expected_intersection));
        
        draw_list.pop_clip();
        assert_eq!(draw_list.current_clip(), Some(&outer_clip));
        
        draw_list.pop_clip();
        assert_eq!(draw_list.current_clip(), None);
    }

    #[test]
    fn test_draw_list_non_intersecting_clips() {
        let mut draw_list = DrawList::new();
        let first_clip = Rect::new(0.0, 0.0, 100.0, 100.0);
        let non_intersecting_clip = Rect::new(200.0, 200.0, 100.0, 100.0);
        
        draw_list.push_clip(first_clip);
        draw_list.push_clip(non_intersecting_clip);
        
        // Should result in zero-sized clip
        let current_clip = draw_list.current_clip().unwrap();
        assert_eq!(current_clip.size, Vec2::ZERO);
    }

    #[test]
    fn test_draw_list_pop_clip_empty_stack() {
        let mut draw_list = DrawList::new();
        
        // Popping from empty stack should do nothing
        draw_list.pop_clip();
        assert!(draw_list.is_empty());
        assert_eq!(draw_list.current_clip(), None);
    }

    #[test]
    fn test_draw_list_clear() {
        let mut draw_list = DrawList::new();
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        
        draw_list.add_rect(rect, RED);
        draw_list.push_clip(rect);
        
        assert!(!draw_list.is_empty());
        assert!(draw_list.current_clip().is_some());
        assert!(draw_list.culling_stats().total_count() > 0);
        
        draw_list.clear();
        
        assert!(draw_list.is_empty());
        assert_eq!(draw_list.current_clip(), None);
        assert_eq!(draw_list.culling_stats().total_count(), 0);
    }

    #[test]
    fn test_draw_list_current_pos() {
        let mut draw_list = DrawList::new();
        
        let pos1 = draw_list.current_pos();
        assert_eq!(pos1.index(), 0);
        
        draw_list.add_rect(Rect::new(0.0, 0.0, 10.0, 10.0), RED);
        let pos2 = draw_list.current_pos();
        assert_eq!(pos2.index(), 1);
        
        draw_list.add_text(Vec2::ZERO, "test", TextStyle::default());
        let pos3 = draw_list.current_pos();
        assert_eq!(pos3.index(), 2);
    }

    #[test]
    fn test_draw_list_insert_rect_at() {
        let mut draw_list = DrawList::new();
        
        // Add some commands first
        draw_list.add_rect(Rect::new(0.0, 0.0, 10.0, 10.0), RED);
        draw_list.add_rect(Rect::new(20.0, 20.0, 10.0, 10.0), BLUE);
        
        let pos = DrawListPos(1); // Insert at position 1
        let insert_rect = Rect::new(10.0, 10.0, 5.0, 5.0);
        
        draw_list.insert_rect_at(pos, insert_rect, WHITE);
        
        assert_eq!(draw_list.commands().len(), 3);
        
        // Check that the rectangle was inserted at the correct position
        match &draw_list.commands()[1] {
            DrawCommand::Rect { rect, color } => {
                assert_eq!(*rect, insert_rect);
                assert_eq!(*color, WHITE);
            }
            _ => panic!("Expected Rect command at position 1"),
        }
    }

    #[test]
    fn test_draw_list_insert_transparent_rect() {
        let mut draw_list = DrawList::new();
        
        draw_list.add_rect(Rect::new(0.0, 0.0, 10.0, 10.0), RED);
        
        let pos = DrawListPos(0);
        let transparent_color = Color::rgba(1.0, 0.0, 0.0, 0.0);
        
        draw_list.insert_rect_at(pos, Rect::new(5.0, 5.0, 10.0, 10.0), transparent_color);
        
        // Should not insert transparent rectangles
        assert_eq!(draw_list.commands().len(), 1);
    }

    #[test]
    fn test_draw_list_viewport_culling() {
        let viewport = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut draw_list = DrawList::with_viewport(viewport);
        
        // Rectangle within viewport - should be added
        let visible_rect = Rect::new(25.0, 25.0, 50.0, 50.0);
        draw_list.add_rect(visible_rect, RED);
        
        // Rectangle outside viewport - should be culled
        let invisible_rect = Rect::new(200.0, 200.0, 50.0, 50.0);
        draw_list.add_rect(invisible_rect, BLUE);
        
        assert_eq!(draw_list.commands().len(), 1);
        assert_eq!(draw_list.culling_stats().rendered_count, 1);
        assert_eq!(draw_list.culling_stats().culled_count, 1);
        assert_eq!(draw_list.culling_stats().culling_percentage(), 50.0);
    }

    #[test]
    fn test_draw_list_debug_culling_mode() {
        let viewport = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut draw_list = DrawList::with_viewport(viewport);
        draw_list.set_debug_culling(true);
        
        // Rectangle outside viewport with debug mode
        let invisible_rect = Rect::new(200.0, 200.0, 50.0, 50.0);
        draw_list.add_rect(invisible_rect, BLUE);
        
        // Should still add the command for debug visualization
        assert_eq!(draw_list.commands().len(), 1);
        assert_eq!(draw_list.culling_stats().culled_count, 1);
        
        // Check that debug color was applied
        match &draw_list.commands()[0] {
            DrawCommand::Rect { rect, color } => {
                assert_eq!(*rect, invisible_rect);
                assert_eq!(color.red, 1.0);
                assert_eq!(color.alpha, 0.2);
            }
            _ => panic!("Expected debug Rect command"),
        }
    }

    #[test]
    fn test_draw_list_add_frame() {
        let mut draw_list = DrawList::new();
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        let style = ElementStyle {
            fill: Fill::Solid(RED),
            border_width: 2.0,
            border_color: BLACK,
            ..ElementStyle::default()
        };
        
        draw_list.add_frame(rect, style.clone());
        
        assert!(!draw_list.is_empty());
        assert_eq!(draw_list.commands().len(), 1);
        assert_eq!(draw_list.culling_stats().rendered_count, 1);
        
        match &draw_list.commands()[0] {
            DrawCommand::Frame { rect: r, style: s } => {
                assert_eq!(*r, rect);
                assert_eq!(*s, style);
            }
            _ => panic!("Expected Frame command"),
        }
    }

    #[test]
    fn test_draw_list_add_frame_fully_transparent() {
        let mut draw_list = DrawList::new();
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        let style = ElementStyle {
            fill: Fill::Solid(TRANSPARENT),
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: None,
            ..ElementStyle::default()
        };
        
        draw_list.add_frame(rect, style);
        
        // Should not add fully transparent frames
        assert!(draw_list.is_empty());
        assert_eq!(draw_list.culling_stats().rendered_count, 0);
    }

    #[test]
    fn test_draw_list_add_frame_with_shadow() {
        let mut draw_list = DrawList::new();
        let rect = Rect::new(50.0, 50.0, 100.0, 50.0);
        let shadow = Shadow {
            color: Color::rgba(0.0, 0.0, 0.0, 0.5),
            blur: 5.0,
            offset: Vec2::new(2.0, 2.0),
        };
        let style = ElementStyle {
            fill: Fill::Solid(WHITE),
            shadow: Some(shadow),
            ..ElementStyle::default()
        };
        
        draw_list.add_frame(rect, style.clone());
        
        assert!(!draw_list.is_empty());
        assert_eq!(draw_list.commands().len(), 1);
        
        match &draw_list.commands()[0] {
            DrawCommand::Frame { rect: r, style: s } => {
                assert_eq!(*r, rect);
                assert_eq!(*s, style);
            }
            _ => panic!("Expected Frame command"),
        }
    }

    #[test]
    fn test_draw_list_commands_access() {
        let mut draw_list = DrawList::new();
        let rect = Rect::new(0.0, 0.0, 50.0, 50.0);
        
        draw_list.add_rect(rect, RED);
        
        // Test immutable access
        let commands = draw_list.commands();
        assert_eq!(commands.len(), 1);
        
        // Test mutable access
        let commands_mut = draw_list.commands_mut();
        commands_mut.push(DrawCommand::Rect { rect, color: BLUE });
        
        assert_eq!(draw_list.commands().len(), 2);
    }

    // === Integration Tests ===

    #[test]
    fn test_draw_list_complex_scenario() {
        let mut draw_list = DrawList::new();
        
        // Add background
        draw_list.add_rect(Rect::new(0.0, 0.0, 800.0, 600.0), WHITE);
        
        // Add clipped content
        draw_list.push_clip(Rect::new(100.0, 100.0, 600.0, 400.0));
        draw_list.add_rect(Rect::new(50.0, 50.0, 100.0, 100.0), RED); // Partially clipped
        draw_list.add_text(Vec2::new(200.0, 250.0), "Hello World", TextStyle::default());
        draw_list.pop_clip();
        
        // Add footer
        draw_list.add_rect(Rect::new(0.0, 550.0, 800.0, 50.0), BLACK);
        
        assert_eq!(draw_list.commands().len(), 6); // Background, PushClip, Rect, Text, PopClip, Footer
        assert_eq!(draw_list.culling_stats().rendered_count, 4); // Actual content rendered
    }
}
