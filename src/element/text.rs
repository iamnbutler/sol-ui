use crate::{
    element::{Element, LayoutContext, PaintContext},
    geometry::Rect,
    render::PaintText,
    style::TextStyle,
};
use taffy::prelude::*;

/// Create a new text element
pub fn text(content: impl Into<String>, style: TextStyle) -> Text {
    Text::new(content, style)
}

/// A simple text element
pub struct Text {
    content: String,
    style: TextStyle,
    node_id: Option<NodeId>,
}

impl Text {
    pub fn new(content: impl Into<String>, style: TextStyle) -> Self {
        Self {
            content: content.into(),
            style,
            node_id: None,
        }
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
            measured_size: Some(bounds.size),
        });
    }
}
