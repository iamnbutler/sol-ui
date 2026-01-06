use crate::{
    color::Color,
    element::{Element, LayoutContext, PaintContext},
    geometry::Rect,
    render::PaintText,
    style::{FontWeight, TextStyle},
};
use taffy::prelude::*;

/// Create a new text element with default styling.
///
/// Use builder methods to customize:
/// ```ignore
/// text("Hello")
///     .size(24.0)
///     .color(colors::WHITE)
///     .weight(FontWeight::Bold)
/// ```
///
/// Or provide a complete TextStyle:
/// ```ignore
/// text("Hello").with_style(TextStyle { size: 24.0, ..Default::default() })
/// ```
pub fn text(content: impl Into<String>) -> Text {
    Text::new(content)
}

/// Create a text element with explicit style (backward compatible)
pub fn styled_text(content: impl Into<String>, style: TextStyle) -> Text {
    Text {
        content: content.into(),
        style,
        node_id: None,
    }
}

/// A simple text element with builder-style configuration
pub struct Text {
    content: String,
    style: TextStyle,
    node_id: Option<NodeId>,
}

impl Text {
    /// Create a new text element with default style
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: TextStyle::default(),
            node_id: None,
        }
    }

    /// Set the complete text style
    pub fn with_style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the font size in logical pixels
    pub fn size(mut self, size: f32) -> Self {
        self.style.size = size;
        self
    }

    /// Set the text color
    pub fn color(mut self, color: Color) -> Self {
        self.style.color = color;
        self
    }

    /// Set the font family name
    pub fn font_family(mut self, family: &'static str) -> Self {
        self.style.font_family = family;
        self
    }

    /// Set the font weight
    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.style.weight = weight;
        self
    }

    /// Set the line height multiplier
    pub fn line_height(mut self, line_height: f32) -> Self {
        self.style.line_height = line_height;
        self
    }

    /// Make the text bold (shorthand for weight(FontWeight::Bold))
    pub fn bold(mut self) -> Self {
        self.style.weight = FontWeight::Bold;
        self
    }
}

impl Element for Text {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        let node_id = ctx.request_text_layout(Style::default(), &self.content, &self.style);
        self.node_id = Some(node_id);
        node_id
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        if !ctx.is_visible(&bounds) {
            return;
        }

        ctx.paint_text(PaintText {
            position: bounds.pos,
            text: self.content.clone(),
            style: self.style.clone(),
        });
    }
}
