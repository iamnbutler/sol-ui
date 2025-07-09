use crate::color::{Color, ColorExt, colors::WHITE};

use super::{DrawList, FrameStyle, IdStack, Rect, TextStyle, WidgetId};
use glam::Vec2;

/// The main context for immediate mode UI
pub struct ImmediateUiContext {
    /// Draw commands accumulated during the frame
    draw_list: DrawList,

    /// Widget ID stack for hierarchical ID generation
    id_stack: IdStack,

    /// Current cursor position for layout
    cursor: Vec2,

    /// Layout state
    layout: LayoutState,

    /// Window/screen dimensions
    screen_size: Vec2,
}

/// Layout state for automatic positioning
#[derive(Debug, Clone)]
struct LayoutState {
    /// Starting position of current layout group
    start_pos: Vec2,

    /// Maximum extent in the cross-axis direction
    max_cross_axis: f32,

    /// Current layout direction
    direction: LayoutDirection,

    /// Spacing between elements
    spacing: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LayoutDirection {
    Vertical,
    Horizontal,
}

impl ImmediateUiContext {
    /// Create a new UI context with the given screen dimensions
    pub fn new(screen_size: Vec2) -> Self {
        Self {
            draw_list: DrawList::new(),
            id_stack: IdStack::new(),
            cursor: Vec2::ZERO,
            layout: LayoutState {
                start_pos: Vec2::ZERO,
                max_cross_axis: 0.0,
                direction: LayoutDirection::Vertical,
                spacing: 5.0,
            },
            screen_size,
        }
    }

    /// Begin a new frame. Call this before any UI elements.
    pub fn begin_frame(&mut self) {
        self.draw_list.clear();
        self.cursor = Vec2::new(10.0, 10.0); // Default margin
        self.layout.start_pos = self.cursor;
        self.layout.max_cross_axis = 0.0;
        self.id_stack.reset_child_counter();
    }

    /// End the current frame and return the draw list
    pub fn end_frame(&mut self) -> &DrawList {
        &self.draw_list
    }

    /// Get the draw list without ending the frame (for layer system)
    pub fn draw_list(&self) -> &DrawList {
        &self.draw_list
    }

    /// Set the current cursor position for manual layout
    pub fn set_cursor(&mut self, pos: Vec2) {
        self.cursor = pos;
        self.layout.start_pos = pos;
    }

    /// Get the current cursor position
    pub fn cursor(&self) -> Vec2 {
        self.cursor
    }

    /// Get the screen dimensions
    pub fn screen_size(&self) -> Vec2 {
        self.screen_size
    }

    /// Update screen dimensions (e.g., on window resize)
    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.screen_size = Vec2::new(width, height);
    }

    /// Draw text at the current cursor position
    pub fn text(&mut self, text: impl Into<String>) {
        self.text_styled(text, TextStyle::default());
    }

    /// Draw text with custom styling
    pub fn text_styled(&mut self, text: impl Into<String>, style: TextStyle) {
        let text = text.into();

        // Add text to draw list
        self.draw_list.add_text(self.cursor, &text, style.clone());

        // Advance cursor (approximate - we don't have proper text metrics yet)
        let text_height = style.size;
        let text_width = text.len() as f32 * style.size * 0.6; // Rough approximation

        self.advance_cursor(Vec2::new(text_width, text_height));
    }

