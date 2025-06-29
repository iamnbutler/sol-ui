use glam::Vec2;
use palette::{Srgba, named};

/// Re-export palette's Srgba as our Color type
pub type Color = Srgba;

/// Common color constants
pub mod colors {
    use super::*;

    pub const WHITE: Color = Srgba::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Color = Srgba::new(0.0, 0.0, 0.0, 1.0);
    pub const RED: Color = Srgba::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Color = Srgba::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Color = Srgba::new(0.0, 0.0, 1.0, 1.0);
    pub const TRANSPARENT: Color = Srgba::new(0.0, 0.0, 0.0, 0.0);
}

/// Helper trait for Color
pub trait ColorExt {
    fn rgb(r: f32, g: f32, b: f32) -> Self;
    fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self;
    fn with_alpha(self, alpha: f32) -> Self;
}

impl ColorExt for Color {
    fn rgb(r: f32, g: f32, b: f32) -> Self {
        Srgba::new(r, g, b, 1.0)
    }

    fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Srgba::new(r, g, b, a)
    }

    fn with_alpha(self, alpha: f32) -> Self {
        Srgba::new(self.red, self.green, self.blue, alpha)
    }
}

/// A rectangle defined by position and size
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub pos: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            size: Vec2::new(width, height),
        }
    }

    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self { pos, size }
    }

    pub fn min(&self) -> Vec2 {
        self.pos
    }

    pub fn max(&self) -> Vec2 {
        self.pos + self.size
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.pos.x
            && point.y >= self.pos.y
            && point.x <= self.pos.x + self.size.x
            && point.y <= self.pos.y + self.size.y
    }

    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let min = self.min().max(other.min());
        let max = self.max().min(other.max());

        if min.x < max.x && min.y < max.y {
            Some(Rect::from_pos_size(min, max - min))
        } else {
            None
        }
    }
}

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
            color: colors::BLACK,
        }
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
    /// Push a clipping rectangle
    PushClip { rect: Rect },
    /// Pop the current clipping rectangle
    PopClip,
}

/// A list of draw commands to be rendered
pub struct DrawList {
    commands: Vec<DrawCommand>,
    clip_stack: Vec<Rect>,
}

/// A marker for a position in the draw list
#[derive(Debug, Clone, Copy)]
pub struct DrawListPos(usize);

impl DrawList {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            clip_stack: Vec::new(),
        }
    }

    /// Add a filled rectangle to the draw list
    pub fn add_rect(&mut self, rect: Rect, color: Color) {
        // Skip if completely transparent
        if color.alpha <= 0.0 {
            return;
        }

        // TODO: Clip against current clip rect
        self.commands.push(DrawCommand::Rect { rect, color });
    }

    /// Add text to the draw list
    pub fn add_text(&mut self, position: Vec2, text: impl Into<String>, style: TextStyle) {
        let text = text.into();
        if text.is_empty() {
            return;
        }

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
    }

    /// Get all commands
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// Check if the draw list is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Record the current position in the draw list
    pub fn current_pos(&self) -> DrawListPos {
        DrawListPos(self.commands.len())
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
}

impl Default for DrawList {
    fn default() -> Self {
        Self::new()
    }
}
