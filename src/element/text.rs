use crate::{
    color::Color,
    element::{Element, LayoutContext, PaintContext},
    geometry::Rect,
    render::PaintText,
    style::TextStyle,
};
use taffy::prelude::*;

/// Create a new text element with default styling
pub fn text(content: impl Into<String>) -> Text {
    Text::new(content)
}

/// Create a new text element with explicit styling
pub fn text_styled(content: impl Into<String>, style: TextStyle) -> Text {
    Text::with_style(content, style)
}

/// A simple text element
pub struct Text {
    content: String,
    style: TextStyle,
    node_id: Option<NodeId>,
}

impl Text {
    /// Create a new text element with default styling
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: TextStyle::default(),
            node_id: None,
        }
    }

    /// Create a new text element with explicit styling
    pub fn with_style(content: impl Into<String>, style: TextStyle) -> Self {
        Self {
            content: content.into(),
            style,
            node_id: None,
        }
    }

    /// Set the text style
    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the text size
    pub fn size(mut self, size: f32) -> Self {
        self.style.size = size;
        self
    }

    /// Set the text color
    pub fn color(mut self, color: Color) -> Self {
        self.style.color = color;
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