    /// Begin a group (container) with optional background
    pub fn group<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.group_styled(None, f)
    }

    /// Begin a group with background color
    pub fn group_styled<R>(
        &mut self,
        background: Option<Color>,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        // Save current layout state
        let saved_cursor = self.cursor;
        let saved_layout = self.layout.clone();

        // Generate ID for this group
        let id = self.next_id();
        self.id_stack.push(id);

        // Record starting position
        let start_pos = self.cursor;

        // Reset layout for group content
        self.layout.start_pos = self.cursor;
        self.layout.max_cross_axis = 0.0;

        // Record position before content (for background insertion)
        let draw_pos = self.draw_list.current_pos();

        // Call the group content function
        let result = f(self);

        // Calculate group bounds
        let size = match self.layout.direction {
            LayoutDirection::Vertical => {
                Vec2::new(self.layout.max_cross_axis, self.cursor.y - start_pos.y)
            }
            LayoutDirection::Horizontal => {
                Vec2::new(self.cursor.x - start_pos.x, self.layout.max_cross_axis)
            }
        };

        // Draw background if specified (insert at recorded position)
        if let Some(color) = background {
            self.draw_list
                .insert_rect_at(draw_pos, Rect::from_pos_size(start_pos, size), color);
        }

        // Restore layout state and advance cursor
        self.cursor = saved_cursor;
        self.layout = saved_layout;
        self.advance_cursor(size);

        // Pop ID
        self.id_stack.pop();

        result
    }

    /// Create a vertical layout group
    pub fn vertical<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let saved_direction = self.layout.direction;
        self.layout.direction = LayoutDirection::Vertical;
        let result = self.group(f);
        self.layout.direction = saved_direction;
        result
    }

    /// Create a horizontal layout group
    pub fn horizontal<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let saved_direction = self.layout.direction;
        self.layout.direction = LayoutDirection::Horizontal;
        let result = self.group(f);
        self.layout.direction = saved_direction;
        result
    }

    /// Add spacing in the current layout direction
    pub fn space(&mut self, amount: f32) {
        match self.layout.direction {
            LayoutDirection::Vertical => self.cursor.y += amount,
            LayoutDirection::Horizontal => self.cursor.x += amount,
        }
    }

    /// Draw a colored rectangle
    pub fn rect(&mut self, size: Vec2, color: Color) {
        let rect = Rect::from_pos_size(self.cursor, size);
        self.draw_list.add_rect(rect, color);
        self.advance_cursor(size);
    }

    /// Draw a frame with rounded corners and optional border
    pub fn frame(&mut self, size: Vec2, style: FrameStyle) {
        let rect = Rect::from_pos_size(self.cursor, size);
        self.draw_list.add_frame(rect, style);
        self.advance_cursor(size);
    }

    /// Create a frame container that can hold other UI elements
    pub fn frame_container<R>(&mut self, style: FrameStyle, f: impl FnOnce(&mut Self) -> R) -> R {
        // Save current layout state
        let saved_cursor = self.cursor;
        let saved_layout = self.layout.clone();

        // Generate ID for this frame
        let id = self.next_id();
        self.id_stack.push(id);

        // Record starting position
        let start_pos = self.cursor;

        // Reset layout for frame content
        self.layout.start_pos = self.cursor;
        self.layout.max_cross_axis = 0.0;

        // Draw the frame background first
        let temp_rect = Rect::from_pos_size(start_pos, Vec2::new(1.0, 1.0));
        self.draw_list.add_frame(temp_rect, style.clone());
        let frame_cmd_pos = self.draw_list.commands().len() - 1;

        // Push clipping for content
        self.draw_list.push_clip(temp_rect);
        let clip_cmd_pos = self.draw_list.commands().len() - 1;

        // Call the frame content function
        let result = f(self);

        // Pop clipping
        self.draw_list.pop_clip();

        // Calculate actual frame bounds
        let size = match self.layout.direction {
            LayoutDirection::Vertical => {
                Vec2::new(self.layout.max_cross_axis, self.cursor.y - start_pos.y)
            }
            LayoutDirection::Horizontal => {
                Vec2::new(self.cursor.x - start_pos.x, self.layout.max_cross_axis)
            }
        };

        // Update frame command with actual size
        let frame_rect = Rect::from_pos_size(start_pos, size);
        let commands = self.draw_list.commands_mut();
        if let Some(super::DrawCommand::Frame { rect, .. }) = commands.get_mut(frame_cmd_pos) {
            *rect = frame_rect;
        }

        // Update clip command with actual size
        if let Some(super::DrawCommand::PushClip { rect }) = commands.get_mut(clip_cmd_pos) {
            *rect = frame_rect;
        }

        // Restore layout state and advance cursor
        self.cursor = saved_cursor;
        self.layout = saved_layout;
        self.advance_cursor(size);

        // Pop ID
        self.id_stack.pop();

        result
    }

    /// Create a frame container with padding
    pub fn frame_container_padded<R>(
        &mut self,
        style: FrameStyle,
        padding: f32,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.frame_container(style, |ui| {
            // Apply padding
            ui.cursor += Vec2::new(padding, padding);
            ui.layout.start_pos = ui.cursor;

            let result = f(ui);

            // Add padding to final size
            ui.cursor += Vec2::new(padding, padding);

            result
        })
    }

    /// Advance cursor based on element size and current layout
    pub fn advance_cursor(&mut self, size: Vec2) {
        match self.layout.direction {
            LayoutDirection::Vertical => {
                self.cursor.y += size.y + self.layout.spacing;
                self.layout.max_cross_axis = self.layout.max_cross_axis.max(size.x);
            }
            LayoutDirection::Horizontal => {
                self.cursor.x += size.x + self.layout.spacing;
                self.layout.max_cross_axis = self.layout.max_cross_axis.max(size.y);
            }
        }
    }

    /// Generate the next widget ID
    fn next_id(&self) -> WidgetId {
        // For now, use a simple counter-based ID
        // In a real implementation, this would use the widget_id! macro
        WidgetId::from_source_location("ui", 0, 0)
    }
}

/// Builder-style methods for common patterns
impl ImmediateUiContext {
    /// Create a centered group
    pub fn centered<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let center = self.screen_size * 0.5;
        self.set_cursor(center);
        self.group(f)
    }

    /// Create a window-style container
    pub fn window<R>(
        &mut self,
        title: &str,
        pos: Vec2,
        size: Vec2,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.set_cursor(pos);

        // Window background
        self.draw_list
            .add_rect(Rect::from_pos_size(pos, size), Color::rgb(0.2, 0.2, 0.2));

        // Title bar
        let title_height = 24.0;
        self.draw_list.add_rect(
            Rect::from_pos_size(pos, Vec2::new(size.x, title_height)),
            Color::rgb(0.3, 0.3, 0.3),
        );

        // Title text
        self.set_cursor(pos + Vec2::new(8.0, 4.0));
        self.text_styled(
            title,
            TextStyle {
                size: 16.0,
                color: WHITE,
            },
        );

        // Content area
        self.set_cursor(pos + Vec2::new(8.0, title_height + 8.0));

        // Clip content to window bounds
        self.draw_list.push_clip(Rect::from_pos_size(
            pos + Vec2::new(0.0, title_height),
            Vec2::new(size.x, size.y - title_height),
        ));

        let result = f(self);

        self.draw_list.pop_clip();

        result
    }
}
